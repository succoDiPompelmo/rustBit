use std::sync::{Arc, Mutex};

use crate::messages::{new_handshake, new_interested};
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::torrent::writer::write_piece;

use super::download::{download, Downloadable};

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
    if let Ok(mut mutex_info) = info_arc.lock() {
        if (*mutex_info).is_none() {
            let info_bytes = download(peer, Downloadable::Info).unwrap();
            let info = Info::from_bytes(info_bytes).unwrap();
            *mutex_info = Some(info);
        }

        let info = mutex_info.as_ref().unwrap();
        return (info.get_piece_length(), info.get_total_length());
    }

    panic!("Error during info lock")
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
        let piece = download(
            peer,
            Downloadable::Block((piece_length, piece_idx, total_length)),
        )
        .unwrap();

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
