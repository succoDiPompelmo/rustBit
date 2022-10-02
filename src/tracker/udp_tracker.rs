use std::net::UdpSocket;
use std::time::Duration;

use crate::tracker::tracker::Tracker;

pub fn get_tracker(
    info_hash: &Vec<u8>,
    peer_id: &str,
    tracker_url: &str,
) -> Result<Tracker, &'static str> {
    let socket = UdpSocket::bind("0.0.0.0:34222").expect("couldn't bind to address");
    let tracker_hostname = get_tracker_hostname(tracker_url);
    let transaction_id: &[u8] = &[0x00, 0x01, 0x19, 0x9e];
    let connection_id = connect_to_tracker(transaction_id, &socket, tracker_hostname)?;

    let message = &make_announce_message(
        transaction_id,
        &connection_id,
        peer_id.as_bytes(),
        info_hash.as_slice(),
    );
    send_upd_packet(&socket, message, tracker_hostname)?;

    socket.set_read_timeout(Some(Duration::new(10, 0))).unwrap();
    let mut annouce_buf: [u8; 4000] = [0x00; 4000];

    let resp_size = read_upd_packet(&socket, &mut annouce_buf)?;

    if resp_size > 26 {
        let peers_info = Tracker::peers_info_from_bytes(&annouce_buf[20..resp_size].to_vec());
        return Ok(Tracker {
            interval: 0,
            peers: peers_info,
        });
    }

    Err("")
}

fn connect_to_tracker(
    transaction_id: &[u8],
    socket: &UdpSocket,
    tracker_hostname: &str,
) -> Result<Vec<u8>, &'static str> {
    let message = &make_connection_message(transaction_id);

    let mut buf: [u8; 16] = [0x00; 16];

    for _ in 1..5 {
        send_upd_packet(&socket, message, tracker_hostname)?;
        if read_upd_packet(&socket, &mut buf).is_ok() {
            return Ok(buf[8..].to_vec());
        }
    }

    Err("No connection to tracker")
}

fn read_upd_packet(socket: &UdpSocket, buffer: &mut [u8]) -> Result<usize, &'static str> {
    socket.set_read_timeout(Some(Duration::new(10, 0))).unwrap();

    let (resp_size, _) = socket
        .recv_from(buffer)
        .or_else(|_| Err("recv function failed: {e:?}"))?;

    Ok(resp_size)
}

fn send_upd_packet(socket: &UdpSocket, message: &[u8], hostname: &str) -> Result<(), &'static str> {
    match socket.send_to(message, hostname) {
        Ok(_) => Ok(()),
        Err(_) => Err("couldn't send message"),
    }
}

fn make_connection_message(transaction_id: &[u8]) -> Vec<u8> {
    let connection_id: &[u8] = &[0x00, 0x00, 0x04, 0x17, 0x27, 0x10, 0x19, 0x80];
    let action: &[u8] = &[0x00, 0x00, 0x00, 0x00];

    [connection_id, action, transaction_id].concat().to_vec()
}

fn make_announce_message(
    transaction_id: &[u8],
    connection_id: &[u8],
    peer_id: &[u8],
    info_hash: &[u8],
) -> Vec<u8> {
    let action = &(1 as u32).to_be_bytes();
    // let action: &[u8] = &[0x00, 0x00, 0x00, 0x01];
    let downloaded: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let left: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let uploaded: &[u8] = &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let event: &[u8] = &[0x00, 0x00, 0x00, 0x00];
    let ip: &[u8] = &[0x00, 0x00, 0x00, 0x00];
    let key: &[u8] = &[0x00, 0x00, 0x00, 0x00];
    let num_want: &[u8] = &[0xff, 0xff, 0xff, 0xff]; // -1
    let port: &[u8] = &[0x1f, 0x40]; // 8000

    [
        connection_id,
        action,
        transaction_id,
        info_hash,
        peer_id,
        downloaded,
        left,
        uploaded,
        event,
        ip,
        key,
        num_want,
        port,
    ]
    .concat()
    .to_vec()
}

fn get_tracker_hostname(tracker_url: &str) -> &str {
    tracker_url
        .strip_prefix("udp://")
        .expect("No preifx")
        .strip_suffix("/announce")
        .expect("No suffix")
}
