use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn generate_peer_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect()
}
