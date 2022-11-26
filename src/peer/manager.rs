use std::cmp;
use std::sync::{Arc, Mutex};

use crate::messages::{new_handshake, new_interested, new_metadata, new_request, Message};
use crate::peer::buffer::MessageBuffer;
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::torrent::writer::write_piece;

const BLOCK_SIZE: usize = 16384;
const INFO_PIECE_SIZE: usize = 16384;

fn get_block_size(block_offset: usize, torrent_length: usize, piece_offset: usize) -> usize {
    cmp::min(torrent_length - (block_offset + piece_offset), BLOCK_SIZE)
}

pub fn download_info(peer: &mut Peer) -> Result<Vec<u8>, &'static str> {
    let next_info_piece = |peer: &mut Peer, piece_index| {
        let metadata_id = peer.get_extension_id_by_name("ut_metadata");
        peer.send_message(new_metadata(metadata_id, piece_index));
    };

    let mut buffer = MessageBuffer::new(
        Message::is_extension_data_message,
        next_info_piece,
        (0..peer.get_metadata_size()).step_by(INFO_PIECE_SIZE).len(),
    );

    for _ in 0..10 {
        peer.read_message().map_or((), |msg| {
            peer.apply_message(&msg);
            buffer.push_message(msg);
        });

        if buffer.is_full() {
            return Ok(buffer.assemble_content());
        };

        buffer.request_next_message(peer);
    }
    Err("No info retrieved")
}

fn download_piece(
    peer: &mut Peer,
    piece_length: usize,
    piece_index: usize,
    total_length: usize,
) -> Result<Vec<u8>, &'static str> {
    let next_piece_block = |peer: &mut Peer, block_index: usize| {
        let block_offset = block_index * BLOCK_SIZE;
        let block_length = get_block_size(block_offset, total_length, piece_index * piece_length);
        peer.send_message(new_request(
            piece_index as u32,
            block_offset as u32,
            block_length as u32,
        ));
    };

    let mut buffer = MessageBuffer::new(
        Message::is_request_message,
        next_piece_block,
        (0..peer.get_metadata_size()).step_by(INFO_PIECE_SIZE).len(),
    );

    loop {
        peer.read_message().map_or((), |msg| {
            peer.apply_message(&msg);
            buffer.push_message(msg);
        });

        if buffer.is_full() {
            return Ok(buffer.assemble_content());
        };

        if peer.is_choked() {
            panic!("Chocked peer")
        }

        buffer.request_next_message(peer);
    }
}

fn starup_peer(peer: &mut Peer) {
    peer.send_message(new_handshake(&peer.get_info_hash(), &peer.get_peer_id()));
    peer.read_message()
        .map_or((), |msg| peer.apply_message(&msg));

    if !peer.is_active() {
        panic!("Handshake failed")
    }

    peer.send_message(new_interested());
    peer.send_metadata_handshake_request();

    for _ in 0..100 {
        peer.read_message()
            .map_or((), |msg| peer.apply_message(&msg));

        if peer.is_ready() {
            return;
        }
    }
    panic!("Peer not ready");
}

fn prepare_info(peer: &mut Peer, info_arc: &Arc<Mutex<Option<Info>>>) -> (usize, usize) {
    let mut piece_length = 0;
    let mut total_length = 0;

    match info_arc.lock() {
        Ok(mut mutex_info) => {
            if let Some(info) = &mut *mutex_info {
                piece_length = info.get_piece_length();
                total_length = info.get_total_length();
            } else {
                let info_bytes = download_info(peer).unwrap();
                let info = Info::from_bytes(info_bytes).unwrap();
                piece_length = info.get_piece_length();
                total_length = info.get_total_length();
                *mutex_info = Some(info);
            }
            return (piece_length, total_length);
        }
        _ => panic!("Error during info lock"),
    }
}

pub fn peer_thread(
    peer: &mut Peer,
    info_arc: Arc<Mutex<Option<Info>>>,
    lock_counter: Arc<Mutex<usize>>,
) {
    starup_peer(peer);

    let (piece_length, total_length) = prepare_info(peer, &info_arc);
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
            Err(err) => {
                println!("Error during lock acquisition to write piece: {:?}", err);
                panic!("Error during lock acquisition")
            }
        }
    }
}
