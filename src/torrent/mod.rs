pub mod file;
pub mod magnet;
pub mod writer;

use sha1::{Digest, Sha1};

use crate::bencode::decode::Decoder;
use crate::bencode::encode::{encode_dict_entry, Encode};
use crate::bencode::metainfo;
use crate::bencode::metainfo::Metainfo;
use crate::torrent::file::File;
use crate::torrent::magnet::Magnet;

#[derive(Debug, PartialEq, Eq)]
pub struct Torrent {
    announce: String,
    announce_list: Option<Vec<Vec<String>>>,
    info: Option<Info>,
    info_hash: Vec<u8>,
}

impl Torrent {
    pub fn from_metainfo(a: &Metainfo) -> Result<Torrent, &'static str> {
        let announce = metainfo::get_string_from_dict(a, "announce")?;

        let announce_list = match metainfo::get_list_from_dict(a, "announce-list") {
            Ok(announce_list_metainfo) => {
                let announce_list = announce_list_from_metainfo(announce_list_metainfo);
                Some(announce_list)
            }
            Err(_) => None,
        };

        let info_metainfo = metainfo::get_value_from_dict(a, "info")?;
        let info = Info::from_metainfo(info_metainfo)?;
        let info_hash = compute_info_hash(info.encode());

        Ok(Torrent {
            announce,
            announce_list,
            info: Some(info),
            info_hash,
        })
    }

    pub fn from_info_hash(magnet: &Magnet) -> Result<Torrent, &'static str> {
        Ok(Torrent {
            announce: "".to_owned(),
            announce_list: None,
            info: None,
            info_hash: magnet.get_info_hash(),
        })
    }

    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.to_vec()
    }
}

fn compute_info_hash(bencoded_info: Vec<u8>) -> Vec<u8> {
    Sha1::digest(bencoded_info).as_slice().to_owned()
}

fn announce_list_from_metainfo(announce_list_metainfo: &[Metainfo]) -> Vec<Vec<String>> {
    let mut announce_list: Vec<Vec<String>> = Vec::new();
    for announce_item_metainfo in announce_list_metainfo {
        if let Ok(announce_item) = metainfo::get_list_content(announce_item_metainfo) {
            announce_list.push(
                announce_item
                    .iter()
                    .map(|item| metainfo::get_string_content(item).unwrap_or_default())
                    .collect::<Vec<String>>(),
            );
        };
    }
    announce_list
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Info {
    files: Option<Vec<File>>,
    length: Option<usize>,
    name: String,
    piece_length: usize,
    pieces: Vec<u8>,
}

impl Info {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Info, &'static str> {
        let decoded_info = Decoder::init(bytes).decode();
        Info::from_metainfo(&decoded_info)
    }

    pub fn from_metainfo(a: &Metainfo) -> Result<Info, &'static str> {
        let pieces = match metainfo::get_value_from_dict(a, "pieces")? {
            Metainfo::String(pieces) => pieces,
            _ => return Err("No pieces found"),
        };
        let piece_length = metainfo::get_integer_from_dict(a, "piece length")?;
        let name = metainfo::get_string_from_dict(a, "name")?;
        let length = metainfo::get_integer_from_dict(a, "length").ok();

        let files = match metainfo::get_list_from_dict(a, "files") {
            Ok(metainfo_files) => {
                let mut output_files = Vec::new();
                for metainfo_file in metainfo_files {
                    output_files.push(File::from_metainfo(metainfo_file)?)
                }
                Some(output_files)
            }
            Err(_) => None,
        };

        Ok(Info {
            name,
            pieces: pieces.to_vec(),
            piece_length,
            files,
            length,
        })
    }

    pub fn get_piece(&self, index: usize) -> &[u8] {
        self.pieces
            .chunks_exact(20)
            .nth(index)
            .expect("No piece at the provided index")
    }

    pub fn verify_piece(&self, piece: &[u8], piece_idx: usize) -> bool {
        Sha1::digest(piece).as_slice() == self.get_piece(piece_idx)
    }

    pub fn get_piece_length(&self) -> usize {
        self.piece_length
    }

    pub fn get_total_length(&self) -> usize {
        match &self.files {
            Some(files) => files.iter().map(|file| file.get_length()).sum::<usize>(),
            None => self.length.unwrap(),
        }
    }

    pub fn get_files(&self) -> Result<Vec<File>, &'static str> {
        match &self.files {
            Some(files) => Ok(files.to_vec()),
            None => {
                let file = File::new(vec![self.name.to_owned()], self.length.unwrap());
                Ok(vec![file])
            }
        }
    }
}

impl Encode for Info {
    fn encode(&self) -> Vec<u8> {
        let files = encode_dict_entry("files", &self.files);
        let length = encode_dict_entry("files", &self.length);
        let name = encode_dict_entry("name", &self.name);
        let piece_length = encode_dict_entry("piece length", &self.piece_length);
        let pieces = encode_dict_entry("pieces", &self.pieces);

        [
            "d".as_bytes(),
            files.as_slice(),
            length.as_slice(),
            name.as_slice(),
            piece_length.as_slice(),
            pieces.as_slice(),
            "e".as_bytes(),
        ]
        .concat()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::bencode::decode::Decoder;
    use std::fs::File as Fs;
    use std::io::prelude::*;

    fn read_test_data(file_name: &str) -> Vec<u8> {
        let mut file = Fs::open(file_name).unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();

        contents
    }

    #[test]
    fn from_metainfo_single_file() {
        let torrent_bytes = read_test_data("src/torrent/test_data/fake_debian.torrent");
        let torrent_metainfo = Decoder::init(torrent_bytes).decode();
        let torrents_result = Torrent::from_metainfo(&torrent_metainfo);

        let pieces = vec![
            239, 191, 189, 67, 38, 69, 39, 239, 191, 189, 239, 191, 189, 239, 191, 189, 239, 191,
            189, 39, 110, 239, 191, 189, 104, 211, 157, 239, 191, 189, 5, 239, 191, 189, 239, 191,
            189, 54, 52, 51,
        ];

        let info = Info {
            piece_length: 262144,
            name: "debian-11.3.0-amd64-netinst.iso".to_owned(),
            pieces: pieces,
            files: None,
            length: Some(396361728),
        };

        let info_hash = compute_info_hash(info.encode());

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
        let torrent_metainfo = Decoder::init(torrent_bytes).decode();
        let torrents_result = Torrent::from_metainfo(&torrent_metainfo);

        let pieces = vec![
            6, 239, 191, 189, 239, 191, 189, 239, 191, 189, 239, 191, 189, 89, 239, 191, 189, 69,
            239, 191, 189, 219, 129, 8, 72, 41, 56, 22, 45, 239, 191, 189, 51, 239, 191, 189, 94,
            48, 48, 48,
        ];

        let info = Info {
            piece_length: 8388608,
            name: "Prey.2022.1080p.DSNP.WEBRip.DDP5.1.Atmos.x264-CM".to_owned(),
            pieces: pieces,
            files: Some(vec![
                File::new(
                    vec!["Prey.2022.1080p.DSNP.WEB-DL.DDP5.1.Atmos.H.264-CM.mkv".to_owned()],
                    5482855733,
                ),
                File::new(vec!["RARBG.txt".to_owned()], 31),
            ]),
            length: None,
        };

        let info_hash = compute_info_hash(info.encode());

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

    #[test]
    fn encode_info() {
        let file = File::new(vec!["/bin".to_owned()], 234);

        let info = Info {
            name: "pippo".to_owned(),
            piece_length: 43921,
            pieces: "ABCDE".as_bytes().to_vec(),
            files: Some(Vec::from([file])),
            length: None,
        };

        let expected_hash = "d5:filesld6:lengthi234e4:pathl4:/bineee4:name5:pippo12:piece lengthi43921e6:pieces5:ABCDEe";
        let result_hash = info.encode();

        assert_eq!(expected_hash.as_bytes().to_vec(), result_hash);
    }
}
