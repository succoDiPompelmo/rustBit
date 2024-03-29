use actix::prelude::*;
use url::Url;

use super::{messages::TorrentRegistered, tracker::TrackerActor};

pub struct TrackersInterfaceActor {
    trackers: Vec<Addr<TrackerActor>>,
}

impl TrackersInterfaceActor {
    pub fn new() -> TrackersInterfaceActor {
        let urls = vec![
            "udp://93.158.213.92:1337/announce",
            "udp://102.223.180.235:6969/announce",
            "udp://23.134.88.6:1337/announce",
            "udp://193.189.100.187:6969/announce",
            "udp://185.243.218.213:80/announce",
            "udp://91.216.110.52:451/announce",
            "udp://208.83.20.20:6969/announce",
            "udp://23.157.120.14:6969/announce",
            "udp://156.234.201.18:80/announce",
            "udp://185.102.219.163:6969/announce",
            "udp://185.50.159.149:6969/announce",
            "udp://209.141.59.16:6969/announce",
            "udp://38.7.201.142:6969/announce",
            "udp://73.170.204.100:6969/announce",
            "udp://176.31.250.174:6969/announce",
            "udp://82.156.24.219:6969/announce",
            "udp://83.102.180.21:80/announce",
            "udp://185.230.4.150:1337/announce",
        ];
        let mut trackers = vec![];

        for url in urls {
            let tracker = TrackerActor {
                url: Url::parse(url).ok().unwrap(),
            }
            .start();

            trackers.push(tracker);
        }

        TrackersInterfaceActor { trackers }
    }
}

// Provide Actor implementation for our actor
impl Actor for TrackersInterfaceActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {}

    fn stopped(&mut self, _ctx: &mut Context<Self>) {}
}

impl Handler<TorrentRegistered> for TrackersInterfaceActor {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: TorrentRegistered, _ctx: &mut Context<Self>) -> Self::Result {
        for actor in self.trackers.iter() {
            let _ = actor.try_send(msg.clone());
        }

        Ok(true)
    }
}
