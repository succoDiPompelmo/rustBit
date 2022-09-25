#[cfg(test)]
mod test {
    use super::*;

    use crate::bencode::decode::Decoder;
    use crate::read_file;
    use crate::torrent::magnet::parse_magnet;
    use crate::torrent::torrent::Torrent;
    use crate::tracker::manager::download;
    use crate::tracker::tracker::PeerConnectionInfo;

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

        download(vec![peer], "bbaaaaaaaaaaaaaaaaaa", torrent);
    }

    #[ignore]
    #[test]
    fn test_handshake_with_magnet() {
        let peer = PeerConnectionInfo {
            ip: "192.168.1.218".to_owned(),
            port: 59500,
        };

        let magnet_uri = "".as_bytes().to_vec();
        let info_hash = parse_magnet(magnet_uri).unwrap();
        let torrent = &mut Torrent::from_info_hash(&info_hash).unwrap();
        let result = download(vec![peer], "bbaaaaaaaaaaaaaaaaaa", torrent);

        println!("{:?}", result);
    }
}
