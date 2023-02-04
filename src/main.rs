mod bencode;
mod bridge;
mod common;
mod integration_test;
mod messages;
mod peer;
mod torrent;
mod tracker;

use crate::common::file::read_file;
use crate::torrent::magnet;
use crate::torrent::manager::TorrentManager;
use crate::torrent::Torrent;
use crate::tracker::Tracker;

use actix_web::{post, App, HttpResponse, HttpServer};

#[post("/torrent")]
async fn add_magnet(torrent_source: String) -> HttpResponse {
    println!("{:?}", torrent_source);

    let torrent = if torrent_source.ends_with(".torrent") {
        let contents = read_file("torrent_files/HouseOfDragons.torrent");
        let decoded_data = bencode::decode::Decoder::init(contents).decode().unwrap();
        Torrent::from_metainfo(&decoded_data).unwrap()
    } else {
        let magnet = magnet::parse_magnet(torrent_source.as_bytes().to_vec()).unwrap();
        Torrent::from_info_hash(&magnet).unwrap()
    };

    actix_web::rt::spawn(Tracker::find_peers(torrent.get_info_hash()));
    actix_web::rt::spawn(TorrentManager::init(torrent));

    HttpResponse::Ok().body("Torrent registered")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(add_magnet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
