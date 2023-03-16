use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time::Duration;
use std::{thread, time, vec};

use log::error;

use crate::common::mock_stream::MockStream;

#[derive(Debug)]
pub enum StreamInterface {
    #[allow(dead_code)]
    Mocked(MockStream),
    Tcp(TcpStream),
    #[allow(dead_code)]
    Nothing(),
}

impl io::Read for StreamInterface {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            StreamInterface::Mocked(ref mut s) => s.read(buf),
            StreamInterface::Tcp(ref mut s) => s.read(buf),
            StreamInterface::Nothing() => Ok(0),
        }
    }
}

impl io::Write for StreamInterface {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            StreamInterface::Mocked(ref mut s) => s.write(buf),
            StreamInterface::Tcp(ref mut s) => s.write(buf),
            StreamInterface::Nothing() => Ok(0),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            StreamInterface::Mocked(ref mut s) => s.flush(),
            StreamInterface::Tcp(ref mut s) => s.flush(),
            StreamInterface::Nothing() => Ok(()),
        }
    }
}

impl StreamInterface {
    pub fn connect(endpoint: &str, mocked: bool) -> Result<Self, &'static str> {
        if mocked {
            return Ok(StreamInterface::Mocked(MockStream::new()));
        }

        let server: std::net::SocketAddr =
            endpoint.parse().expect("Unable to parse socket address");
        let connect_timeout = Duration::from_secs(1);
        let stream =
            TcpStream::connect_timeout(&server, connect_timeout).map_err(|_| "Connection error")?;
        Ok(StreamInterface::Tcp(stream))
    }

    fn peek(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            StreamInterface::Mocked(ref mut s) => s.peek(buf),
            StreamInterface::Tcp(ref mut s) => s.peek(buf),
            StreamInterface::Nothing() => Ok(0),
        }
    }

    fn set_read_timeout(&mut self, duration: Option<Duration>) -> io::Result<()> {
        match *self {
            StreamInterface::Mocked(_) => Ok(()),
            StreamInterface::Tcp(ref mut s) => s.set_read_timeout(duration),
            StreamInterface::Nothing() => Ok(()),
        }
    }
}

fn has_messages(stream: &mut StreamInterface) -> bool {
    let mut buffer: [u8; 1] = [0x00; 1];

    match stream.peek(&mut buffer) {
        Ok(0) => false,
        Ok(_) => true,
        Err(_) => false,
    }
}

fn peek_and_read(stream: &mut StreamInterface, body: &mut [u8], length: usize) -> bool {
    stream.peek(body).unwrap_or(0) == length && stream.read(body).unwrap_or(0) == length
}

// Here we should try to return a Result to signal the presence of errors
pub fn read_stream(stream: &mut StreamInterface) -> Option<(Vec<u8>, u8, u32)> {
    if stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .is_err()
    {
        return None;
    }

    if !has_messages(stream) {
        return None;
    }

    let mut buffer: [u8; 68] = [0x00; 68];
    if stream.peek(&mut buffer).ok() == Some(68) && buffer[0] == 0x13 {
        if stream.read(&mut buffer).is_err() {
            return None;
        }
        return Some((buffer.to_vec(), 19, 68));
    }

    let mut buffer: [u8; 4] = [0x00; 4];
    if stream.read(&mut buffer).is_err() {
        return None;
    }
    let mut length = u32::from_be_bytes(buffer);

    let mut buffer: [u8; 1] = [0x00; 1];
    if stream.read(&mut buffer).is_err() {
        return None;
    }
    let id = buffer[0];

    if length == 0 {
        return Some((vec![], id, 0));
    } else {
        length -= 1
    }

    let mut body = vec![0; length as usize];
    for retry in 1..40 {
        if peek_and_read(stream, &mut body, length as usize) {
            return Some((body, id, length));
        }
        thread::sleep(time::Duration::from_millis(5 * retry));
    }
    None
}

pub fn write_stream(stream: &mut StreamInterface, buffer: &[u8]) {
    match stream.write_all(buffer) {
        Ok(_) => (),
        Err(err) => error!("Error {:?} in writing buffer {:?}", err, buffer),
    }
}

pub fn send_metadata_handshake_request(stream: &mut StreamInterface) -> Result<(), &'static str> {
    let content = [
        &60_u32.to_be_bytes(),
        [0x14].as_slice(),
        [0x00].as_slice(),
        "d1:md11:ut_metadatai1e6:ut_pexi2ee13:metadata_sizei28282ee".as_bytes(),
    ]
    .concat();

    stream
        .write_all(&content)
        .map_err(|_| "Error in metadata request")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_has_messages() {
        let mut s = MockStream::new();
        s.push_bytes_to_read([1].as_slice());
        let mut e = StreamInterface::Mocked(s);

        assert_eq!(has_messages(&mut e), true);
    }

    #[test]
    fn test_has_no_messages() {
        let mut s = MockStream::new();
        s.push_bytes_to_read([].as_slice());
        let mut e = StreamInterface::Mocked(s);

        assert_eq!(has_messages(&mut e), false);
    }

    #[test]
    fn test_peek_and_read() {
        let mut s = MockStream::new();
        s.push_bytes_to_read([1].as_slice());
        let mut e = StreamInterface::Mocked(s);

        let length = 1;
        let mut buffer = vec![0; length as usize];

        let expect: Vec<u8> = [1].to_vec();

        assert_eq!(peek_and_read(&mut e, &mut buffer, length), true);
        assert_eq!(buffer, expect)
    }

    #[test]
    fn test_peek_and_read_empty() {
        let mut s = MockStream::new();
        s.push_bytes_to_read([].as_slice());
        let mut e = StreamInterface::Mocked(s);

        let length = 0;
        let mut buffer = vec![0; length as usize];

        let expect: Vec<u8> = [].to_vec();

        assert_eq!(peek_and_read(&mut e, &mut buffer, length), true);
        assert_eq!(buffer, expect)
    }

    #[test]
    fn test_peek_and_read_less_than_available() {
        let mut s = MockStream::new();
        s.push_bytes_to_read([1, 2, 3, 4, 5, 6].as_slice());
        let mut e = StreamInterface::Mocked(s);

        let length = 3;
        let mut buffer = vec![0; length as usize];

        let expect: Vec<u8> = [1, 2, 3].to_vec();

        assert_eq!(peek_and_read(&mut e, &mut buffer, length), true);
        assert_eq!(buffer, expect)
    }

    #[test]
    fn test_read_stream_no_body() {
        let mut s = MockStream::new();
        s.push_bytes_to_read([0, 0, 0, 1, 5].as_slice());
        let mut e = StreamInterface::Mocked(s);

        let expect: (Vec<u8>, u8, u32) = ([].to_vec(), 5, 0);
        assert_eq!(read_stream(&mut e), Some(expect));
    }

    #[test]
    fn test_read_stream_with_body() {
        let mut s = MockStream::new();
        s.push_bytes_to_read([0, 0, 0, 6, 9, 1, 2, 3, 4, 5].as_slice());
        let mut e = StreamInterface::Mocked(s);

        let expect: (Vec<u8>, u8, u32) = ([1, 2, 3, 4, 5].to_vec(), 9, 5);
        assert_eq!(read_stream(&mut e), Some(expect));
    }

    #[test]
    fn test_read_stream_empty() {
        let mut s = MockStream::new();
        s.push_bytes_to_read([].as_slice());
        let mut e = StreamInterface::Mocked(s);

        assert_eq!(read_stream(&mut e), None);
    }

    #[test]
    fn test_read_stream_handshake() {
        let mut s = MockStream::new();
        let mut handshake = [0x00; 68];
        handshake[0] = 0x13;
        s.push_bytes_to_read(handshake.as_slice());
        let mut e = StreamInterface::Mocked(s);

        let expect: (Vec<u8>, u8, u32) = (handshake.to_vec(), 19, 68);
        assert_eq!(read_stream(&mut e), Some(expect));
    }
}
