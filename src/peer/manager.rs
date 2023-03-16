use log::info;

use crate::messages::{new_handshake, new_interested};
use crate::peer::Peer;
use crate::torrent::info::Info;
use crate::torrent::writer::write_piece;

use super::download::Downloadable;
use super::piece_pool::PiecePool;
use super::stream::StreamInterface;

pub fn get_info(info_hash: &[u8], endpoint: String) -> Result<Info, &'static str> {
    let stream = StreamInterface::connect(&endpoint, false)?;
    let mut peer = Peer::new(stream, info_hash);

    init_peer(&mut peer)?;
    let info = Downloadable::Info.download(&mut peer)?;

    Info::from_bytes(info).map_err(|_| "Error getting info data")
}

struct Context {
    pub endpoint: String,
    pub peer_id: String,
    pub piece_idx: usize,
}

pub fn peer_thread(endpoint: String, info: Info, pool: PiecePool) -> Result<(), &'static str> {
    // Avoid establish a tcp connection if there are no pieces to download
    if pool.is_emtpy() {
        info!("No more pieces to download");
        return Ok(());
    }

    let stream = StreamInterface::connect(&endpoint, false)?;
    let mut peer = Peer::new(stream, &info.compute_info_hash());

    init_peer(&mut peer)?;

    loop {
        let piece_idx = pool.pop().ok_or_else(|| {
            info!("No more pieces to download");
            "No more pieces"
        })?;

        let ctx = Context {
            peer_id: peer.get_peer_id(),
            piece_idx,
            endpoint: endpoint.to_owned(),
        };
        track_progress(PieceEventType::StartDownload(), &ctx);

        let block =
            Downloadable::Block((info.get_piece_length(), piece_idx, info.get_total_length()));
        let piece = match block.download(&mut peer) {
            Ok(piece) => piece,
            Err(err) => {
                pool.insert(piece_idx);
                return Err(err);
            }
        };

        track_progress(PieceEventType::CompleteDownload(), &ctx);
        if info.verify_piece(&piece, piece_idx) {
            write_piece(
                piece,
                piece_idx,
                info.get_piece_length(),
                info.get_files().unwrap(),
            );
            track_progress(PieceEventType::CompleteWrite(), &ctx);
        } else {
            pool.insert(piece_idx);
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

pub enum PieceEventType {
    StartDownload(),
    CompleteDownload(),
    CompleteWrite(),
}

fn track_progress(event_type: PieceEventType, ctx: &Context) {
    match event_type {
        PieceEventType::StartDownload() => info!(
            "Start download by {:?} piece {:?} from peer {:?}",
            ctx.peer_id, ctx.piece_idx, ctx.endpoint
        ),
        PieceEventType::CompleteDownload() => info!(
            "Completed downloadby {:?} for piece {:?} from peer {:?}",
            ctx.peer_id, ctx.piece_idx, ctx.endpoint
        ),
        PieceEventType::CompleteWrite() => info!(
            "Completed write by {:?} to filesystem for piece {:?} from peer {:?}",
            ctx.peer_id, ctx.piece_idx, ctx.endpoint
        ),
    }
}
