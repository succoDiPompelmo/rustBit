use std::io::prelude::*;
use std::time::Duration;

use crate::bencode::decode::Decoder;
use crate::bencode::metainfo;
use crate::tracker::Tracker;

pub fn get_tracker(
    info_hash: &[u8],
    peer_id: &str,
    tracker: &str,
) -> Result<Tracker, &'static str> {
    let url_encoded_info_hash = urlencoding::encode_binary(info_hash).into_owned();

    let tracker_url = format!(
        "{}?info_hash={}&peer_id={}",
        tracker, url_encoded_info_hash, peer_id
    );

    let result = get_peers(tracker_url)?;
    let tracker_metainfo = Decoder::init(result).decode()?;
    let tracker = from_metainfo(tracker_metainfo)?;

    Ok(tracker)
}

fn from_metainfo(metainfo: metainfo::Metainfo) -> Result<Tracker, &'static str> {
    let interval = metainfo::get_integer_from_dict(&metainfo, "interval")?;
    let peers_list = match metainfo::get_value_from_dict(&metainfo, "peers")? {
        metainfo::Metainfo::String(peers) => peers,
        _ => return Err("No pieces found"),
    };

    Ok(Tracker {
        interval,
        peers: Tracker::peers_info_from_bytes(peers_list),
    })
}

fn get_peers(url: String) -> Result<Vec<u8>, &'static str> {
    let respone = call_tracker_for_peers(url)?;
    let mut bytes = vec![];
    respone.into_reader().read_to_end(&mut bytes).unwrap();
    Ok(bytes)
}

fn call_tracker_for_peers(url: String) -> Result<ureq::Response, &'static str> {
    for _ in 1..2 {
        let response_result = ureq::get(&url)
            .timeout(Duration::from_millis(500))
            .set("Accept-Encoding", "gzip, deflate, br")
            .set("Accept", "*/*")
            .set("Connection", "keep-alive")
            .set("User-Agent", "PostmanRuntime/7.29.0")
            .set("Host", "192.168.1.2")
            .call();

        match response_result {
            Err(_) => (),
            Ok(response) => return Ok(response),
        }
    }

    Err("Maximum retires reached")
}
