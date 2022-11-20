use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use crate::peer::peer_manager::{download_info, peer_thread};
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::tracker::PeerConnectionInfo;

use rayon::prelude::*;

pub fn thread_evo(
    peers_info_receiver: Receiver<Vec<PeerConnectionInfo>>,
    peer_id: &str,
    info_hash: &[u8],
) -> Result<(), &'static str> {
    let mut info: Option<Info> = None;
    let mut handles = vec![];
    let piece_counter = Arc::new(Mutex::new(0));

    loop {
        let streams: Vec<Result<TcpStream, &str>> = peers_info_receiver
            .recv()
            .unwrap()
            .into_par_iter()
            // .map(|peer_conn_info| peer_conn_info.get_peer_endpoint())
            .map(connect)
            .filter(|stream_result| stream_result.is_ok())
            .collect();

        println!("{:?}", streams);

        for stream in streams {
            let mut peer = Peer::new(Some(stream?));
            if peer.handshake(info_hash, peer_id).is_err() {
                continue;
            }

            if let Some(info) = &info {
                let info = info.clone();
                let counter_clone = piece_counter.clone();
                handles.push(thread::spawn(move || {
                    peer_thread(&mut peer, &info, counter_clone)
                }));
            } else {
                info = match download_info(&mut peer) {
                    Ok(info) => {
                        println!("INFO IS COMPLETED");
                        Some(info)
                    }
                    Err(_) => None,
                }
            }
        }
    }
}

fn connect(peer_connection_info: PeerConnectionInfo) -> Result<TcpStream, &'static str> {
    let peer_url = format!("{}:{}", peer_connection_info.ip, peer_connection_info.port);
    let server: SocketAddr = peer_url.parse().expect("Unable to parse socket address");

    let connect_timeout = Duration::from_secs(3);
    TcpStream::connect_timeout(&server, connect_timeout).map_err(|_| "Error")
}
