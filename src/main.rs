use std::fs::File;
use std::io::prelude::*;

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
    let contents = read_file();
    let decoded_data = bencode::decode::Decoder::init(contents).decode();
    let mut torrent = Torrent::from_metainfo(&decoded_data).unwrap();

    tracker::tracker::Tracker::init_tracker(&mut torrent).unwrap();

    Ok(())
}
