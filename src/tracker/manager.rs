use std::cmp;
use sha1::{Digest, Sha1};
use std::{thread, time};
use std::time::Duration;

use crate::messages::ContentType;
use crate::messages::extension::ExtensionMessage;
use crate::messages::handshake;
use crate::messages::request::RequestMessage;
use crate::peer::Peer;
use crate::torrent::torrent::{Info, Torrent};
use crate::torrent::writer::get_file_writers;
use crate::tracker::tracker::PeerConnectionInfo;

struct Manager {
    info_buffer: Vec<ExtensionMessage>,
    info_buffer_ids: Vec<usize>,
    peer: Peer,
    piece_index_request: usize,
    block_buffer: Vec<RequestMessage>,
    is_block_downloading: bool,
    block_size: usize,
}

impl Manager {
    fn new(peer: Peer) -> Manager {
        Manager {
            info_buffer: vec![],
            info_buffer_ids: vec![],
            peer,
            piece_index_request: 0,
            block_buffer: vec![],
            is_block_downloading: false,
            block_size: 16384,
        }
    }

    fn get_info_buffer(&mut self) -> &mut Vec<ExtensionMessage> {
        &mut self.info_buffer
    }

    fn get_block_buffer(&mut self) -> &mut Vec<RequestMessage> {
        &mut self.block_buffer
    }

    fn get_info_buffer_ids(&mut self) -> &mut Vec<usize> {
        &mut self.info_buffer_ids
    }

    fn get_peer(&mut self) -> &mut Peer {
        &mut self.peer
    }

    fn is_block_downloading(&self) -> bool {
        self.is_block_downloading
    }

    fn get_piece_index_request(&self) -> usize {
        self.piece_index_request
    }

    fn manage_inbound_messages(&mut self) -> Result<(), &'static str> {
        let stream = self.peer.get_stream();
        let message = if let Some(message) = stream.read_message() {
            message
        } else {
            return Ok(());
        };

        self.peer.apply_message(&message);

        if let ContentType::Request(request_message) = message.get_content() {
            self.block_buffer.push(request_message.clone());
            self.is_block_downloading = false;
        }
        if let ContentType::Extension(extension_message) = message.get_content() {
            if extension_message.get_msg_type().is_some() {
                self.info_buffer.push(extension_message.clone());
            }
        }
        Ok(())
    }

    fn manage_info(&mut self, torrent: &mut Torrent) -> Result<(), &'static str> {
        let tot_info_pieces = (0..self.peer.get_metadata_size()).step_by(16384).len();

        if self.info_buffer.len() == tot_info_pieces {
            println!("METADATA READY");
            let mut info: Vec<u8> = vec![];
            for info_message in &self.info_buffer {
                info.extend(info_message.get_data());
            }
            torrent.set_info(Info::from_bytes(info).unwrap());
        } else {
            println!("SEND METADATA REQUEST");
            // If we finish the piece but we have not yet concluded the info download here
            // we will get an error of No piece available. DA CAMBIARE COMUNQUE FA SCHIFO.
            let piece = choose_piece_to_download(tot_info_pieces, self)?;
            let extension_id = self.peer.get_extension_id("ut_metadata").unwrap();

            self.peer
                .get_stream()
                .send_metadata_request(extension_id, piece);
            self.info_buffer_ids.push(piece);
        }

        Ok(())
    }

    fn manage_request(&mut self, torrent: &mut Torrent) {
        let block_size = 16384;

        let piece_length = torrent.get_piece_length().unwrap();
        let torrent_total_length = torrent.get_total_length().unwrap();

        if self.block_buffer.len() * block_size >= piece_length {
            make_piece(self, torrent).unwrap();

            self.piece_index_request = self.piece_index_request + 1;
            self.block_buffer.clear();

            if self.piece_index_request * piece_length >= torrent_total_length {
                return;
            }
        } else {
            let remainder = torrent_total_length
                - (piece_length * self.piece_index_request + block_size * self.block_buffer.len());
            let expecetd_block_size = cmp::min(remainder, block_size);

            let piece_index_request_u32 = self.piece_index_request as u32;
            let block_buffer_size = self.block_buffer.len();
            self.peer.get_stream().send_request(
                expecetd_block_size as u32,
                (block_buffer_size * block_size) as u32,
                piece_index_request_u32,
            );

            self.is_block_downloading = true;
        }
    }
}

pub fn download(
    peers_info: Vec<PeerConnectionInfo>,
    peer_id: &str,
    torrent: &mut Torrent,
) -> Result<(), &'static str> {
    let peer = get_peer(peers_info, peer_id, torrent)
        .ok_or("No peers concluded an handshake with success")?;
    let mut manager = Manager::new(peer);

    println!("START DOWNLOADING");

    loop {
        manager.manage_inbound_messages();

        if manager.get_peer().is_choked() {
            thread::sleep(time::Duration::from_secs(1));
            manager.get_peer().get_stream().send_interested();
            continue;
        }

        if torrent.get_info().is_err() && manager.get_peer().get_metadata_size() != 0 {
            manager.manage_info(torrent);
        }

        if torrent.get_info().is_ok() && !manager.is_block_downloading() {
            manager.manage_request(torrent);
        }
    }

    return Ok(());
}

pub fn get_peer<'arr>(
    peers_info: Vec<PeerConnectionInfo>,
    peer_id: &str,
    torrent: &Torrent,
) -> Option<Peer> {
    for peer_info in peers_info {
        if let Ok(stream) = handshake::perform(peer_info, &torrent.get_info_hash(), peer_id) {
            return Some(Peer::new(stream));
        }
    }
    return None;
}

fn make_piece(manager: &mut Manager, torrent: &Torrent) -> Result<(), &'static str> {
    let mut piece: Vec<u8> = vec![];

    for block_message in manager.get_block_buffer() {
        piece.extend(&block_message.get_block_data());
    }

    if verify_piece(&piece, torrent, manager.get_piece_index_request()) {
        write_piece(torrent, &piece, manager.get_piece_index_request());
        Ok(())
    } else {
        println!("Piece doesn't corresponde to pieces hash");
        return Err("Wrong piece");
    }
}

pub fn write_piece(
    torrent: &Torrent,
    piece: &Vec<u8>,
    piece_index_request: usize,
) -> Result<(), &'static str> {
    let file_writers = get_file_writers(
        torrent.get_files()?,
        piece.to_vec(),
        piece_index_request as u32,
        torrent.get_piece_length()? as u32,
    );

    for writer in file_writers {
        writer.write_to_filesystem();
    }

    Ok(())
}

pub fn verify_piece(piece: &Vec<u8>, torrent: &Torrent, piece_index_request: usize) -> bool {
    let piece_verifier = Sha1::digest(&piece).as_slice().to_owned();
    if let Ok(piece) = torrent.get_piece(piece_index_request) {
        piece_verifier == piece
    } else {
        false
    }
}

fn choose_piece_to_download(
    tot_info_pieces: usize,
    manager: &mut Manager,
) -> Result<usize, &'static str> {
    for piece_index in 0..tot_info_pieces {
        let mut already_downloaded = false;

        for info_message in manager.get_info_buffer() {
            if Some(piece_index) == info_message.get_piece() {
                println!("ALREADY DOWNLOADED PIECE");
                already_downloaded = true;
                break;
            }
        }

        if !already_downloaded && !manager.get_info_buffer_ids().contains(&piece_index) {
            if let Some(extension_id) = manager.get_peer().get_extension_id("ut_metadata") {
                println!("Sent metadata request for index {:?}", piece_index);
                return Ok(piece_index);
            }
        }
    }

    return Err("No piece index available");
}
