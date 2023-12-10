pub mod peer_endpoint;
mod tcp_tracker;
mod tracked_peer;
mod udp_tracker;

use std::str;

use crate::common::file::read_file;
use crate::common::generator::generate_peer_id;

use rayon::prelude::*;
use url::Url;

use self::{
    peer_endpoint::PeerEndpoint,
    tcp_tracker::TcpTrackerError,
    tracked_peer::{all_endpoints_by_hash, insert_tracked_peers},
    udp_tracker::UdpTrackerError,
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

impl Tracker {
    pub async fn find_reachable_peers(info_hash: &[u8]) -> Vec<String> {
        all_endpoints_by_hash(info_hash.to_vec())
            .await
            .into_par_iter()
            .filter(|el| PeerEndpoint::is_reachable(el))
            .collect::<Vec<String>>()
    }

    pub async fn find_peers(info_hash: Vec<u8>) {
        loop {
            for tracker_url in list_trackers() {
                match get_peers_by_tracker(&tracker_url, &info_hash) {
                    Ok(peers) => insert_tracked_peers(peers, &info_hash).await,
                    Err(err) => log::error!("Tracker: {}", err.to_string()),
                }
            }
        }
    }
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

fn list_trackers() -> Vec<Url> {
    str::from_utf8(read_file("tracker_list.txt").as_slice())
        .unwrap_or("")
        .to_owned()
        .split('\n')
        .filter_map(|tracker| Url::parse(tracker).ok())
        .collect::<Vec<Url>>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_list_trackers() {
        let trackers = list_trackers();

        assert!(!trackers.is_empty());

        let first_tracker = trackers.first().unwrap();

        assert_eq!(first_tracker.scheme(), "udp");
        assert_eq!(first_tracker.host_str().unwrap(), "107.150.14.110");
        assert_eq!(first_tracker.as_str(), "udp://107.150.14.110:6969/announce");
    }
}
