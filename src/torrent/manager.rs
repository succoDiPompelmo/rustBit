use std::{fs::File, io::Read};

use log::info;

use crate::{
    common::thread_pool::ThreadPool,
    peer::{
        manager::{get_info, peer_thread},
        piece_pool::PiecePool,
    },
    torrent::Torrent,
    tracker::Tracker,
};

use super::info::Info;

pub struct TorrentManager {}

impl TorrentManager {
    pub async fn init(torrent: Torrent) {
        let info = retrieve_info(&torrent.get_info_hash()).await;

        let piece_count = (0..info.get_total_length())
            .step_by(info.get_piece_length())
            .len();

        let piece_pool = PiecePool::new(piece_count);

        let pool = ThreadPool::new(1);
        let writers = ThreadPool::new(1);

        loop {
            let endpoints = Tracker::find_reachable_peers(&torrent.get_info_hash()).await;
            for endpoint in endpoints {
                let safe_piece_pool_clone = piece_pool.clone();
                let info_clone = info.clone();
                let writers_clone = writers.clone();
                pool.execute(move || {
                    peer_thread(endpoint, info_clone, safe_piece_pool_clone, writers_clone)?;
                    Ok(())
                });
            }
        }
    }
}

pub async fn retrieve_info(info_hash: &[u8]) -> Info {
    let filename = urlencoding::encode_binary(info_hash).into_owned();
    let file_path = format!("./downloads/{filename}");

    if let Ok(mut info_file) = File::open(&file_path) {
        info!("Torrent info from file: {:?}", filename);
        let mut info_buffer = "".to_string();
        info_file.read_to_string(&mut info_buffer).unwrap();

        return serde_json::from_str(&info_buffer).unwrap();
    }

    loop {
        let endpoints = Tracker::find_reachable_peers(info_hash).await;
        for endpoint in endpoints {
            match get_info(info_hash, endpoint) {
                Ok(info) => {
                    info!("Torrent info from peer");
                    serde_json::to_writer(&File::create(&file_path).unwrap(), &info).unwrap();
                    return info;
                }
                Err(err) => {
                    log::error!("Info error: {}", err.to_string());
                }
            }
        }
    }
}
