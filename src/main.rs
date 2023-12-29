mod actors;
mod bencode;
mod common;
mod messages;
mod peer;
mod torrent;
mod tracker;

use actors::trackers_interface::TrackersInterfaceActor;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

use std::fs::File;
use std::io::Write;

use torrent::magnet;

use actix::prelude::*;
use actix_web::{post, web, App, HttpResponse, HttpServer};

use crate::actors::messages::TorrentRegistered;
use crate::actors::torrent::TorrentActor;

#[post("/add/magnet")]
async fn add_magnet(data: web::Data<AppState>, magnet_raw: String) -> HttpResponse {
    let magnet = magnet::parse_magnet(magnet_raw.as_bytes().to_vec()).unwrap();
    let info_hash = magnet.get_info_hash();

    let addr = TorrentActor::new(info_hash.clone()).start();

    let msg = TorrentRegistered {
        info_hash,
        torrent_actor_addr: addr.clone(),
    };

    let _ = data.trackers_interface.try_send(msg);

    HttpResponse::Ok().body("Test")
}

struct AppState {
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

    let state = web::Data::new(AppState { trackers_interface });

    HttpServer::new(move || App::new().app_data(state.clone()).service(add_magnet))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
