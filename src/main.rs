mod bencode;
mod common;
mod integration_test;
mod messages;
mod peer;
mod torrent;
mod tracker;

use chrono::Local;
use env_logger::Builder;
use log::{info, LevelFilter};

use std::fs::File;
use std::io::Write;

use common::file::read_file;
use torrent::magnet;
use torrent::manager::TorrentManager;
use torrent::Torrent;
use tracker::Tracker;

use actix_web::{post, App, HttpResponse, HttpServer};

#[post("/torrent")]
async fn add_magnet(torrent_source: String) -> HttpResponse {
    info!("{:?}", torrent_source);

    let torrent = if torrent_source.ends_with(".torrent") {
        let contents = read_file("torrent_files/HouseOfDragons.torrent");
        let decoded_data = bencode::decode::Decoder::init(contents).decode().unwrap();
        Torrent::from_metainfo(&decoded_data).unwrap()
    } else {
        let magnet = magnet::parse_magnet(torrent_source.as_bytes().to_vec()).unwrap();
        Torrent::from_info_hash(&magnet)
    };

    actix_web::rt::spawn(Tracker::find_peers(torrent.get_info_hash()));
    actix_web::rt::spawn(TorrentManager::init(torrent));

    HttpResponse::Ok().body("Torrent registered")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let target = Box::new(File::create("./test.log").expect("Can't create file"));
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(target))
        .filter(None, LevelFilter::Debug)
        .filter_module("sqlx::query", log::LevelFilter::Off)
        .init();

    HttpServer::new(|| App::new().service(add_magnet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
