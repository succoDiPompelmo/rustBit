use std::cmp;
use std::sync::mpsc::{Receiver, Sender};

use crate::messages::Message;
use crate::peer::Peer;
use crate::torrent::torrent::Info;
use crate::torrent::writer::write_piece;

const BLOCK_SIZE: usize = 16384;
const INFO_PIECE_SIZE: usize = 16384;

struct PeerManager {
    buffer: Vec<Message>,
    buffer_capacity: usize,
    is_downloading: bool,
}

impl PeerManager {
    fn new(buffer_capacity: usize) -> PeerManager {
        PeerManager {
            buffer: vec![],
            buffer_capacity,
            is_downloading: false,
        }
    }

    fn apply_message(&mut self, message: &Message) {
        if message.is_extension_data_message() || message.is_request_message() {
            self.buffer.push(message.clone());
            self.is_downloading = false;
        }
    }

    fn is_buffer_full(&self) -> bool {
        self.buffer.len() >= self.buffer_capacity
    }

    fn try_assemble(&mut self, peer: &mut Peer) -> Option<Vec<u8>> {
        peer.get_stream().read_message().map_or((), |msg| {
            peer.apply_message(&msg);
            self.apply_message(&msg);
        });

        if self.is_buffer_full() {
            return Some(self.assemble_buffer());
        };
        None
    }

    fn assemble_buffer(&self) -> Vec<u8> {
        self.buffer
            .iter()
            .flat_map(|el| el.get_content_data())
            .collect()
    }

    fn get_offset(&self, offset_unit: usize) -> usize {
        self.buffer.len() * offset_unit
    }

    fn set_downloading(&mut self) {
        self.is_downloading = true
    }

    fn is_ready(&self) -> bool {
        !self.is_downloading
    }
}

fn get_block_size(block_offset: usize, torrent_length: usize, piece_offset: usize) -> usize {
    cmp::min(torrent_length - (block_offset + piece_offset), BLOCK_SIZE)
}

fn init_download(peer: &mut Peer) {
    peer.get_stream().send_interested();

    loop {
        peer.get_stream()
            .read_message()
            .map_or((), |msg| peer.apply_message(&msg));

        if peer.get_metadata_size() != 0 && !peer.is_choked() {
            return;
        }
    }
}

fn download_info(peer: &mut Peer) -> Result<Info, &'static str> {
    let mut manager =
        PeerManager::new((0..peer.get_metadata_size()).step_by(INFO_PIECE_SIZE).len());

    loop {
        if let Some(info_buffer) = manager.try_assemble(peer) {
            let info = Info::from_bytes(info_buffer)?;
            return Ok(info);
        }

        if manager.is_ready() {
            peer.request_info_piece(manager.get_offset(1));
            manager.set_downloading();
        }
    }
}

fn download_piece(peer: &mut Peer, info: &Info, piece_idx: usize) -> Result<Vec<u8>, &'static str> {
    let mut manager = PeerManager::new((0..info.get_piece_length()).step_by(BLOCK_SIZE).len());

    loop {
        if let Some(piece) = manager.try_assemble(peer) {
            return Ok(piece);
        }

        if manager.is_ready() {
            let block_offset = manager.get_offset(BLOCK_SIZE);
            let block_size = get_block_size(
                block_offset,
                info.get_total_length(),
                piece_idx * info.get_piece_length(),
            );

            peer.request_piece(block_size as u32, block_offset as u32, piece_idx as u32);
            manager.set_downloading();
        }
    }
}

pub fn peer_thread(peer: &mut Peer, tx: Sender<Info>, piece_rx: Receiver<usize>) {
    init_download(peer);
    let info = download_info(peer).unwrap();
    tx.send(info.clone()).unwrap();

    loop {
        if let Ok(piece_idx) = piece_rx.recv() {
            println!("{:?}", piece_idx);
            let piece = download_piece(peer, &info, piece_idx).unwrap();
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
