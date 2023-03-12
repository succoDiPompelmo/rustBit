use sqlx::Connection;
use sqlx::PgConnection;

use super::peer_endpoint::PeerEndpoint;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrackedPeer {
    endpoint: String,
}

impl TrackedPeer {
    pub fn endpoint(&self) -> String {
        self.endpoint.to_string()
    }
}

pub async fn all_endpoints_by_hash(info_hash: Vec<u8>) -> Vec<String> {
    let mut db = db_conn().await;

    let query = "SELECT endpoint FROM tracked_peers WHERE info_hash = $1";

    let result = sqlx::query_as::<_, TrackedPeer>(query)
        .bind(info_hash)
        .fetch_all(&mut db)
        .await
        .unwrap_or_default();

    result
        .into_iter()
        .map(|peer| peer.endpoint())
        .collect::<Vec<String>>()
}

pub async fn insert_tracked_peers(peers: Vec<PeerEndpoint>, info_hash: &[u8]) {
    let mut db = db_conn().await;

    let query = "INSERT INTO tracked_peers
                        (info_hash, endpoint)
                        VALUES ($1, $2) ON CONFLICT (info_hash, endpoint) DO NOTHING";

    for peer in &peers {
        let peer_endpoint = format!("{}:{}", peer.ip(), peer.port());

        sqlx::query(query)
            .bind(info_hash.to_vec())
            .bind(peer_endpoint)
            .execute(&mut db)
            .await
            .unwrap();
    }
}

async fn db_conn() -> PgConnection {
    PgConnection::connect("postgres://postgres:password@localhost/rust_bit")
        .await
        .unwrap()
}
