#[cfg(test)]
mod test {

    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread;

    use crate::torrent::magnet::parse_magnet;
    use crate::torrent::Torrent;
    use crate::tracker::manager::manager_thread;
    use crate::tracker::PeerConnectionInfo;

    // #[ignore]
    #[test]
    fn test_handshake_with_magnet() {
        let peer = PeerConnectionInfo {
            ip: "192.168.1.218".to_owned(),
            port: 59500,
        };

        let magnet_uri = "magnet:?xt=urn:btih:d73b8fb6ffad7be4bd447ed9b1d56bda2c0bf2bf&dn=%5BSubsPlease%5D%20Chainsaw%20Man%20-%2005%20%281080p%29%20%5B16CC6267%5D.mkv&tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce".as_bytes().to_vec();
        let info_hash = parse_magnet(magnet_uri).unwrap();
        let info_hash = Torrent::from_info_hash(&info_hash).unwrap().get_info_hash();

        let (tx, rx): (
            Sender<Vec<PeerConnectionInfo>>,
            Receiver<Vec<PeerConnectionInfo>>,
        ) = mpsc::channel();

        let handle = thread::spawn(move || manager_thread(rx, "AAAAAAAAAAAAAAAAAAAA", &info_hash));
        tx.send(vec![peer]).unwrap();

        handle.join().unwrap().unwrap();
    }
}
