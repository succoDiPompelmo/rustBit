use std::{
    fs::File,
    io::Read,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use rayon::prelude::*;

use crate::{
    common::thread_pool::ThreadPool,
    peer::manager::{get_info, peer_thread},
    torrent::Torrent,
    tracker::{PeerConnectionInfo, Tracker},
};

use super::info::Info;

pub struct TorrentManager {}

impl TorrentManager {
    pub fn init(torrent: &mut Torrent) {
        let rx_tracker = spawn_tracker(torrent.get_info_hash());

        let info = retrieve_info(&torrent.get_info_hash(), &rx_tracker);

        let piece_counter = Arc::new(Mutex::new(0));
        let pool = ThreadPool::new(1);

        loop {
            let connections = find_reachable_peers(rx_tracker.recv().unwrap());
            for connection in connections {
                let counter_clone = piece_counter.clone();
                let info_clone = info.clone();
                pool.execute(move || {
                    peer_thread(connection.get_peer_endpoint(), info_clone, counter_clone)
                });
            }
        }
    }
}

fn retrieve_info(info_hash: &[u8], tracker: &Receiver<Vec<PeerConnectionInfo>>) -> Info {
    let filename = urlencoding::encode_binary(info_hash).into_owned();

    if let Ok(mut info_file) = File::open(&filename) {
        let mut info_buffer = "".to_string();
        info_file.read_to_string(&mut info_buffer).unwrap();

        return serde_json::from_str(&info_buffer).unwrap();
    }

    loop {
        let connections = find_reachable_peers(tracker.recv().unwrap());
        for connection in connections {
            if let Ok(info) = get_info(info_hash, connection.get_peer_endpoint()) {
                serde_json::to_writer(&File::create(&filename).unwrap(), &info).unwrap();
                return info;
            }
        }
    }
}

fn spawn_tracker(info_hash: Vec<u8>) -> Receiver<Vec<PeerConnectionInfo>> {
    let (tx, rx): (
        Sender<Vec<PeerConnectionInfo>>,
        Receiver<Vec<PeerConnectionInfo>>,
    ) = mpsc::channel();

    thread::spawn(move || Tracker::find_peers(info_hash, tx));
    rx
}

fn find_reachable_peers(
    peers_connection_info: Vec<PeerConnectionInfo>,
) -> Vec<PeerConnectionInfo> {
    peers_connection_info
        .to_vec()
        .into_par_iter()
        .filter(|el| el.is_reachable())
        .collect()
}
