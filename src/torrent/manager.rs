use std::{
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

use rayon::prelude::*;

use log::{info};

use crate::{
    common::thread_pool::ThreadPool,
    peer::manager::{get_info, peer_thread},
    torrent::Torrent,
    tracker::Tracker,
};

use super::info::Info;

pub struct TorrentManager {}

impl TorrentManager {
    pub async fn init(torrent: Torrent) {
        let info = retrieve_info(&torrent.get_info_hash()).await;

        let piece_counter = Arc::new(Mutex::new(0));
        let pool = ThreadPool::new(3);

        loop {
            let endpoints = find_reachable_peers(&torrent.get_info_hash()).await;
            for endpoint in endpoints {
                let counter_clone = piece_counter.clone();
                let info_clone = info.clone();
                pool.execute(move || peer_thread(endpoint, info_clone, counter_clone));
            }
        }
    }
}

async fn retrieve_info(info_hash: &[u8]) -> Info {
    let filename = urlencoding::encode_binary(info_hash).into_owned();

    if let Ok(mut info_file) = File::open(&filename) {
        info!("Torrent info from file: {:?}", filename);
        let mut info_buffer = "".to_string();
        info_file.read_to_string(&mut info_buffer).unwrap();

        return serde_json::from_str(&info_buffer).unwrap();
    }

    loop {
        let endpoints = find_reachable_peers(info_hash).await;
        for endpoint in endpoints {
            if let Ok(info) = get_info(info_hash, endpoint) {
                info!("Torrent info from peer");
                serde_json::to_writer(&File::create(&filename).unwrap(), &info).unwrap();
                return info;
            }
        }
    }
}

async fn find_reachable_peers(info_hash: &[u8]) -> Vec<String> {
    if let Some(peers) = Tracker::get_tracked_peers(info_hash.to_vec()).await {
        peers
            .to_vec()
            .into_par_iter()
            .filter(|el| el.is_reachable())
            .map(|e| e.endpoint())
            .collect()
    } else {
        vec![]
    }
}
