use crate::bencode::decode::Decoder;
use crate::tracker::tracker::Tracker;
use std::time::Duration;

use crate::bencode::metainfo;

use std::fs::File;
use std::io::prelude::*;

pub fn get_tracker(
    info_hash: &Vec<u8>,
    peer_id: &str,
    tracker: &str,
) -> Result<Tracker, &'static str> {
    let url_encoded_info_hash = urlencoding::encode_binary(info_hash.as_slice()).into_owned();

    let tracker_url = format!(
        "{}?info_hash={}&peer_id={}",
        tracker, url_encoded_info_hash, peer_id
    );

    let result = get_peers(tracker_url)?;
    let tracker_metainfo = Decoder::init(result).decode();
    let tracker = from_metainfo(tracker_metainfo)?;

    return Ok(tracker);
}

fn from_metainfo(metainfo: metainfo::Metainfo) -> Result<Tracker, &'static str> {
    let interval = metainfo::get_integer_from_dict(&metainfo, "interval")?;
    let peers_list = match metainfo::get_value_from_dict(&metainfo, "peers")? {
        metainfo::Metainfo::String(peers) => peers,
        _ => return Err("No pieces found"),
    };

    return Ok(Tracker {
        interval: interval,
        peers: Tracker::peers_info_from_bytes(peers_list),
    });
}

fn get_peers(url: String) -> Result<Vec<u8>, &'static str> {
    let respone = call_tracker_for_peers(url)?;
    let mut bytes = vec![];
    respone.into_reader().read_to_end(&mut bytes);
    return Ok(bytes);
}

fn call_tracker_for_peers(url: String) -> Result<ureq::Response, &'static str> {
    println!("{:?}", url);

    for _ in 1..2 {
        let response_result = ureq::get(&url)
            .timeout(Duration::new(1, 0))
            .set("Accept-Encoding", "gzip, deflate, br")
            .set("Accept", "*/*")
            .set("Connection", "keep-alive")
            .set("User-Agent", "PostmanRuntime/7.29.0")
            .set("Host", "192.168.1.2")
            .call();

        match response_result {
            Err(err) => (),
            Ok(response) => return Ok(response),
        }
    }

    return Err("Maximum retires reached");
}
