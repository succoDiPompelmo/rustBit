use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::messages::handshake;
use crate::peer::peer_manager::peer_thread;
use crate::peer::Peer;
use crate::torrent::torrent::{Info, Torrent};
use crate::tracker::tracker::PeerConnectionInfo;

pub fn download(
    peers_info: Vec<PeerConnectionInfo>,
    peer_id: &str,
    torrent: &mut Torrent,
) -> Result<(), &'static str> {
    let (tx, rx): (Sender<Info>, Receiver<Info>) = mpsc::channel();
    let (piece_tx, piece_rx): (Sender<usize>, Receiver<usize>) = mpsc::channel();

    let mut peer = get_peer(peers_info, peer_id, torrent.get_info_hash())
        .ok_or("No peers concluded an handshake with success")?;

    let handle = thread::spawn(move || peer_thread(&mut peer, tx, piece_rx));
    if let Ok(info) = rx.recv() {
        for piece_idx in 0..info.get_total_pieces() {
            piece_tx.send(piece_idx).unwrap();
        }
    }

    match handle.join() {
        Err(err) => {
            println!("{:?}", err);
            Err("Error in thread execution")
        }
        Ok(_) => Ok(()),
    }
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
    None
}
