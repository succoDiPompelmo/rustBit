use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use crate::peer::manager::{download_info, peer_thread};
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
        let endpoints: Vec<String> = peers_info_receiver
            .recv()
            .unwrap()
            .into_par_iter()
            .map(|peer_conn_info| peer_conn_info.get_peer_endpoint())
            .filter(|endpoint| connect(endpoint).is_ok())
            .collect();

        println!("{:?}", endpoints);

        for endpoint in endpoints {

            let mut peer = match connect(&endpoint) {
                Ok(stream) => Peer::new(Some(stream)),
                Err(err) => {
                    println!("Error during peer connection: {:?}", err);
                    continue
                },
            };

            if let Err(err) = peer.handshake(info_hash, peer_id) {
                println!("Error during peer handshake {:?}", err);
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

fn connect(endpoint: &str) -> Result<TcpStream, &'static str> {
    let server: SocketAddr = endpoint.parse().expect("Unable to parse socket address");
    let connect_timeout = Duration::from_secs(3);
    TcpStream::connect_timeout(&server, connect_timeout).map_err(|_| "Error")
}
