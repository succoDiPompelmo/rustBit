use std::{net::TcpStream, time::Duration};

pub struct PeerEndpoint(String, u16);

impl PeerEndpoint {
    pub fn from_bytes(bytes: &[u8]) -> Vec<Self> {
        let mut peers_info = Vec::new();

        for chunk in bytes.chunks_exact(6) {
            let ip = format!(
                "{:?}.{:?}.{:?}.{:?}",
                chunk[0], chunk[1], chunk[2], chunk[3]
            );
            let port = ((chunk[4] as u16) << 8) | (chunk[5] as u16);

            peers_info.push(PeerEndpoint(ip, port))
        }

        peers_info
    }

    pub fn is_reachable(endpoint: &str) -> bool {
        match endpoint.parse() {
            Ok(socket) => TcpStream::connect_timeout(&socket, Duration::from_millis(300)).is_ok(),
            Err(_) => false,
        }
    }

    pub fn ip(&self) -> String {
        self.0.to_string()
    }

    pub fn port(&self) -> u16 {
        self.1
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_peers_endpoint_from_bytes() {
        let bytes = vec![b'3', b'4', b'0', b'1', b'1', b'1'];

        let peers_endpoint = PeerEndpoint::from_bytes(&bytes);

        assert_eq!(peers_endpoint.len(), 1);

        let peer = peers_endpoint.first().unwrap();

        assert_eq!(peer.ip(), "51.52.48.49");
        assert_eq!(peer.port(), 12593);
    }
}
