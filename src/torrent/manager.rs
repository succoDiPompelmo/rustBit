use std::{
    net::{SocketAddr, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration, fs::File, io::Read,
};

use rayon::prelude::*;

use crate::{
    common::thread_pool::ThreadPool,
    peer::{manager::{peer_thread, get_info}, stream::StreamInterface, Peer},
    torrent::Torrent,
    tracker::{PeerConnectionInfo, Tracker},
};

use super::info::Info;

pub struct TorrentManager {}

impl TorrentManager {
    pub fn init(torrent: &mut Torrent) {
        let (tx_tracker, rx_tracker): (
            Sender<Vec<PeerConnectionInfo>>,
            Receiver<Vec<PeerConnectionInfo>>,
        ) = mpsc::channel();

        let (tx_peer, rx_peer): (
            Sender<Vec<PeerConnectionInfo>>,
            Receiver<Vec<PeerConnectionInfo>>,
        ) = mpsc::channel();

        let info_hash = torrent.get_info_hash();
        thread::spawn(move || Tracker::find_peers(&info_hash, tx_tracker));

        let filename = urlencoding::encode_binary(&torrent.get_info_hash()).into_owned();

        let info = if let Ok(mut file) = File::open(&filename) {
            let mut info_string = "".to_string();
            file.read_to_string(&mut info_string).unwrap();

            let info: Info = serde_json::from_str(&info_string).unwrap();
            info
        } else {
            let info = loop {
                let peers_connection_info = rx_tracker.recv().unwrap();
    
                if let Ok(info) = info_thread(peers_connection_info, &torrent.get_info_hash()) {
                    serde_json::to_writer(&File::create(&filename).unwrap(), &info).unwrap();
                    break info
                } else {
                    println!("INFO NOT FOUND")
                }
            };
            info
        };

        thread::spawn(move || piece_thread(rx_peer, info));

        loop {
            let peers_connection_info = rx_tracker.recv().unwrap();
            tx_peer.send(peers_connection_info).unwrap();
        }
    }
}

pub fn info_thread(peers_connection_info: Vec<PeerConnectionInfo>,
    info_hash: &[u8],) -> Result<Info, &'static str> {    
        let endpoints: Vec<String> = peers_connection_info.to_vec()
            .into_par_iter()
            .map(|peer_conn_info| peer_conn_info.get_peer_endpoint())
            .filter(|endpoint| connect(endpoint).is_ok())
            .collect();

        println!("{:?}", endpoints);

        for endpoint in endpoints {
            if let Ok(stream) = connect(&endpoint) {
                let mut peer = Peer::new(StreamInterface::Tcp(stream), info_hash);

                if let Ok(info) = get_info(&mut peer) {
                    return Ok(info)
                }
            } else {
                println!("Error during peer connection");
            }
        }

        Err("A")
}

pub fn piece_thread(
    peers_info_receiver: Receiver<Vec<PeerConnectionInfo>>,
    info: Info,
) -> Result<(), &'static str> {
    let piece_counter = Arc::new(Mutex::new(0));
    let peer_pool = ThreadPool::new(1);

    let info_hash = info.compute_info_hash();

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

        for endpoint in endpoints {
            if let Ok(stream) = connect(&endpoint) {
                let mut peer = Peer::new(StreamInterface::Tcp(stream), &info_hash);
                let counter_clone = piece_counter.clone();

                let info_clone = info.clone();
                peer_pool.execute(move || peer_thread(&mut peer, info_clone, counter_clone));
            } else {
                println!("Error during peer connection");
            }
        }
    }
}

fn connect(endpoint: &str) -> Result<TcpStream, &'static str> {
    let server: SocketAddr = endpoint.parse().expect("Unable to parse socket address");
    let connect_timeout = Duration::from_secs(1);
    TcpStream::connect_timeout(&server, connect_timeout).map_err(|_| "Error")
}
