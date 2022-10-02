use std::env;
use std::fs::File;
use std::io::prelude::*;

use crate::torrent::magnet;
use crate::torrent::torrent::Torrent;

mod bencode;
mod integration_test;
mod messages;
mod peer;
mod torrent;
mod tracker;

fn read_file() -> Vec<u8> {
    let mut file = File::open("torrent_files/HouseOfDragons.torrent").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    return contents;
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("Provide magnet link or path to a torrent file")
    }

    let torrent_source = &args[1];

    let mut torrent = if torrent_source.ends_with(".torrent") {
        let contents = read_file();
        let decoded_data = bencode::decode::Decoder::init(contents).decode();
        Torrent::from_metainfo(&decoded_data).unwrap()
    } else {
        let magnet = magnet::parse_magnet(torrent_source.as_bytes().to_vec()).unwrap();
        Torrent::from_info_hash(&magnet).unwrap()
    };

    tracker::tracker::Tracker::init_tracker(&mut torrent).unwrap();
    Ok(())
}
