use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use crate::peer::manager::peer_thread_evp;
use crate::peer::stream::StreamInterface;
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::tracker::PeerConnectionInfo;

use rayon::prelude::*;

pub fn thread_evo(
    peers_info_receiver: Receiver<Vec<PeerConnectionInfo>>,
    info_hash: &[u8],
) -> Result<(), &'static str> {
    let mut handles = vec![];
    let piece_counter = Arc::new(Mutex::new(0));
    let info_mutex: Arc<Mutex<Option<Info>>> = Arc::new(Mutex::new(None));

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
            match connect(&endpoint) {
                Ok(stream) => {
                    let mut peer = Peer::new(StreamInterface::Tcp(stream), info_hash);
                    let info_mutex_clone = info_mutex.clone();
                    let counter_clone = piece_counter.clone();
                    handles.push(thread::spawn(move || {
                        peer_thread_evp(&mut peer, info_mutex_clone, counter_clone)
                    }));
                }
                Err(err) => {
                    println!("Error during peer connection: {:?}", err);
                }
            };
        }
    }
}

fn connect(endpoint: &str) -> Result<TcpStream, &'static str> {
    let server: SocketAddr = endpoint.parse().expect("Unable to parse socket address");
    let connect_timeout = Duration::from_secs(3);
    TcpStream::connect_timeout(&server, connect_timeout).map_err(|_| "Error")
}
