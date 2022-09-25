use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs::File;
use std::io::prelude::*;
use std::str;

use crate::peer::Peer;

use crate::bencode::metainfo;

use crate::torrent::torrent::Torrent;
use crate::tracker::manager::download;

use crate::tracker::tcp_tracker;
use crate::tracker::udp_tracker;

use ureq;

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

fn read_file() -> Vec<u8> {
    let mut file = File::open("tracker_list.txt").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    return contents;
}

fn random_peer_id() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();

    return rand_string;
}

impl Tracker {
    pub fn init_tracker(torrent: &mut Torrent) -> Result<Tracker, &'static str> {
        let info_hash = &torrent.get_info_hash();

        let url_encoded_info_hash = urlencoding::encode_binary(info_hash.as_slice()).into_owned();
        let tracker_list = str::from_utf8(read_file().as_slice()).unwrap().to_owned();
        let announce_list = torrent.get_announce_list();

        let trackers = [
            announce_list,
            // tracker_list.split("\n").collect::<Vec<String>>(),
        ]
        .concat();

        for tracker_name in trackers {
            let peer_id = &random_peer_id();

            let mut tracker_result = match &tracker_name[0..3] {
                "htt" => tcp_tracker::get_tracker(info_hash, peer_id, &tracker_name),
                "udp" => udp_tracker::get_tracker(info_hash, peer_id, &tracker_name),
                _ => return Err("Protocol not supported"),
            };

            if let Ok(mut tracker) = tracker_result {
                println!("Found {:?} peers", tracker.peers.len());
                let peers = tracker.get_peers_info();
                download(peers.to_vec(), &peer_id, torrent);
            }
        }
        return Err("No tracker found");
    }

    fn get_peers_info(&self) -> &Vec<PeerConnectionInfo> {
        return &self.peers;
    }

    pub fn peers_info_from_bytes(bytes: &Vec<u8>) -> Vec<PeerConnectionInfo> {
        let mut peers_info = Vec::new();

        for chunk in bytes.chunks_exact(6) {
            let ip = format!(
                "{:?}.{:?}.{:?}.{:?}",
                chunk[0], chunk[1], chunk[2], chunk[3]
            );
            let port = ((chunk[4] as u16) << 8) | (chunk[5] as u16);

            peers_info.push(PeerConnectionInfo { ip, port })
        }

        return peers_info;
    }
}

mod test {}
