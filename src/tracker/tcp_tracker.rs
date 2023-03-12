use std::io::prelude::*;
use std::time::Duration;

use crate::bencode::decode::{Decoder, DecoderError};
use crate::bencode::metainfo::MetainfoError;

use log::error;
use url::Url;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum TcpTrackerError {
    #[error("Error handling metainfo")]
    Metainfo(#[from] MetainfoError),
    #[error("Error during metainfo decoding")]
    Decoder(#[from] DecoderError),
    #[error("Error during connection to tracker {0}")]
    Connection(String),
    #[error("Error during reading of buffer")]
    BufferReading(),
}

pub fn call(info_hash: &[u8], peer_id: &str, tracker: Url) -> Result<Vec<u8>, TcpTrackerError> {
    let url_encoded_info_hash = urlencoding::encode_binary(info_hash).into_owned();

    let tracker_url = format!(
        "{}?info_hash={}&peer_id={}",
        tracker.as_str(),
        url_encoded_info_hash,
        peer_id
    );

    let result = get_peers(tracker_url)?;
    let tracker_metainfo = Decoder::init(result).decode()?;
    Ok(tracker_metainfo.get_bytes_from_dict("peers")?)
}

fn get_peers(url: String) -> Result<Vec<u8>, TcpTrackerError> {
    let respone = call_tracker_for_peers(url)?;
    let mut bytes = vec![];
    respone
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|_| TcpTrackerError::BufferReading())?;
    Ok(bytes)
}

fn call_tracker_for_peers(url: String) -> Result<ureq::Response, TcpTrackerError> {
    ureq::get(&url)
        .timeout(Duration::from_millis(400))
        .set("Accept-Encoding", "gzip, deflate, br")
        .set("Accept", "*/*")
        .set("Connection", "keep-alive")
        .set("User-Agent", "PostmanRuntime/7.29.0")
        .set("Host", "192.168.1.2")
        .call()
        .map_err(|_| TcpTrackerError::Connection(url))
}
