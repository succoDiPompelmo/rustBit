use std::io::{Cursor, Error, ErrorKind, Read, Result, Write};

#[cfg(test)]
use std::mem::swap;

#[derive(Clone, Debug)]
pub struct MockStream {
    reader: Cursor<Vec<u8>>,
    writer: Cursor<Vec<u8>>,
}

impl Default for MockStream {
    fn default() -> Self {
        MockStream::new()
    }
}

fn new_cursor() -> Cursor<Vec<u8>> {
    Cursor::new(Vec::new())
}

impl MockStream {
    /// Create new empty stream
    pub fn new() -> MockStream {
        MockStream {
            reader: new_cursor(),
            writer: new_cursor(),
        }
    }

    /// Extract all bytes written by Write trait calls.
    #[cfg(test)]
    pub fn peek_bytes_written(&mut self) -> &Vec<u8> {
        self.writer.get_ref()
    }

    /// Extract all bytes written by Write trait calls.
    #[cfg(test)]
    pub fn peek_bytes_to_read(&mut self) -> usize {
        self.reader.get_ref().len()
    }

    /// Extract all bytes written by Write trait calls.
    #[cfg(test)]
    pub fn pop_bytes_written(&mut self) -> Vec<u8> {
        let mut result = Vec::new();
        swap(&mut result, self.writer.get_mut());
        self.writer.set_position(0);
        result
    }

    /// Provide data to be read by Read trait calls.
    #[cfg(test)]
    pub fn push_bytes_to_read(&mut self, bytes: &[u8]) {
        let avail = self.reader.get_ref().len();
        if self.reader.position() == avail as u64 {
            self.reader = new_cursor();
        }
        self.reader.get_mut().extend(bytes.iter().copied());
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
}

impl MockStream {
    pub fn peek(&mut self, buf: &mut [u8]) -> Result<usize> {
        let starting_position = self.reader.position();
        let output_buffer = self.reader.read(buf);
        self.reader.set_position(starting_position);
        output_buffer
    }
}

/// `FailingMockStream` mocks a stream which will fail upon read or write
///
/// # Examples
///
/// ```
/// use std::io::{Cursor, Read};
///
/// struct CountIo {}
///
/// impl CountIo {
///     fn read_data(&self, r: &mut Read) -> usize {
///         let mut count: usize = 0;
///         let mut retries = 3;
///
///         loop {
///             let mut buffer = [0; 5];
///             match r.read(&mut buffer) {
///                 Err(_) => {
///                     if retries == 0 { break; }
///                     retries -= 1;
///                 },
///                 Ok(0) => break,
///                 Ok(n) => count += n,
///             }
///         }
///         count
///     }
/// }
///
/// #[test]
/// fn test_io_retries() {
///     let mut c = Cursor::new(&b"1234"[..])
///             .chain(FailingMockStream::new(ErrorKind::Other, "Failing", 3))
///             .chain(Cursor::new(&b"5678"[..]));
///
///     let sut = CountIo {};
///     // this will fail unless read_data performs at least 3 retries on I/O errors
///     assert_eq!(8, sut.read_data(&mut c));
/// }
/// ```
#[derive(Clone)]
pub struct FailingMockStream {
    kind: ErrorKind,
    message: &'static str,
    repeat_count: i32,
}

impl FailingMockStream {
    /// Creates a FailingMockStream
    ///
    /// When `read` or `write` is called, it will return an error `repeat_count` times.
    /// `kind` and `message` can be specified to define the exact error.
    pub fn new(kind: ErrorKind, message: &'static str, repeat_count: i32) -> FailingMockStream {
        FailingMockStream {
            kind,
            message,
            repeat_count,
        }
    }

    fn error(&mut self) -> Result<usize> {
        if self.repeat_count == 0 {
            Ok(0)
        } else {
            if self.repeat_count > 0 {
                self.repeat_count -= 1;
            }
            Err(Error::new(self.kind, self.message))
        }
    }
}

impl Read for FailingMockStream {
    fn read(&mut self, _: &mut [u8]) -> Result<usize> {
        self.error()
    }
}

impl Write for FailingMockStream {
    fn write(&mut self, _: &[u8]) -> Result<usize> {
        self.error()
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_mock_stream_read() {
        let mut s = MockStream::new();
        s.push_bytes_to_read("abcd".as_bytes());
        let mut v = [11; 6];
        assert_eq!(s.read(v.as_mut()).unwrap(), 4);
        assert_eq!(v, [97, 98, 99, 100, 11, 11]);
    }

    #[test]
    fn test_mock_stream_pop_again() {
        let mut s = MockStream::new();
        s.write_all(b"abcd").unwrap();
        assert_eq!(s.pop_bytes_written(), b"abcd");
        s.write_all(b"efgh").unwrap();
        assert_eq!(s.pop_bytes_written(), b"efgh");
    }

    #[test]
    fn test_mock_stream_empty_and_fill() {
        let mut s = MockStream::new();
        let mut v = [11; 6];
        assert_eq!(s.read(v.as_mut()).unwrap(), 0);
        s.push_bytes_to_read("abcd".as_bytes());
        assert_eq!(s.read(v.as_mut()).unwrap(), 4);
        assert_eq!(s.read(v.as_mut()).unwrap(), 0);
    }

    #[test]
    fn test_mock_stream_read_lines() {
        let mut s = MockStream::new();
        s.push_bytes_to_read("abcd\r\ndcba\r\n".as_bytes());
        let first_line = s
            .bytes()
            .map(|c| c.unwrap())
            .take_while(|&c| c != b'\n')
            .collect::<Vec<u8>>();
        assert_eq!(first_line, (vec![97, 98, 99, 100, 13]));
    }

    #[test]
    fn test_failing_mock_stream_read() {
        let mut s =
            FailingMockStream::new(ErrorKind::BrokenPipe, "The dog ate the ethernet cable", 1);
        let mut v = [0; 4];
        let error = s.read(v.as_mut()).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::BrokenPipe);
        assert_eq!(error.to_string(), "The dog ate the ethernet cable");
        // after a single error, it will return Ok(0)
        assert_eq!(s.read(v.as_mut()).unwrap(), 0);
    }

    #[test]
    fn test_failing_mock_stream_chain() {
        let mut s1 = MockStream::new();
        s1.push_bytes_to_read("abcd".as_bytes());
        let s2 = FailingMockStream::new(ErrorKind::Other, "Failing", -1);

        let mut c = s1.chain(s2);
        let mut v = [0; 8];
        assert_eq!(c.read(v.as_mut()).unwrap(), 4);
        assert_eq!(c.read(v.as_mut()).unwrap_err().kind(), ErrorKind::Other);
        assert_eq!(c.read(v.as_mut()).unwrap_err().kind(), ErrorKind::Other);
    }

    #[test]
    fn test_failing_mock_stream_chain_interrupted() {
        let mut c = Cursor::new(&b"abcd"[..])
            .chain(FailingMockStream::new(
                ErrorKind::Interrupted,
                "Interrupted",
                5,
            ))
            .chain(Cursor::new(&b"ABCD"[..]));

        let mut v = [0; 8];
        c.read_exact(v.as_mut()).unwrap();
        assert_eq!(v, [0x61, 0x62, 0x63, 0x64, 0x41, 0x42, 0x43, 0x44]);
        assert_eq!(c.read(v.as_mut()).unwrap(), 0);
    }

    #[test]
    fn test_mock_stream_write() {
        let mut s = MockStream::new();
        assert_eq!(s.write("abcd".as_bytes()).unwrap(), 4);
        assert_eq!(s.pop_bytes_written().as_ref(), [97, 98, 99, 100]);
        assert!(s.pop_bytes_written().is_empty());
    }

    #[test]
    fn test_failing_mock_stream_write() {
        let mut s = FailingMockStream::new(ErrorKind::PermissionDenied, "Access denied", -1);
        let error = s.write("abcd".as_bytes()).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(error.to_string(), "Access denied");
        // it will keep failing
        s.write("abcd".as_bytes()).unwrap_err();
    }
}
