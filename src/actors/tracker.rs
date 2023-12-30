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

    fn started(&mut self, _ctx: &mut Context<Self>) {}

    fn stopped(&mut self, _ctx: &mut Context<Self>) {}
}

impl Handler<TorrentRegistered> for TrackerActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: TorrentRegistered, ctx: &mut Context<Self>) -> Self::Result {
        let url = self.url.clone();

        async move {
            let result = tracker::get_peers_by_tracker(&url, &msg.info_hash).await;

            if let Ok(peers) = result {
                for peer in peers {
                    if PeerEndpoint::is_reachable(&peer.endpoint()) {
                        msg.torrent_actor_addr.do_send(PeerFound { peer });
                    }
                }
            }
        }
        .into_actor(self)
        .spawn(ctx);

        Ok(true)
    }
}
