use actix::prelude::*;
use url::Url;

use crate::{
    actors::messages::PeerFound,
    tracker::{self, peer_endpoint::PeerEndpoint},
};

use super::{messages::TorrentRegistered, tracker::TrackerActor};

pub struct TrackersInterfaceActor {
    trackers: Vec<Addr<TrackerActor>>,
}

impl TrackersInterfaceActor {
    pub fn new() -> TrackersInterfaceActor {
        // Start MyActor in current thread
        let addr1 = TrackerActor {
            url: Url::parse("udp://93.158.213.92:1337/announce")
                .ok()
                .unwrap(),
        }
        .start();
        let addr2 = TrackerActor {
            url: Url::parse("udp://102.223.180.235:6969/announce")
                .ok()
                .unwrap(),
        }
        .start();
        let addr3 = TrackerActor {
            url: Url::parse("udp://23.134.88.6:1337/announce").ok().unwrap(),
        }
        .start();
        let addr4 = TrackerActor {
            url: Url::parse("udp://193.189.100.187:6969/announce")
                .ok()
                .unwrap(),
        }
        .start();
        let addr5 = TrackerActor {
            url: Url::parse("udp://185.243.218.213:80/announce")
                .ok()
                .unwrap(),
        }
        .start();
        let addr6 = TrackerActor {
            url: Url::parse("udp://91.216.110.52:451/announce").ok().unwrap(),
        }
        .start();

        TrackersInterfaceActor {
            trackers: vec![
                addr1.clone(),
                addr2.clone(),
                addr3.clone(),
                addr4.clone(),
                addr5.clone(),
                addr6.clone(),
            ],
        }
    }
}

// Provide Actor implementation for our actor
impl Actor for TrackersInterfaceActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Tracker Actor is alive");
    }

    fn stopped(&mut self, ctx: &mut Context<Self>) {
        println!("Tracker Actor is stopped");
    }
}

impl Handler<TorrentRegistered> for TrackersInterfaceActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: TorrentRegistered, _ctx: &mut Context<Self>) -> Self::Result {
        println!("New torrent found with info hash: {:?}", msg.info_hash);

        for actor in self.trackers.iter() {
            let _ = actor.try_send(msg.clone());
        }

        Ok(true)
    }
}
