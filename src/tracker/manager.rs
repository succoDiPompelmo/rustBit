use sha1::{Digest, Sha1};
use std::cmp;
use std::time::Duration;
use std::time;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

use crate::messages::extension::ExtensionMessage;
use crate::messages::handshake;
use crate::messages::request::RequestMessage;
use crate::messages::ContentType;
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

    fn manage_info(&mut self) -> Result<Info, &'static str> {
        let tot_info_pieces = (0..self.peer.get_metadata_size()).step_by(16384).len();

        if self.info_buffer.len() == tot_info_pieces {
            println!("METADATA READY");
            let mut info: Vec<u8> = vec![];
            for info_message in &self.info_buffer {
                info.extend(info_message.get_data());
            }
            let info_2 = Info::from_bytes(info).unwrap();
            return Ok(info_2);
        };

        if self.info_buffer_ids.len() != tot_info_pieces {
            println!("SEND METADATA REQUEST");
            // If we finish the piece but we have not yet concluded the info download here
            // we will get an error of No piece available. DA CAMBIARE COMUNQUE FA SCHIFO.
            let piece = choose_piece_to_download(tot_info_pieces, self)?;
            let extension_id = self.peer.get_extension_id("ut_metadata").unwrap();

            self.peer
                .get_stream()
                .send_metadata_request(extension_id, piece);
            self.info_buffer_ids.push(piece);
            return Err("No info ready");
        };

        Err("Generic Error")
    }

    fn manage_request(&mut self, default_piece_length: usize, total_length: usize) -> Result<Vec<u8>, &'static str> {
        let block_size = 16384;

        if self.block_buffer.len() * block_size >= default_piece_length {
            let piece = make_piece(self);

            self.piece_index_request = self.piece_index_request + 1;
            self.block_buffer.clear();

            return Ok(piece);
        } else {
            let remainder = total_length
                - (default_piece_length * self.piece_index_request + block_size * self.block_buffer.len());
            let expecetd_block_size = cmp::min(remainder, block_size);

            let piece_index_request_u32 = self.piece_index_request as u32;
            let block_buffer_size = self.block_buffer.len();
            self.peer.get_stream().send_request(
                expecetd_block_size as u32,
                (block_buffer_size * block_size) as u32,
                piece_index_request_u32,
            );

            self.is_block_downloading = true;

            return Err("Piece still not completed");
        }
    }
}

// Perform all the required checks before the download can start.
fn init_download(manager: &mut Manager) -> Result<Info, &'static str> {
    loop {
        manager.manage_inbound_messages();

        if manager.get_peer().is_choked() {
            thread::sleep(time::Duration::from_secs(1));
            manager.get_peer().get_stream().send_interested();
            continue;
        }

        if manager.get_peer().get_metadata_size() != 0 {
            if let Ok(info) = manager.manage_info() {
                return Ok(info)
            }
        }
    }
}

fn download_piece(manager: &mut Manager, info: Info) {
    loop {
        manager.manage_inbound_messages();
            
        if manager.is_block_downloading() {
            continue
        }

        if let Ok(piece) = manager.manage_request(info.get_piece_length(), info.get_total_length()) {
            get_file_writers(
                info.get_files().unwrap(),
                piece,
                (manager.get_piece_index_request() - 1) as u32,
                info.get_piece_length() as u32,
            ).iter().for_each(|writer| writer.write_to_filesystem());
        }
    }
}

pub fn download(
    peers_info: Vec<PeerConnectionInfo>,
    peer_id: &str,
    torrent: &mut Torrent,
) -> Result<(), &'static str> {

    let (tx, rx): (Sender<Info>, Receiver<Info>) = mpsc::channel();

    let mut peer = get_peer(peers_info, peer_id, torrent.get_info_hash())
        .ok_or("No peers concluded an handshake with success")?;

    let handle = thread::spawn(move || {
        let mut manager = Manager::new(peer);

        let info = init_download(&mut manager).unwrap();
        tx.send(info.clone()).unwrap();
        download_piece(&mut manager, info);
    });

    // loop {
    //     let info = rx.recv();
    // }

    handle.join();

    Ok(())
}

pub fn get_peer<'arr>(
    peers_info: Vec<PeerConnectionInfo>,
    peer_id: &str,
    info_hash: Vec<u8>,
) -> Option<Peer> {
    for peer_info in peers_info {
        if let Ok(stream) = handshake::perform(peer_info, &info_hash, peer_id) {
            return Some(Peer::new(stream));
        }
    }
    return None;
}

fn make_piece(manager: &mut Manager) -> Vec<u8> {
    let mut piece: Vec<u8> = vec![];

    for block_message in manager.get_block_buffer() {
        piece.extend(&block_message.get_block_data());
    }
    piece
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
                return Ok(piece_index);
            }
        }
    }

    return Err("No piece index available");
}
