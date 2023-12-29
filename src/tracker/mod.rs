pub mod peer_endpoint;
mod tcp_tracker;
mod udp_tracker;

use crate::common::generator::generate_peer_id;

use url::Url;

use self::{
    peer_endpoint::PeerEndpoint, tcp_tracker::TcpTrackerError, udp_tracker::UdpTrackerError,
};

#[derive(Debug)]
pub struct Tracker {}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum TrackerError {
    #[error(transparent)]
    TcpTracker(#[from] TcpTrackerError),
    #[error(transparent)]
    UdpTracker(#[from] UdpTrackerError),
    #[error("Protocol {0} not supported")]
    ProtocolNotSupported(String),
}

pub fn get_peers_by_tracker(
    tracker: &Url,
    info_hash: &[u8],
) -> Result<Vec<PeerEndpoint>, TrackerError> {
    let peer_id = &generate_peer_id();

    // A tracker response struct with a get_peers method bound to a trait could be useful here ?
    let response = match tracker.scheme() {
        "http" => tcp_tracker::call(info_hash, peer_id, tracker)?,
        "udp" => udp_tracker::call(info_hash, peer_id, tracker)?,
        scheme => Err(TrackerError::ProtocolNotSupported(scheme.to_string()))?,
    };

    Ok(PeerEndpoint::from_bytes(&response))
}
