#[cfg(test)]
mod test {
    use crate::bencode::decode::Decoder;
    use crate::read_file;
    use crate::torrent::magnet::parse_magnet;
    use crate::torrent::Torrent;
    use crate::tracker::manager::download;
    use crate::tracker::PeerConnectionInfo;

    #[ignore]
    #[test]
    fn test_handshake() {
        let peer = PeerConnectionInfo {
            ip: "192.168.1.218".to_owned(),
            port: 59500,
        };

        let contents = read_file();

        let mut decoder = Decoder::init(contents);
        let decoded_data = decoder.decode();
        let torrent = &mut Torrent::from_metainfo(&decoded_data).unwrap();

        download(vec![peer], "bbaaaaaaaaaaaaaaaaaa", torrent).unwrap();
    }

    // #[ignore]
    #[test]
    fn test_handshake_with_magnet() {
        let peer = PeerConnectionInfo {
            ip: "192.168.1.11".to_owned(),
            port: 61893,
        };

        let magnet_uri = "magnet:?xt=urn:btih:3609aa1a54b9c198da0f3811a29fbac83953ed37&dn=%5BASW%5D%20Spy%20x%20Family%20-%2013%20%5B1080p%20HEVC%20x265%2010Bit%5D%5BAAC%5D&tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce".as_bytes().to_vec();
        let info_hash = parse_magnet(magnet_uri).unwrap();
        let torrent = &mut Torrent::from_info_hash(&info_hash).unwrap();
        let result = download(vec![peer], "bbaaaaaaaaaaaaaaaaaa", torrent);

        println!("{:?}", result);
    }
}
