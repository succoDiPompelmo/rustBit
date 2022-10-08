use std::sync::{Arc, Mutex};
use std::thread;

use crate::messages::handshake;
use crate::peer::peer_manager::{download_info, peer_thread};
use crate::peer::Peer;
use crate::torrent::{Info, Torrent};
use crate::tracker::PeerConnectionInfo;

pub fn download(
    peers_info: &Vec<PeerConnectionInfo>,
    peer_id: &str,
    torrent: &mut Torrent,
) -> Result<(), &'static str> {
    let piece_counter = Arc::new(Mutex::new(0));
    let info = get_info(peers_info, peer_id, torrent.get_info_hash())?;
    let mut handles = vec![];

    for peer_info in peers_info {
        if let Ok(stream) = handshake::perform(&peer_info, &torrent.get_info_hash(), peer_id) {
            let mut peer = Peer::new(stream);
            let info = info.clone();
            let counter_clone = piece_counter.clone();
            handles.push(thread::spawn(move || {
                peer_thread(&mut peer, &info, counter_clone)
            }));
        }
    }

    for handle in handles {
        match handle.join() {
            Err(err) => {
                println!("{:?}", err);
                return Err("Error in thread execution");
            }
            Ok(_) => return Ok(()),
        }
    }

    Ok(())
}

pub fn get_info(
    peers_info: &Vec<PeerConnectionInfo>,
    peer_id: &str,
    info_hash: Vec<u8>,
) -> Result<Info, &'static str> {
    for peer_info in peers_info {
        println!("{:?}", peer_info);
        if let Ok(stream) = handshake::perform(peer_info, &info_hash, peer_id) {
            let mut peer = Peer::new(stream);
            if let Ok(info) = download_info(&mut peer) {
                return Ok(info);
            }
        }
    }
    Err("No info downloaded")
}
