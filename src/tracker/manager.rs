use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use crate::common::thread_pool::{ThreadPool};
use crate::peer::manager::peer_thread;
use crate::peer::stream::StreamInterface;
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::tracker::PeerConnectionInfo;

use rayon::prelude::*;

pub fn thread_evo(
    peers_info_receiver: Receiver<Vec<PeerConnectionInfo>>,
    info_hash: &[u8],
) -> Result<(), &'static str> {
    let piece_counter = Arc::new(Mutex::new(0));
    let info_mutex: Arc<Mutex<Option<Info>>> = Arc::new(Mutex::new(None));

    loop {
        // Once implemented the thread pool here we could put everythin in parallel in this for loop.
        // The thread pool will be responsible for executing with the right resources the tasks.
        let endpoints: Vec<String> = peers_info_receiver
            .recv()
            .unwrap()
            .into_par_iter()
            .map(|peer_conn_info| peer_conn_info.get_peer_endpoint())
            .filter(|endpoint| connect(endpoint).is_ok())
            .collect();

        println!("{:?}", endpoints);

        let peer_pool = ThreadPool::new(5);

        for endpoint in endpoints {
            if let Ok(stream) = connect(&endpoint) {
                let mut peer = Peer::new(StreamInterface::Tcp(stream), info_hash);

                let info_mutex_clone = info_mutex.clone();
                let counter_clone = piece_counter.clone();

                peer_pool.execute(move || {
                    peer_thread(&mut peer, info_mutex_clone, counter_clone)
                });
            } else {
                println!("Error during peer connection");
            }
        }
    }
}

fn connect(endpoint: &str) -> Result<TcpStream, &'static str> {
    let server: SocketAddr = endpoint.parse().expect("Unable to parse socket address");
    let connect_timeout = Duration::from_secs(3);
    TcpStream::connect_timeout(&server, connect_timeout).map_err(|_| "Error")
}
