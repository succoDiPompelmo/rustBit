use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{error, info};

use crate::messages::{new_handshake, new_interested};
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::torrent::writer::write_piece;

use super::download::{download, Downloadable};
use super::stream::StreamInterface;

pub fn get_info(info_hash: &[u8], endpoint: String) -> Result<Info, &'static str> {
    let stream = connect(&endpoint)?;
    let mut peer = Peer::new(StreamInterface::Tcp(stream), info_hash);

    init_peer(&mut peer)?;
    Info::from_bytes(download(&mut peer, Downloadable::Info)?)
}

pub fn peer_thread(
    endpoint: String,
    info: Info,
    lock_counter: Arc<Mutex<usize>>,
) -> Result<(), &'static str> {
    let stream = connect(&endpoint)?;
    let mut peer = Peer::new(StreamInterface::Tcp(stream), &info.compute_info_hash());

    init_peer(&mut peer)?;

    let mut piece_idx = 0;

// Use a queue to manage the insertion/retry of piece download
    loop {
        if let Ok(mut counter) = lock_counter.lock() {
            piece_idx = *counter;
            *counter += 1;
        }

        info!("Start download by {:?} piece {:?} from peer {:?}", peer.get_peer_id(), piece_idx, endpoint);

        let piece = download(
            &mut peer,
            Downloadable::Block((info.get_piece_length(), piece_idx, info.get_total_length())),
        )?;

        info!("Completed downloadby {:?} for piece {:?} from peer {:?}", peer.get_peer_id(), piece_idx, endpoint);

        if info.verify_piece(&piece, piece_idx) {
            write_piece(
                piece,
                piece_idx,
                info.get_piece_length(),
                info.get_files().unwrap(),
            );
            info!("Completed write by {:?} to filesystem for piece {:?} from peer {:?}", peer.get_peer_id(), piece_idx, endpoint);

        } else {
            return Err("Error during piece verification");
        }
    }
}

fn init_peer(peer: &mut Peer) -> Result<(), &'static str> {
    peer.send_message(new_handshake(&peer.get_info_hash(), &peer.get_peer_id()));
    peer.read_message()
        .map_or((), |msg| peer.apply_message(&msg));

    if !peer.is_active() {
        return Err("Handshake failed");
    }

    peer.send_message(new_interested());
    peer.send_metadata_handshake_request()?;

    for _ in 0..10 {
        peer.read_message()
            .map_or((), |msg| peer.apply_message(&msg));

        if peer.is_ready() {
            return Ok(());
        }
    }
    Err("Peer not ready")
}

fn connect(endpoint: &str) -> Result<TcpStream, &'static str> {
    let server: SocketAddr = endpoint.parse().expect("Unable to parse socket address");
    let connect_timeout = Duration::from_secs(1);
    TcpStream::connect_timeout(&server, connect_timeout).map_err(|_| "Connection error")
}

// #[cfg(test)]
// mod test {
//     use crate::{peer::{stream::StreamInterface, download}, common::mock_stream::MockStream};

//     use super::*;

//     #[test]
//     fn test_peer_thread() {

//         let piece_counter = Arc::new(Mutex::new(0));
//         let info_mutex: Arc<Mutex<Option<Info>>> = Arc::new(Mutex::new(None));

//         let info_hash = "aaaaaaaaaaaaaaaaaaaa".as_bytes();
//         let peer_id = "bbbbbbbbbbbbbbbbbbbb";

//         let mut s = MockStream::new();

//         // HANDSHAKE
//         s.push_bytes_to_read(&new_handshake(info_hash, peer_id).as_bytes());

//         // UNCHOKE MESSAGE
//         s.push_bytes_to_read([0, 0, 0, 1, 1].as_slice());
//         // EXTENSION DATA MESSAGE
//         let dictionary = "d8:msg_typei1ee".as_bytes().to_vec();
//         let info = "d6:pieces20:aaaaaaaaaaaaaaaaaaaa12:piece lengthi12e4:name1:B6:lengthi12ee".as_bytes().to_vec();
//         let message = [vec![0, 0, 0, 90, 20, 2], dictionary, info].concat();
//         s.push_bytes_to_read(&message);

//         // PIECE MESSAGE
//         s.push_bytes_to_read([0, 0, 0, 11, 6, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2].as_slice());

//         let e = StreamInterface::Mocked(s.clone());
//         let mut peer = Peer::new(e, info_hash);

//         peer.set_metadata_size(download::INFO_PIECE_SIZE);
//         peer.add_extension("ut_metadata".to_owned(), 1);

//         peer_thread(&mut peer, info_mutex, piece_counter);
//     }
// }
