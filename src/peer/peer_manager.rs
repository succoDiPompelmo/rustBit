use std::cmp;
use std::sync::mpsc::{Receiver, Sender};

use crate::messages::extension::ExtensionMessage;
use crate::messages::request::RequestMessage;
use crate::messages::Message;
use crate::peer::Peer;
use crate::torrent::torrent::Info;
use crate::torrent::writer::write_piece;

const BLOCK_SIZE: usize = 16384;
const INFO_PIECE_SIZE: usize = 16384;

struct PeerInfoManager {
    piece_buffer: Vec<ExtensionMessage>,
    pieces_count: usize,
    is_downloading: bool,
}

impl PeerInfoManager {
    fn new(metadata_size: usize) -> PeerInfoManager {
        PeerInfoManager {
            piece_buffer: vec![],
            pieces_count: (0..metadata_size).step_by(INFO_PIECE_SIZE).len(),
            is_downloading: false,
        }
    }

    fn apply_message(&mut self, message: &Message) {
        if let Some(extension_message) = message.get_extension_data_message() {
            self.piece_buffer.push(extension_message.clone());
            self.is_downloading = false;
        }
    }

    fn is_info_ready(&self) -> bool {
        self.piece_buffer.len() >= self.pieces_count
    }

    fn get_info(&self) -> Vec<u8> {
        self.piece_buffer
            .iter()
            .map(|el| el.get_data())
            .flatten()
            .collect()
    }

    fn get_piece_offset(&self) -> usize {
        self.piece_buffer.len()
    }

    fn set_downloading(&mut self) {
        self.is_downloading = true
    }

    fn is_ready(&self) -> bool {
        !self.is_downloading
    }
}

struct PeerRequestManager {
    block_buffer: Vec<RequestMessage>,
    blocks_count: usize,
    is_downloading: bool,
}

impl PeerRequestManager {
    fn new(piece_length: usize) -> PeerRequestManager {
        PeerRequestManager {
            block_buffer: vec![],
            blocks_count: (0..piece_length).step_by(BLOCK_SIZE).len(),
            is_downloading: false,
        }
    }

    fn is_piece_ready(&self) -> bool {
        self.block_buffer.len() >= self.blocks_count
    }

    fn get_piece(&self) -> Vec<u8> {
        self.block_buffer
            .iter()
            .map(|el| el.get_block_data())
            .flatten()
            .collect()
    }

    fn apply_message(&mut self, message: &Message) {
        if let Some(request_message) = message.get_request_message() {
            self.block_buffer.push(request_message.clone());
            self.is_downloading = false;
        }
    }

    fn get_block_offset(&self) -> usize {
        self.block_buffer.len() * BLOCK_SIZE
    }

    fn get_block_size(&self, torrent_length: usize, piece_offset: usize) -> usize {
        cmp::min(
            torrent_length - (self.get_block_offset() + piece_offset),
            BLOCK_SIZE,
        )
    }

    fn is_ready(&self) -> bool {
        !self.is_downloading
    }

    fn set_downloading(&mut self) {
        self.is_downloading = true
    }
}

// Perform all the required checks before the download can start.
fn init_download(peer: &mut Peer) {
    peer.get_stream().send_interested();

    loop {
        if let Some(message) = peer.get_stream().read_message() {
            peer.apply_message(&message);
        }

        if peer.get_metadata_size() != 0 && !peer.is_choked() {
            return;
        }
    }
}

fn download_info(peer: &mut Peer) -> Result<Info, &'static str> {
    let mut manager = PeerInfoManager::new(peer.get_metadata_size());

    loop {
        if let Some(message) = peer.get_stream().read_message() {
            peer.apply_message(&message);
            manager.apply_message(&message);
        }

        if !manager.is_ready() {
            continue;
        }

        if manager.is_info_ready() {
            return Ok(Info::from_bytes(manager.get_info())?);
        };

        let extension_id = peer.get_extension_id("ut_metadata").unwrap();
        peer.get_stream()
            .send_metadata_request(extension_id, manager.get_piece_offset());

        manager.set_downloading();
    }
}

fn download_piece(info: &Info, piece_idx: usize, peer: &mut Peer) -> Result<Vec<u8>, &'static str> {
    let mut manager = PeerRequestManager::new(info.get_piece_length());

    loop {
        if let Some(message) = peer.get_stream().read_message() {
            peer.apply_message(&message);
            manager.apply_message(&message);
        }

        if !manager.is_ready() {
            continue;
        }

        if manager.is_piece_ready() {
            return Ok(manager.get_piece());
        }

        let block_offset = manager.get_block_offset();
        let block_size =
            manager.get_block_size(info.get_total_length(), piece_idx * info.get_piece_length());

        peer.get_stream()
            .send_request(block_size as u32, block_offset as u32, piece_idx as u32);

        manager.set_downloading();
    }
}

pub fn peer_thread(peer: &mut Peer, tx: Sender<Info>, piece_rx: Receiver<usize>) {
    init_download(peer);
    let info = download_info(peer).unwrap();
    tx.send(info.clone()).unwrap();

    loop {
        if let Ok(piece_idx) = piece_rx.recv() {
            println!("{:?}", piece_idx);
            let piece = download_piece(&info, piece_idx, peer).unwrap();
            if info.verify_piece(&piece, piece_idx) {
                write_piece(
                    piece,
                    piece_idx,
                    info.get_piece_length(),
                    info.get_files().unwrap(),
                )
            } else {
                panic!();
            }
        } else {
            return;
        }
    }
}
