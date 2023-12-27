mod actors;
mod bencode;
mod common;
mod integration_test;
mod messages;
mod peer;
mod torrent;
mod tracker;

use actors::trackersInterface::TrackersInterfaceActor;
use chrono::Local;
use env_logger::Builder;
use log::{info, LevelFilter};

use std::io::Write;
use std::{fs::File, sync::Mutex};

use common::file::read_file;
use torrent::magnet;
use torrent::manager::TorrentManager;
use torrent::Torrent;
use tracker::Tracker;

use actix::prelude::*;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};

use crate::actors::messages::TorrentRegistered;
use crate::actors::torrent::TorrentActor;

#[get("trackers")]
async fn trackers(data: web::Data<AppState>) -> HttpResponse {
    let raw_magnet = "magnet:?xt=urn:btih:ef57d5083f6f8be4bb8a393902b870c6ba9d58ee&dn=%5BSubsPlease%5D%20Kage%20no%20Jitsuryokusha%20ni%20Naritakute%21%20S2%20-%2010%20%281080p%29%20%5B761085EA%5D.mkv&tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce";

    let info_hash = magnet::parse_magnet(raw_magnet.as_bytes().to_vec())
        .unwrap()
        .get_info_hash();

    let addr = TorrentActor::new(info_hash.clone()).start();

    let msg = TorrentRegistered {
        info_hash,
        torrent_actor_addr: addr.clone(),
    };

    let _ = data.trackers_interface.try_send(msg);

    HttpResponse::Ok().body("Test")
}

#[post("/torrent/magnet")]
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

struct AppState {
    background_torrents: Mutex<Vec<Addr<TorrentActor>>>,
    trackers_interface: Addr<TrackersInterfaceActor>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let trackers_interface = TrackersInterfaceActor::new().start();

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

    let state = web::Data::new(AppState {
        background_torrents: Mutex::new(vec![]),
        trackers_interface,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(add_magnet)
            .service(trackers)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
