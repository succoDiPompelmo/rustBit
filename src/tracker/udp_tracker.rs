use tokio::net::UdpSocket;

use url::Url;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum UdpTrackerError {
    #[error("Tracker missing post")]
    PostMissing(),
    #[error("Tracker missing hostname")]
    HostnameMissing(),
    #[error("Wrong response size less than 26 bytes")]
    WrongResponseSize(),
    #[error("No connection established with udp tracker")]
    NoConnectionEstablished(),
    #[error("Error in recieving packet through socket")]
    RecieveError(),
    #[error("Error in sending packet through socket")]
    SendError(),
}

pub async fn call(
    info_hash: &[u8],
    peer_id: &str,
    tracker_url: &Url,
) -> Result<Vec<u8>, UdpTrackerError> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .expect("couldn't bind to address");
    let tracker_hostname = format!(
        "{}:{}",
        tracker_url
            .host_str()
            .ok_or(UdpTrackerError::HostnameMissing())?,
        tracker_url.port().ok_or(UdpTrackerError::PostMissing())?
    );

    socket
        .connect(tracker_hostname.clone())
        .await
        .expect("Panico");

    let transaction_id: &[u8] = &[0x00, 0x01, 0x19, 0x9e];
    let connection_id = connect_to_tracker(transaction_id, &socket).await?;

    let message = &make_announce_message(
        transaction_id,
        &connection_id,
        peer_id.as_bytes(),
        info_hash,
    );
    send_upd_packet(&socket, message).await?;

    let mut annouce_buf: [u8; 4000] = [0x00; 4000];
    let resp_size = read_upd_packet(&socket, &mut annouce_buf).await?;
    if resp_size > 26 {
        return Ok(annouce_buf[20..resp_size].to_vec());
    }

    Err(UdpTrackerError::WrongResponseSize())
}

async fn connect_to_tracker(
    transaction_id: &[u8],
    socket: &UdpSocket,
) -> Result<Vec<u8>, UdpTrackerError> {
    let message = &make_connection_message(transaction_id);

    let mut buf: [u8; 16] = [0x00; 16];

    for retry in 1..5 {
        if send_upd_packet(socket, message).await.is_err() {
            log::error!("Send udp connect packet failed, retry number {}", retry);
        };

        if read_upd_packet(socket, &mut buf).await.is_ok() {
            return Ok(buf[8..].to_vec());
        }
    }

    Err(UdpTrackerError::NoConnectionEstablished())
}

async fn read_upd_packet(socket: &UdpSocket, buffer: &mut [u8]) -> Result<usize, UdpTrackerError> {
    let (resp_size, _) = socket
        .recv_from(buffer)
        .await
        .map_err(|_| UdpTrackerError::RecieveError())?;

    Ok(resp_size)
}

async fn send_upd_packet(socket: &UdpSocket, message: &[u8]) -> Result<(), UdpTrackerError> {
    match socket.send(message).await {
        Ok(_) => Ok(()),
        Err(_) => Err(UdpTrackerError::SendError()),
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
    let action = &1_u32.to_be_bytes();
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
