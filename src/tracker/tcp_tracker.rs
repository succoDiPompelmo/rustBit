use std::io::prelude::*;
use std::time::Duration;

use crate::bencode::decode::Decoder;
use crate::bencode::metainfo::Metainfo;
use crate::tracker::Tracker;

use super::PeerConnectionInfo;

pub fn get_tracker(
    info_hash: &[u8],
    peer_id: &str,
    tracker: &str,
) -> Result<Vec<PeerConnectionInfo>, &'static str> {
    let url_encoded_info_hash = urlencoding::encode_binary(info_hash).into_owned();

    let tracker_url = format!(
        "{}?info_hash={}&peer_id={}",
        tracker, url_encoded_info_hash, peer_id
    );

    let result = get_peers(tracker_url)?;
    let tracker_metainfo = Decoder::init(result).decode()?;
    from_metainfo(tracker_metainfo)
}

fn from_metainfo(metainfo: Metainfo) -> Result<Vec<PeerConnectionInfo>, &'static str> {
    let peers_list = metainfo.get_value_from_dict("peers")?.get_bytes_content()?;

    Ok(Tracker::peers_info_from_bytes(&peers_list))
}

fn get_peers(url: String) -> Result<Vec<u8>, &'static str> {
    let respone = call_tracker_for_peers(url)?;
    let mut bytes = vec![];
    respone.into_reader().read_to_end(&mut bytes).unwrap();
    Ok(bytes)
}

fn call_tracker_for_peers(url: String) -> Result<ureq::Response, &'static str> {
    let response_result = ureq::get(&url)
        .timeout(Duration::from_millis(400))
        .set("Accept-Encoding", "gzip, deflate, br")
        .set("Accept", "*/*")
        .set("Connection", "keep-alive")
        .set("User-Agent", "PostmanRuntime/7.29.0")
        .set("Host", "192.168.1.2")
        .call();

    match response_result {
        Err(_) => Err("Error calling tcp tracker"),
        Ok(response) => Ok(response),
    }
}
