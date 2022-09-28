use std::cmp;
use std::fs;
use std::fs::OpenOptions;
use std::os::unix::prelude::FileExt;

use crate::torrent::file::File;

#[derive(Debug, PartialEq)]
pub struct FileWriter {
    path: Vec<String>,
    start: u32,
    end: u32,
    piece: Vec<u8>,
}

impl FileWriter {
    fn new(path: Vec<String>, start: u32, end: u32, piece: Vec<u8>) -> FileWriter {
        return FileWriter {
            path,
            start,
            end,
            piece,
        };
    }

    // TODO: Add optional folder where to save output
    pub fn write_to_filesystem(&self) {
        let path_string = &self.path.join("/");
        let path = std::path::Path::new(path_string);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)
            .unwrap();

        file.write_at(&self.piece, self.start as u64);
    }
}

struct FileParser {
    piece_length: u32,
    piece_index: u32,
    offset: u32,
    default_piece_length: u32,
    start_file_index: u32,
    end_file_index: u32,
    right_file_boundary: u32,
}

impl FileParser {
    fn new(piece_length: u32, piece_index: u32, default_piece_length: u32) -> FileParser {
        let offset = piece_index * default_piece_length;
        FileParser {
            piece_index,
            piece_length,
            default_piece_length,
            offset,
            start_file_index: offset,
            right_file_boundary: 0,
            end_file_index: 0,
        }
    }

    fn update_start_file_index(&mut self) {
        self.start_file_index = self.end_file_index;
    }

    fn update_right_file_boundary(&mut self, file_length: u32) {
        self.right_file_boundary += file_length
    }

    fn update_end_file_index(&mut self) {
        self.end_file_index = cmp::min(self.right_file_boundary, self.offset + self.piece_length)
    }

    fn get_start_piece_index(&self) -> usize {
        (self.start_file_index - self.offset) as usize
    }

    fn get_end_piece_index(&self) -> usize {
        (self.end_file_index - self.offset) as usize
    }

    fn get_start_file_index(&self, file_length: u32) -> u32 {
        self.start_file_index - (self.right_file_boundary - file_length)
    }

    fn get_end_file_index(&self, file_length: u32) -> u32 {
        self.end_file_index - (self.right_file_boundary - file_length)
    }

    fn is_piece_in_file(&self) -> bool {
        self.offset < self.right_file_boundary
    }

    fn is_piece_finished(&self) -> bool {
        self.offset + self.piece_length > self.right_file_boundary
    }
}

pub fn get_file_writers(
    files: Vec<File>,
    piece: Vec<u8>,
    piece_index: u32,
    torrent_piece_length: u32,
) -> Vec<FileWriter> {
    let mut fileParser = FileParser::new(piece.len() as u32, piece_index, torrent_piece_length);
    let mut files_to_write = vec![];

    for file in files {
        let file_length = file.get_length() as u32;
        fileParser.update_right_file_boundary(file_length);

        if fileParser.is_piece_in_file() {
            fileParser.update_end_file_index();

            files_to_write.push(FileWriter::new(
                file.get_path(),
                fileParser.get_start_file_index(file_length),
                fileParser.get_end_file_index(file_length),
                piece[fileParser.get_start_piece_index()..fileParser.get_end_piece_index()]
                    .to_vec(),
            ));

            if fileParser.is_piece_finished() {
                fileParser.update_start_file_index()
            } else {
                return files_to_write;
            }
        }
    }
    return files_to_write;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_single_file_single_writer() {
        let files = vec![File::new(vec!["path".to_owned()], 32)];
        let result = get_file_writers(files, vec![0x00; 8], 2, 8);
        let expect = vec![FileWriter::new(
            vec!["path".to_owned()],
            16,
            24,
            vec![0x00; 8],
        )];
        assert_eq!(result, expect)
    }

    #[test]
    fn test_multiple_file_single_writer() {
        let files = vec![
            File::new(vec!["path".to_owned()], 32),
            File::new(vec!["path".to_owned()], 32),
        ];
        let result = get_file_writers(files, vec![0x00; 8], 1, 8);
        let expect = vec![FileWriter::new(
            vec!["path".to_owned()],
            8,
            16,
            vec![0x00; 8],
        )];
        assert_eq!(result, expect)
    }

    #[test]
    fn test_multiple_file_multiple_writer() {
        let files = vec![
            File::new(vec!["path".to_owned()], 33),
            File::new(vec!["to".to_owned()], 2),
            File::new(vec!["heaven".to_owned()], 30),
        ];
        let result = get_file_writers(files, vec![0x00; 8], 4, 8);
        let expect = vec![
            FileWriter::new(vec!["path".to_owned()], 32, 33, vec![0x00; 1]),
            FileWriter::new(vec!["to".to_owned()], 0, 2, vec![0x00; 2]),
            FileWriter::new(vec!["heaven".to_owned()], 0, 5, vec![0x00; 5]),
        ];

        assert_eq!(result, expect)
    }

    #[test]
    fn test_multiple_file_multiple_writer_edge_case() {
        let files = vec![
            File::new(vec!["path".to_owned()], 32),
            File::new(vec!["to".to_owned()], 2),
            File::new(vec!["heaven".to_owned()], 30),
        ];
        let result = get_file_writers(files, vec![0x00; 8], 4, 8);
        let expect = vec![
            FileWriter::new(vec!["to".to_owned()], 0, 2, vec![0x00; 2]),
            FileWriter::new(vec!["heaven".to_owned()], 0, 6, vec![0x00; 6]),
        ];

        assert_eq!(result, expect)
    }

    #[test]
    fn test_piece_smaller_than_piece_length() {
        let files = vec![File::new(vec!["path".to_owned()], 12)];
        // Piece length should be 8
        let result = get_file_writers(files, vec![0x00; 4], 1, 8);
        let expect = vec![FileWriter::new(
            vec!["path".to_owned()],
            8,
            12,
            vec![0x00; 4],
        )];

        assert_eq!(result, expect)
    }

    #[test]
    fn test_piece_too_small_ok() {
        let files = vec![File::new(vec!["path".to_owned()], 32)];
        let result = get_file_writers(files, vec![0x00; 2], 0, 2);
        let expect = vec![FileWriter::new(
            vec!["path".to_owned()],
            0,
            2,
            vec![0x00; 2],
        )];
        assert_eq!(result, expect)
    }

    #[test]
    fn test_big_file() {
        let files = vec![
            File::new(vec!["path".to_owned()], 1800),
            File::new(vec!["to".to_owned()], 1500),
            File::new(vec!["heaven".to_owned()], 6000),
        ];
        let result = get_file_writers(files, vec![0x00; 1700], 1, 1700);
        let expect = vec![
            FileWriter::new(vec!["path".to_owned()], 1700, 1800, vec![0x00; 100]),
            FileWriter::new(vec!["to".to_owned()], 0, 1500, vec![0x00; 1500]),
            FileWriter::new(vec!["heaven".to_owned()], 0, 100, vec![0x00; 100]),
        ];

        assert_eq!(result, expect)
    }
}
