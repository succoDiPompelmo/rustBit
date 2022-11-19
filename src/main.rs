mod bencode;
mod common;
mod integration_test;
mod messages;
mod peer;
mod torrent;
mod tracker;

use std::fs::File;
use std::io::prelude::*;
use std::thread;

use crate::torrent::magnet;
use crate::torrent::Torrent;
use crate::tracker::Tracker;

fn read_file() -> Vec<u8> {
    let mut file = File::open("torrent_files/HouseOfDragons.torrent").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();

    contents
}

use actix_web::{post, App, HttpResponse, HttpServer};

#[post("/torrent")]
async fn add_magnet(torrent_source: String) -> HttpResponse {
    println!("{:?}", torrent_source);

    let mut torrent = if torrent_source.ends_with(".torrent") {
        let contents = read_file();
        let decoded_data = bencode::decode::Decoder::init(contents).decode();
        Torrent::from_metainfo(&decoded_data).unwrap()
    } else {
        let magnet = magnet::parse_magnet(torrent_source.as_bytes().to_vec()).unwrap();
        Torrent::from_info_hash(&magnet).unwrap()
    };

    thread::spawn(move || Tracker::init_tracker(&mut torrent));
    HttpResponse::Ok().body("Torrent registered")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(add_magnet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
