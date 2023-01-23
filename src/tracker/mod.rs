pub mod tcp_tracker;
pub mod udp_tracker;

use std::fs::File;
use std::io::prelude::*;
use std::str;
use std::sync::mpsc::Sender;

use crate::common::generator::generate_peer_id;

#[derive(Debug)]
pub struct Tracker {
    pub interval: usize,
    pub peers: Vec<PeerConnectionInfo>,
}

#[derive(Debug, Clone)]
pub struct PeerConnectionInfo {
    pub ip: String,
    pub port: u16,
}

impl PeerConnectionInfo {
    pub fn get_peer_endpoint(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

fn read_file() -> Vec<u8> {
    let mut file = File::open("tracker_list.txt").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    contents
}

impl Tracker {
    pub fn find_peers(info_hash: &[u8], peer_info_sender: Sender<Vec<PeerConnectionInfo>>) {
        let peer_id = &generate_peer_id();
        let trackers_hostname = find_trackers();

        for tracker_hostname in trackers_hostname {
            let tracker = match &tracker_hostname[0..3] {
                "htt" => tcp_tracker::get_tracker(info_hash, peer_id, &tracker_hostname),
                "udp" => udp_tracker::get_tracker(info_hash, peer_id, &tracker_hostname),
                _ => Err("Protocol not supported"),
            };

            if let Ok(tracker) = tracker {
                println!("Found {:?} peers", tracker.peers.len());
                peer_info_sender
                    .send(tracker.get_peers_info().to_vec())
                    .unwrap();
            }
        }
    }

    pub fn get_peers_info(&self) -> &Vec<PeerConnectionInfo> {
        &self.peers
    }

    pub fn peers_info_from_bytes(bytes: &[u8]) -> Vec<PeerConnectionInfo> {
        let mut peers_info = Vec::new();

        for chunk in bytes.chunks_exact(6) {
            let ip = format!(
                "{:?}.{:?}.{:?}.{:?}",
                chunk[0], chunk[1], chunk[2], chunk[3]
            );
            let port = ((chunk[4] as u16) << 8) | (chunk[5] as u16);

            peers_info.push(PeerConnectionInfo { ip, port })
        }

        peers_info
    }
}

fn find_trackers() -> Vec<String> {
    let tracker_list = str::from_utf8(read_file().as_slice()).unwrap().to_owned();
    tracker_list
        .split('\n')
        .map(|tracker| tracker.to_owned())
        .collect::<Vec<String>>()
}

mod test {}
