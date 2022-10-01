use std::cmp;
use std::sync::mpsc::{Receiver, Sender};

use crate::peer::Peer;
use crate::torrent::torrent::Info;
use crate::torrent::writer::write_piece;

const BLOCK_SIZE: usize = 16384;

// Perform all the required checks before the download can start.
fn init_download(peer: &mut Peer) -> Result<Info, &'static str> {
    peer.get_stream().send_interested();
    let mut info_buffer = vec![];
    let mut is_info_downloading = false;

    loop {
        if let Some(message) = peer.get_stream().read_message() {
            peer.apply_message(&message);
            if let Some(extension_message) = message.get_extension_data_message() {
                info_buffer.push(extension_message.clone());
                is_info_downloading = false;
            }
        }

        if peer.get_metadata_size() == 0 || is_info_downloading || peer.is_choked() {
            continue;
        }

        if info_buffer.len() == (0..peer.get_metadata_size()).step_by(16384).len() {
            return Ok(Info::from_bytes(
                info_buffer
                    .iter()
                    .map(|el| el.get_data())
                    .flatten()
                    .collect(),
            )?);
        };

        let extension_id = peer.get_extension_id("ut_metadata").unwrap();
        peer.get_stream()
            .send_metadata_request(extension_id, info_buffer.len());

        is_info_downloading = true;
    }
}

fn download_piece(
    default_piece_length: usize,
    torrent_total_length: usize,
    piece_idx: usize,
    peer: &mut Peer,
) -> Result<Vec<u8>, &'static str> {
    let mut block_buffer = vec![];
    let mut is_block_downloading = false;

    loop {
        if let Some(message) = peer.get_stream().read_message() {
            peer.apply_message(&message);
            if let Some(request_message) = message.get_request_message() {
                block_buffer.push(request_message.clone());
                is_block_downloading = false;
            }
        }

        if is_block_downloading {
            continue;
        }

        if block_buffer.len() * BLOCK_SIZE >= default_piece_length {
            let piece = block_buffer
                .iter()
                .map(|el| el.get_block_data())
                .flatten()
                .collect();

            block_buffer.clear();
            return Ok(piece);
        }

        let block_offset = block_buffer.len() * BLOCK_SIZE;
        let block_size = cmp::min(
            torrent_total_length - (block_offset + piece_idx * default_piece_length),
            BLOCK_SIZE,
        );

        peer.get_stream()
            .send_request(block_size as u32, block_offset as u32, piece_idx as u32);
            
        is_block_downloading = true;
    }
}

pub fn peer_thread(peer: &mut Peer, tx: Sender<Info>, piece_rx: Receiver<usize>) {
    let info = init_download(peer).unwrap();
    tx.send(info.clone()).unwrap();

    loop {
        if let Ok(piece_idx) = piece_rx.recv() {
            println!("{:?}", piece_idx);
            let piece = download_piece(
                info.get_piece_length(),
                info.get_total_length(),
                piece_idx,
                peer,
            )
            .unwrap();
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
