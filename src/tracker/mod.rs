pub mod tcp_tracker;
pub mod udp_tracker;

use std::fs::File;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream};
use std::str;
use std::time::Duration;

use crate::common::generator::generate_peer_id;

use sqlx::postgres::PgConnection;
use sqlx::Connection;

#[derive(Debug)]
pub struct Tracker {}

#[derive(Debug, Clone)]
pub struct PeerConnectionInfo {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrackedPeer {
    endpoint: String,
}

impl TrackedPeer {
    pub fn endpoint(&self) -> String {
        self.endpoint.to_string()
    }

    pub fn is_reachable(&self) -> bool {
        let server: SocketAddr = self.endpoint().parse().unwrap();
        let connect_timeout = Duration::from_millis(300);
        TcpStream::connect_timeout(&server, connect_timeout).is_ok()
    }
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
    pub async fn get_tracked_peers(info_hash: Vec<u8>) -> Option<Vec<TrackedPeer>> {
        let mut db = PgConnection::connect("postgres://postgres:password@localhost/rust_bit")
            .await
            .unwrap();

        let result = sqlx::query_as::<_, TrackedPeer>(
            "SELECT endpoint FROM tracked_peers WHERE info_hash = $1",
        )
        .bind(info_hash)
        .fetch_all(&mut db)
        .await;

        result.ok()
    }

    pub async fn find_peers(info_hash: Vec<u8>) {
        let mut db = PgConnection::connect("postgres://postgres:password@localhost/rust_bit")
            .await
            .unwrap();

        loop {
            let peer_id = &generate_peer_id();
            let trackers_hostname = list_trackers();

            for tracker_hostname in trackers_hostname {
                let tracker = match &tracker_hostname[0..3] {
                    "htt" => tcp_tracker::get_tracker(&info_hash, peer_id, &tracker_hostname),
                    "udp" => udp_tracker::get_tracker(&info_hash, peer_id, &tracker_hostname),
                    _ => Err("Protocol not supported"),
                };
    
                if let Ok(peers) = tracker {
                    for peer in &peers {
                        let query = "INSERT INTO tracked_peers
                        (info_hash, endpoint)
                        VALUES ($1, $2) ON CONFLICT (info_hash, endpoint) DO NOTHING";
    
                        sqlx::query(query)
                            .bind(info_hash.to_vec())
                            .bind(peer.get_peer_endpoint())
                            .execute(&mut db)
                            .await
                            .unwrap();
                    }
                }
            }
        }
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

fn list_trackers() -> Vec<String> {
    let tracker_list = str::from_utf8(read_file().as_slice()).unwrap().to_owned();
    tracker_list
        .split('\n')
        .map(|tracker| tracker.to_owned())
        .collect::<Vec<String>>()
}

mod test {}
