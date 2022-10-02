#[derive(Debug, PartialEq)]
pub struct Magnet {
    info_hash: Vec<u8>,
}

impl Magnet {
    pub fn get_info_hash(&self) -> Vec<u8> {
        self.info_hash.to_vec()
    }
}

fn verify_info_hash(magnet_uri: &Vec<u8>) -> bool {
    return magnet_uri.len() < 60 || &magnet_uri[..8] != "magnet:?".as_bytes();
}

// Given a string representation of an hex character we obtain the hex byte representation.
// The hex character A is converted to ascii decimal 65 that correspond to hex decimal
// 10. It's a simple subtraction in order to project the ascii decimal character to
// the hex byte representation.
// hex -> ascii -> hex (byte)
// A   -> 65    -> 10
// 9   -> 57    -> 9
// 8   -> 56    -> 8
fn byte_to_hex(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte - 55
    } else if byte.is_ascii_lowercase() {
        byte - 87
    } else {
        byte - 48
    }
}

fn hex_decode(info_hash_hex_encoded: &Vec<u8>) -> Vec<u8> {
    info_hash_hex_encoded
        .chunks_exact(2)
        .map(|chunk| {
            let a = byte_to_hex(chunk[0]) << 4;
            let b = byte_to_hex(chunk[1]);
            a | b
        })
        .collect::<Vec<u8>>()
}

fn get_info_hash(magnet_uri: &Vec<u8>) -> Vec<u8> {
    if magnet_uri[60] == b'&' {
        hex_decode(&magnet_uri[20..60].to_vec())
    } else {
        magnet_uri[20..52].to_vec()
    }
}

pub fn parse_magnet(magnet_uri: Vec<u8>) -> Result<Magnet, &'static str> {
    if verify_info_hash(&magnet_uri) {
        return Err("No magnet uri found");
    }

    let info_hash = get_info_hash(&magnet_uri);

    Ok(Magnet { info_hash })
}

#[cfg(test)]
mod test {
    use super::*;
    use base32::Alphabet::RFC4648;

    #[test]
    fn parse_magnet_hex_encoded_test() {
        let magnet = "magnet:?xt=urn:btih:A6e449c2281e62edbf8cdb447413ca288cf0e568&dn=Top.Gun.Maverick.2022.KORSUB.IMAX.1080p.WEBRip.AAC2.0.x264-SHITBOX&tr=http%3A%2F%2Ftracker.trackerfix.com%3A80%2Fannounce&tr=udp%3A%2F%2F9.rarbg.me%3A2730&tr=udp%3A%2F%2F9.rarbg.to%3A2800&tr=udp%3A%2F%2Ftracker.tallpenguin.org%3A15780&tr=udp%3A%2F%2Ftracker.thinelephant.org%3A12720";
        let result = parse_magnet(magnet.as_bytes().to_vec());

        assert_eq!(
            result.unwrap(),
            Magnet {
                info_hash: vec![
                    166, 228, 73, 194, 40, 30, 98, 237, 191, 140, 219, 68, 116, 19, 202, 40, 140,
                    240, 229, 104
                ]
            }
        )
    }

    #[test]
    fn parse_magnet_base32_encoded_test() {
        let magnet = "magnet:?xt=urn:btih:G27L4RSWJP5UK2XU6U35LCRHEEV57INR&dn=See.S03E01.WEBRip.x264-ION10&tr=http%3A%2F%2Ftracker.trackerfix.com%3A80%2Fannounce";
        let result = parse_magnet(magnet.as_bytes().to_vec());

        let info_hash = [
            54, 190, 190, 70, 86, 75, 251, 69, 106, 244, 245, 55, 213, 138, 39, 33, 43, 223, 161,
            177,
        ];

        assert_eq!(
            result.unwrap(),
            Magnet {
                info_hash: base32::encode(RFC4648 { padding: false }, &info_hash)
                    .as_bytes()
                    .to_vec()
            }
        )
    }

    #[test]
    fn parse_magnet_error_test() {
        let magnet = "magneto:?00000000000000000000000000000000000000000000000000000000";
        let result = parse_magnet(magnet.as_bytes().to_vec());

        assert!(result.is_err())
    }
}
