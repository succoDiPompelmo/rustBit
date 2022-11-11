use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::messages::handshake;
use crate::peer::peer_manager::{download_info, peer_thread};
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::tracker::PeerConnectionInfo;

pub fn manager_thread(
    peers_info_receiver: Receiver<Vec<PeerConnectionInfo>>,
    peer_id: &str,
    info_hash: &[u8],
) -> Result<(), &'static str> {
    loop {
        let peers_info = peers_info_receiver.recv().map_err(|_| "A")?;
        println!("Read peers_info");
        let piece_counter = Arc::new(Mutex::new(0));
        let info = get_info(&peers_info, peer_id, info_hash)?;
        let mut handles = vec![];

        for peer_info in &peers_info {
            if let Ok(stream) = handshake::perform(peer_info, info_hash, peer_id) {
                let mut peer = Peer::new(stream);
                let info = info.clone();
                let counter_clone = piece_counter.clone();
                handles.push(thread::spawn(move || {
                    peer_thread(&mut peer, &info, counter_clone)
                }));
            }
        }

        loop {
            for handle in &handles {
                if handle.is_finished() {
                    return Err("CIAO");
                }
            }
        }
    }
}

pub fn get_info(
    peers_info: &Vec<PeerConnectionInfo>,
    peer_id: &str,
    info_hash: &[u8],
) -> Result<Info, &'static str> {
    for peer_info in peers_info {
        if let Ok(stream) = handshake::perform(peer_info, info_hash, peer_id) {
            let mut peer = Peer::new(stream);
            if let Ok(info) = download_info(&mut peer) {
                println!("Info download correctly for the peer {:?}", peer_info);
                return Ok(info);
            }
        }
    }
    Err("No info downloaded")
}
