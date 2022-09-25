#![allow(warnings, unused)]

use core::time::Duration;
use std::fs::File;
use std::io::prelude::*;

use std::{thread, time};

use std::net::TcpStream;
use std::net::UdpSocket;

use crate::torrent::torrent::Torrent;

mod bencode;
mod messages;
mod peer;
mod torrent;
mod tracker;
mod integration_test;

fn read_file() -> Vec<u8> {
    let mut file = File::open("torrent_files/HouseOfDragons.torrent").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    return contents;
}

fn main() -> std::io::Result<()> {
    let contents = read_file();
    let mut decoded_data = bencode::decode::Decoder::init(contents).decode();
    let mut torrent = Torrent::from_metainfo(&decoded_data).unwrap();

    let tracker = tracker::tracker::Tracker::init_tracker(&mut torrent).unwrap();

    Ok(())
}
