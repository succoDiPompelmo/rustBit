use std::cmp;
use std::sync::{Arc, Mutex};

use crate::messages::{new_handshake, new_interested, new_metadata, new_request, Message};
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::torrent::writer::write_piece;

const BLOCK_SIZE: usize = 16384;
const INFO_PIECE_SIZE: usize = 16384;

struct PeerManager {
    buffer: Vec<Message>,
    buffer_capacity: usize,
    is_downloading: bool,
    info_ready: bool,
}

impl PeerManager {
    fn new(buffer_capacity: usize, info_ready: bool) -> PeerManager {
        PeerManager {
            buffer: vec![],
            buffer_capacity,
            is_downloading: false,
            info_ready,
        }
    }

    fn apply_message(&mut self, message: &Message) {
        if (message.is_extension_data_message() && !self.info_ready) || message.is_request_message()
        {
            self.buffer.push(message.clone());
            self.is_downloading = false;
        }
    }

    fn is_buffer_full(&self) -> bool {
        self.buffer.len() >= self.buffer_capacity
    }

    fn try_assemble(&mut self, peer: &mut Peer) -> Option<Vec<u8>> {
        peer.read_message().map_or((), |msg| {
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

fn init_download(peer: &mut Peer) -> Result<(), &'static str> {
    peer.send_message(new_interested());
    peer.send_metadata_handshake_request();

    for _ in 0..100 {
        peer.read_message()
            .map_or((), |msg| peer.apply_message(&msg));

        if peer.get_metadata_size() != 0 && !peer.is_choked() {
            return Ok(());
        }
    }
    Err("Init download failed")
}

pub fn download_info(peer: &mut Peer) -> Result<Info, &'static str> {
    let mut manager = PeerManager::new(
        (0..peer.get_metadata_size()).step_by(INFO_PIECE_SIZE).len(),
        false,
    );

    for _ in 0..10 {
        if let Some(info_buffer) = manager.try_assemble(peer) {
            let info = Info::from_bytes(info_buffer)?;
            return Ok(info);
        }

        let metadata_id = peer.get_extension_id_by_name("ut_metadata");

        if manager.is_ready() {
            peer.send_message(new_metadata(metadata_id, manager.get_offset(1)));
            manager.set_downloading();
        }
    }
    Err("No info retrieved")
}

fn download_piece(
    peer: &mut Peer,
    piece_length: usize,
    piece_idx: usize,
    total_length: usize,
) -> Result<Vec<u8>, &'static str> {
    let mut manager = PeerManager::new((0..piece_length).step_by(BLOCK_SIZE).len(), true);

    loop {
        if let Some(piece) = manager.try_assemble(peer) {
            return Ok(piece);
        }

        if peer.is_choked() {
            panic!("Chocked peer")
        }

        if manager.is_ready() {
            let block_offset = manager.get_offset(BLOCK_SIZE);
            let block_size = get_block_size(block_offset, total_length, piece_idx * piece_length);

            peer.send_message(new_request(
                piece_idx as u32,
                block_offset as u32,
                block_size as u32,
            ));
            manager.set_downloading();
        }
    }
}

pub fn peer_thread_evp(
    peer: &mut Peer,
    info_arc: Arc<Mutex<Option<Info>>>,
    lock_counter: Arc<Mutex<usize>>,
) {
    peer.send_message(new_handshake(&peer.get_info_hash(), &peer.get_peer_id()));
    peer.read_message()
        .map_or((), |msg| peer.apply_message(&msg));

    if !peer.is_active() {
        panic!("Handshake failed")
    }
    println!("HANDSHA KE COMPLETED");

    init_download(peer).unwrap();

    let mut piece_length = 0;
    let mut total_length = 0;

    match info_arc.lock() {
        Ok(mut mutex_info) => {
            if let Some(info) = &mut *mutex_info {
                piece_length = info.get_piece_length();
                total_length = info.get_total_length();
            } else {
                *mutex_info = Some(download_info(peer).unwrap())
            }
        }
        _ => (),
    }

    println!("piece length: {:?}", piece_length);
    println!("total length: {:?}", total_length);

    let mut piece_idx = 0;

    loop {
        if let Ok(mut counter) = lock_counter.lock() {
            piece_idx = *counter + 1;
            *counter += 1;
        }

        println!("{:?} by peer", piece_idx);
        let piece = download_piece(peer, piece_length, piece_idx, total_length).unwrap();

        match info_arc.lock() {
            Ok(mut mutex_info) => {
                if let Some(info) = &mut *mutex_info {
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
                }
            }
            _ => println!("Error during lock acquisition to write piece"),
        }
    }
}
