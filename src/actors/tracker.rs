use actix::prelude::*;
use url::Url;

use crate::{
    actors::messages::PeerFound,
    tracker::{self, peer_endpoint::PeerEndpoint},
};

use super::messages::TorrentRegistered;

pub struct TrackerActor {
    pub url: Url,
}

// Provide Actor implementation for our actor
impl Actor for TrackerActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        println!("Tracker Actor is alive");
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        println!("Tracker Actor is stopped");
    }
}

impl Handler<TorrentRegistered> for TrackerActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: TorrentRegistered, _ctx: &mut Context<Self>) -> Self::Result {
        println!("New torrent found with info hash: {:?}", msg.info_hash);

        let result = tracker::get_peers_by_tracker(&self.url, &msg.info_hash);

        if let Ok(peers) = result {
            for peer in peers {
                if PeerEndpoint::is_reachable(&peer.endpoint()) {
                    msg.torrent_actor_addr.do_send(PeerFound { peer });
                }
            }
        }

        Ok(true)
    }
}
