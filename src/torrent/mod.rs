pub mod file;
pub mod info;
pub mod magnet;
pub mod manager;
pub mod writer;

use crate::bencode::decode::DecoderError;
use crate::bencode::metainfo::{Metainfo, MetainfoError};
use crate::torrent::info::Info;
use crate::torrent::magnet::Magnet;

use self::info::InfoError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Torrent {
    announce: String,
    announce_list: Option<Vec<Vec<String>>>,
    info: Option<Info>,
    info_hash: Vec<u8>,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum TorrentError {
    #[error("Error handling metainfo")]
    Metainfo(#[from] MetainfoError),
    #[error("Error during metainfo decoding")]
    Decoder(#[from] DecoderError),
    #[error("Error during info decoding")]
    Info(#[from] InfoError),
}

impl Torrent {
    pub fn from_metainfo(a: &Metainfo) -> Result<Torrent, TorrentError> {
        let announce = a.get_string_from_dict("announce")?;
        let announce_list = a
            .get_list_from_dict("announce-list")
            .ok()
            .map(|el| announce_list_from_metainfo(el));

        let info_metainfo = a.get_value_from_dict("info")?;
        let info = Info::from_metainfo(info_metainfo)?;
        let info_hash = info.compute_info_hash();

        Ok(Torrent {
            announce,
            announce_list,
            info: Some(info),
            info_hash,
        })
    }

    pub fn from_info_hash(magnet: &Magnet) -> Torrent {
        Torrent {
            announce: "".to_owned(),
            announce_list: None,
            info: None,
            info_hash: magnet.get_info_hash(),
        }
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.to_vec()
    }
}

fn announce_list_from_metainfo(elements: &[Metainfo]) -> Vec<Vec<String>> {
    let mut announce_list: Vec<Vec<String>> = Vec::new();
    for element in elements {
        element
            .get_list_content()
            .map_or((), |item| announce_list.push(announce_list_item(item)));
    }
    announce_list
}

fn announce_list_item(item: &[Metainfo]) -> Vec<String> {
    item.iter()
        .map(|item| item.get_string_content().unwrap_or_default())
        .collect::<Vec<String>>()
}

#[cfg(test)]
mod test {

    use super::*;

    use std::fs::File as Fs;
    use std::io::prelude::*;

    use crate::bencode::decode::Decoder;
    use crate::torrent::file::File;

    fn read_test_data(file_name: &str) -> Vec<u8> {
        let mut file = Fs::open(file_name).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();

        contents
    }

    #[test]
    fn from_metainfo_single_file() {
        let torrent_bytes = read_test_data("src/torrent/test_data/fake_debian.torrent");
        let torrent_metainfo = Decoder::init(torrent_bytes).decode().unwrap();
        let torrents_result = Torrent::from_metainfo(&torrent_metainfo);

        let pieces = vec![
            239, 191, 189, 67, 38, 69, 39, 239, 191, 189, 239, 191, 189, 239, 191, 189, 239, 191,
            189, 39, 110, 239, 191, 189, 104, 211, 157, 239, 191, 189, 5, 239, 191, 189, 239, 191,
            189, 54, 52, 51,
        ];

        let info = Info::new(
            None,
            Some(396361728),
            "debian-11.3.0-amd64-netinst.iso".to_owned(),
            262144,
            pieces,
        );

        let info_hash = info.compute_info_hash();

        let expected_torrents = Torrent {
            announce: "http://bttracker.debian.org:6969/announce".to_owned(),
            announce_list: None,
            info: Some(info),
            info_hash,
        };

        match torrents_result {
            Ok(torrent) => assert_eq!(torrent, expected_torrents),
            Err(_) => panic!("Error parsing torrent metainfo {:?}", torrents_result),
        }
    }

    #[test]
    fn from_metainfo_multiple_file() {
        let torrent_bytes = read_test_data("src/torrent/test_data/fake_prey.torrent");
        let torrent_metainfo = Decoder::init(torrent_bytes).decode().unwrap();
        let torrents_result = Torrent::from_metainfo(&torrent_metainfo);

        let pieces = vec![
            6, 239, 191, 189, 239, 191, 189, 239, 191, 189, 239, 191, 189, 89, 239, 191, 189, 69,
            239, 191, 189, 219, 129, 8, 72, 41, 56, 22, 45, 239, 191, 189, 51, 239, 191, 189, 94,
            48, 48, 48,
        ];

        let info = Info::new(
            Some(vec![
                File::new(
                    vec!["Prey.2022.1080p.DSNP.WEB-DL.DDP5.1.Atmos.H.264-CM.mkv".to_owned()],
                    5482855733,
                ),
                File::new(vec!["RARBG.txt".to_owned()], 31),
            ]),
            None,
            "Prey.2022.1080p.DSNP.WEBRip.DDP5.1.Atmos.x264-CM".to_owned(),
            8388608,
            pieces,
        );

        let info_hash = info.compute_info_hash();

        let expected_torrents = Torrent {
            announce: "http://tracker.trackerfix.com:80/announce".to_owned(),
            announce_list: Some(vec![
                vec!["http://tracker.trackerfix.com:80/announce".to_owned()],
                vec!["udp://9.rarbg.me:2880/announce".to_owned()],
                vec!["udp://9.rarbg.to:2990/announce".to_owned()],
                vec!["udp://tracker.slowcheetah.org:14750/announce".to_owned()],
                vec!["udp://tracker.tallpenguin.org:15710/announce".to_owned()],
            ]),
            info: Some(info),
            info_hash,
        };

        match torrents_result {
            Ok(torrent) => assert_eq!(torrent, expected_torrents),
            Err(_) => panic!("Error parsing torrent metainfo {:?}", torrents_result),
        }
    }
}
