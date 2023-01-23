use std::sync::{Arc, Mutex};

use crate::messages::{new_handshake, new_interested};
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::torrent::writer::write_piece;

use super::download::{download, Downloadable};

fn starup_peer(peer: &mut Peer) -> Result<(), &'static str> {
    peer.send_message(new_handshake(&peer.get_info_hash(), &peer.get_peer_id()));
    peer.read_message()
        .map_or((), |msg| peer.apply_message(&msg));

    if !peer.is_active() {
        return Err("Handshake failed");
    }

    peer.send_message(new_interested());
    peer.send_metadata_handshake_request();

    for _ in 0..10 {
        peer.read_message()
            .map_or((), |msg| peer.apply_message(&msg));

        if peer.is_ready() {
            return Ok(());
        }
    }
    Err("Peer not ready")
}

fn prepare_info(
    peer: &mut Peer,
    info_arc: &Arc<Mutex<Option<Info>>>,
) -> Result<(usize, usize), &'static str> {
    if let Ok(mutex_info) = info_arc.lock() {
        if (*mutex_info).is_some() {
            let info = mutex_info.as_ref().unwrap();
            return Ok((info.get_piece_length(), info.get_total_length()));
        }
    }

    let info_bytes = download(peer, Downloadable::Info)?;
    let info = Info::from_bytes(info_bytes)?;

    if let Ok(mut mutex_info) = info_arc.lock() {
        let result = (info.get_piece_length(), info.get_total_length());
        *mutex_info = Some(info);
        Ok(result)
    } else {
        Err("Error during info lock")
    }
}

pub fn get_info(peer: &mut Peer) -> Result<Info, &'static str> {
    starup_peer(peer)?;
    let info_bytes = download(peer, Downloadable::Info)?;
    return Info::from_bytes(info_bytes)
}

pub fn peer_thread(
    peer: &mut Peer,
    info: Info,
    lock_counter: Arc<Mutex<usize>>,
) -> Result<(), &'static str> {
    starup_peer(peer)?;

    let mut piece_idx = 0;

    loop {
        if let Ok(mut counter) = lock_counter.lock() {
            piece_idx = *counter;
            *counter += 1;
        }

        let piece = download(
            peer,
            Downloadable::Block((info.get_piece_length(), piece_idx, info.get_total_length())),
        )?;

        if info.verify_piece(&piece, piece_idx) {
            write_piece(
                piece,
                piece_idx,
                info.get_piece_length(),
                info.get_files().unwrap(),
            )
        } else {
            return Err("Error during piece verification");
        }
    }
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
