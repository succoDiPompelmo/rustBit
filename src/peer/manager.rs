use log::info;

use crate::messages::{new_handshake, new_interested};
use crate::peer::Peer;
use crate::torrent::info::{Info, InfoError};

use super::download::{Downloadable, DownloadableError};
use super::stream::{StreamError, StreamInterface};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum PeerManagerError {
    #[error("Handshake error with peer")]
    Handshake(),
    #[error("Handshake metadata error with peer")]
    HandshakeMetadata(),
    #[error("Peer not ready")]
    PeerNotReady(),
    #[error("Unsuccessfull piece download: {0}")]
    PieceDownloadFailure(String),
    #[error("Unsuccessfull piece verification")]
    PieceVerificationFailure(),
    #[error(transparent)]
    Stream(#[from] StreamError),
    #[error(transparent)]
    Info(#[from] InfoError),
    #[error(transparent)]
    Download(#[from] DownloadableError),
}

pub fn get_info(info_hash: &[u8], endpoint: String) -> Result<Info, PeerManagerError> {
    let stream = StreamInterface::connect(&endpoint, false)?;
    let mut peer = Peer::new(stream, info_hash);

    init_peer(&mut peer)?;
    let info = Downloadable::Info.download(&mut peer)?;

    let info = Info::from_bytes(info)?;
    Ok(info)
}

struct Context {
    pub endpoint: String,
    pub peer_id: String,
    pub piece_idx: usize,
}

impl Context {
    fn new(peer_id: String, piece_idx: usize, endpoint: String) -> Self {
        Context {
            peer_id,
            piece_idx,
            endpoint,
        }
    }
}

pub fn download(
    endpoint: String,
    info: &Info,
    piece_idx: usize,
) -> Result<Vec<u8>, PeerManagerError> {
    let stream = StreamInterface::connect(&endpoint, false)?;
    let mut peer = Peer::new(stream, &info.compute_info_hash());
    init_peer(&mut peer)?;

    let piece_length = info.get_piece_length();

    let ctx = Context::new(peer.get_peer_id(), piece_idx, endpoint.to_owned());

    track_progress(PieceEventType::StartDownload(), &ctx);

    let block = Downloadable::Block((piece_length, piece_idx, info.get_total_length()));
    let piece = block
        .download(&mut peer)
        .map_err(|err| PeerManagerError::PieceDownloadFailure(err.to_string()))?;
    track_progress(PieceEventType::CompleteDownload(), &ctx);

    if !info.verify_piece(&piece, piece_idx) {
        return Err(PeerManagerError::PieceVerificationFailure());
    }

    Ok(piece)
}

fn init_peer(peer: &mut Peer) -> Result<(), PeerManagerError> {
    peer.send_message(new_handshake(&peer.get_info_hash(), &peer.get_peer_id()));
    peer.read_message()
        .map_or((), |msg| peer.apply_message(&msg));

    if !peer.is_active() {
        return Err(PeerManagerError::Handshake());
    }

    peer.send_message(new_interested());
    peer.send_metadata_handshake_request()
        .map_err(|_| PeerManagerError::HandshakeMetadata())?;

    for _ in 0..10 {
        let _ = peer
            .read_message()
            .map_or((), |msg| peer.apply_message(&msg));

        if peer.is_ready() {
            return Ok(());
        }
    }
    Err(PeerManagerError::PeerNotReady())
}

pub enum PieceEventType {
    StartDownload(),
    CompleteDownload(),
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
    }
}
