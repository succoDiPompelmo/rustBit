use std::cmp;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
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
    peer: Peer,
    default_piece_length: usize,
    block_buffer: Vec<RequestMessage>,
    is_block_downloading: bool,
    is_info_downloading: bool,
    block_size: usize,
}

impl Manager {
    fn new(peer: Peer) -> Manager {
        Manager {
            info_buffer: vec![],
            peer,
            default_piece_length: 0,
            block_buffer: vec![],
            is_block_downloading: false,
            is_info_downloading: false,
            block_size: 16384,
        }
    }

    fn reset_block(&mut self) {
        self.block_buffer.clear();
    }

    fn get_block_buffer(&mut self) -> &mut Vec<RequestMessage> {
        &mut self.block_buffer
    }

    fn get_peer(&mut self) -> &mut Peer {
        &mut self.peer
    }

    fn set_default_piece_length(&mut self, length: usize) {
        self.default_piece_length = length;
    }

    fn is_piece_complete(&self) -> bool {
        self.block_buffer.len() * self.block_size >= self.default_piece_length
    }

    fn manage_inbound_messages(&mut self) -> Result<(), &'static str> {
        let stream = self.peer.get_stream();
        let message = stream.read_message().ok_or("No message to read")?;
        self.peer.apply_message(&message);

        if let ContentType::Request(request_message) = message.get_content() {
            self.block_buffer.push(request_message.clone());
            self.is_block_downloading = false;
        }
        if let ContentType::Extension(extension_message) = message.get_content() {
            if extension_message.get_msg_type().is_some() {
                self.info_buffer.push(extension_message.clone());
                self.is_info_downloading = false;
            }
        }
        Ok(())
    }

    fn get_block_size(&self, info: &Info, piece_idx: usize) -> u32 {
        let remainder = info.get_total_length()
            - (self.default_piece_length * piece_idx + self.block_size * self.block_buffer.len());
        cmp::min(remainder, self.block_size) as u32
    }
}

// Perform all the required checks before the download can start.
fn init_download(manager: &mut Manager) -> Result<Info, &'static str> {
    manager.get_peer().get_stream().send_interested();

    loop {
        manager.manage_inbound_messages();

        if manager.get_peer().is_choked() {
            continue;
        }

        if manager.get_peer().get_metadata_size() == 0 {
            continue;
        }

        if manager.is_info_downloading {
            continue;
        }

        let tot_info_pieces = (0..manager.peer.get_metadata_size()).step_by(16384).len();

        if manager.info_buffer.len() == tot_info_pieces {
            println!("METADATA READY");
            let mut info: Vec<u8> = vec![];
            for info_message in &manager.info_buffer {
                info.extend(info_message.get_data());
            }
            return Ok(Info::from_bytes(info).unwrap());
        };

        let piece = manager.info_buffer.len();
        let extension_id = manager.peer.get_extension_id("ut_metadata").unwrap();

        println!("Info piece requested {:?}", piece);

        manager
            .peer
            .get_stream()
            .send_metadata_request(extension_id, piece);

        manager.is_info_downloading = true;
    }
}

fn download_piece(
    manager: &mut Manager,
    info: &Info,
    piece_idx: usize,
) -> Result<(), &'static str> {
    loop {
        manager.manage_inbound_messages();

        if manager.is_block_downloading {
            continue;
        }

        if manager.is_piece_complete() {
            let mut piece: Vec<u8> = vec![];

            for block_message in manager.get_block_buffer() {
                piece.extend(&block_message.get_block_data());
            }

            if !info.verify_piece(&piece, piece_idx) {
                return Err("Error in piece verification");
            }

            manager.reset_block();

            get_file_writers(
                info.get_files().unwrap(),
                piece,
                piece_idx as u32,
                info.get_piece_length() as u32,
            )
            .iter()
            .for_each(|writer| writer.write_to_filesystem());
            return Ok(());
        }

        let expecetd_block_size = manager.get_block_size(&info, piece_idx);
        let block_buffer_size = manager.block_buffer.len();

        manager.peer.get_stream().send_request(
            expecetd_block_size,
            (block_buffer_size * manager.block_size) as u32,
            piece_idx as u32,
        );

        manager.is_block_downloading = true;
    }
}

fn peer_thread(peer: Peer, tx: Sender<Info>, piece_rx: Receiver<usize>) {
    let mut manager = Manager::new(peer);
    let info = init_download(&mut manager).unwrap();
    manager.set_default_piece_length(info.get_piece_length());
    tx.send(info.clone()).unwrap();

    loop {
        if let Ok(piece_idx) = piece_rx.recv() {
            println!("{:?}", piece_idx);
            download_piece(&mut manager, &info, piece_idx).unwrap();
        } else {
            return;
        }
    }
}

pub fn download(
    peers_info: Vec<PeerConnectionInfo>,
    peer_id: &str,
    torrent: &mut Torrent,
) -> Result<(), &'static str> {
    let (tx, rx): (Sender<Info>, Receiver<Info>) = mpsc::channel();
    let (piece_tx, piece_rx): (Sender<usize>, Receiver<usize>) = mpsc::channel();

    let peer = get_peer(peers_info, peer_id, torrent.get_info_hash())
        .ok_or("No peers concluded an handshake with success")?;

    let handle = thread::spawn(|| peer_thread(peer, tx, piece_rx));
    if let Ok(info) = rx.recv() {
        for piece_idx in 0..info.get_total_pieces() {
            piece_tx.send(piece_idx).unwrap();
        }
    }

    match handle.join() {
        Err(err) => println!("{:?}", err),
        Ok(_) => (),
    };

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
