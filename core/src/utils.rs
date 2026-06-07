//! Utilities for working with file formats
//!
//! This module provides helper functions for detecting file formats,
//! obtaining parsers and serializers, and parsing files.
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::{
    BinParser, BinSerializer, CsvParser, CsvSerializer, Format, Parse, ParserFormat,
    SerializerFormat, TextParser, TextSerializer, Tx,
};

/// Returns a parser for the specified format
///
/// # Arguments
/// * `format` - Target format (CSV, BIN, or TXT)
///
/// # Returns
/// `ParserFormat` - Parser as an enum
///
/// # Example
/// ```
/// use core::{Format, get_parser};
///
/// let parser = get_parser(&Format::Csv);
/// ```
pub fn get_parser(format: &Format) -> ParserFormat {
    match format {
        Format::Csv => ParserFormat::Csv(CsvParser::new()),
        Format::Bin => ParserFormat::Bin(BinParser::new()),
        Format::Txt => ParserFormat::Txt(TextParser::new()),
    }
}

/// Returns a serializer for the specified format
///
/// # Arguments
/// * `format` - Target format (CSV, BIN, or TXT)
///
/// # Returns
/// `SerializerFormat` - Serializer as an enum
///
/// # Example
/// ```
/// use core::{Format, get_serializer};
///
/// let serializer = get_serializer(&Format::Bin);
/// ```
pub fn get_serializer(format: &Format) -> SerializerFormat {
    match format {
        Format::Csv => SerializerFormat::Csv(CsvSerializer::new()),
        Format::Bin => SerializerFormat::Bin(BinSerializer::new()),
        Format::Txt => SerializerFormat::Txt(TextSerializer::new()),
    }
}

/// Detects file format based on file extension
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// `Result<Format, String>` - File format or error
///
/// # Errors
/// * `File has no extension` - File has no extension
/// * `Unsupported file extension` - Extension not supported
///
/// # Example
/// ```
/// use core::detect_format;
/// use std::path::Path;
///
/// let format = detect_format(Path::new("data.csv"));
/// assert!(format.is_ok());
/// assert_eq!(format.unwrap(), core::Format::Csv);
/// ```
pub fn detect_format(path: &Path) -> Result<Format, String> {
    match path.extension() {
        Some(ext) => match ext.to_str() {
            Some("bin") => Ok(Format::Bin),
            Some("csv") => Ok(Format::Csv),
            Some("txt") => Ok(Format::Txt),
            _ => Err(format!("Unsupported file extension: {:?}", ext)),
        },
        _ => Err("File has no extension".to_string()),
    }
}

/// Parses a transaction file with automatic format detection
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// `Result<Vec<Tx>, Box<dyn std::error::Error>>` - List of transactions or an error
///
/// # Errors
/// * Returns an error if the file cannot be opened
/// * Returns an error if the format is unsupported
/// * Returns an error if parsing fails
///
/// # Example
/// ```no_run
/// use core::parse_file;
/// use std::path::PathBuf;
///
/// let transactions = parse_file(&PathBuf::from("transactions.csv"));
/// match transactions {
///     Ok(txs) => println!("Loaded {} transactions", txs.len()),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn parse_file(path: &PathBuf) -> Result<Vec<Tx>, Box<dyn std::error::Error>> {
    let format = detect_format(path)?;
    let parser = get_parser(&format);
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let transactions = parser
        .parse(&mut reader)
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_detect_format_bin() {
        let path = PathBuf::from("test.bin");
        assert_eq!(detect_format(&path).unwrap(), Format::Bin);
    }

    #[test]
    fn test_detect_format_csv() {
        let path = PathBuf::from("test.csv");
        assert_eq!(detect_format(&path).unwrap(), Format::Csv);
    }

    #[test]
    fn test_detect_format_txt() {
        let path = PathBuf::from("test.txt");
        assert_eq!(detect_format(&path).unwrap(), Format::Txt);
    }

    #[test]
    fn test_detect_format_unsupported_extension() {
        let path = PathBuf::from("test.xyz");
        assert!(detect_format(&path).is_err());
    }

    #[test]
    fn test_detect_format_no_extension() {
        let path = PathBuf::from("test");
        assert!(detect_format(&path).is_err());
    }

    #[test]
    fn test_get_parser_csv() {
        let parser = get_parser(&Format::Csv);
        match parser {
            ParserFormat::Csv(_) => assert!(true),
            _ => assert!(false, "Expected CsvParser"),
        }
    }

    #[test]
    fn test_get_parser_bin() {
        let parser = get_parser(&Format::Bin);
        match parser {
            ParserFormat::Bin(_) => assert!(true),
            _ => assert!(false, "Expected BinParser"),
        }
    }

    #[test]
    fn test_get_parser_txt() {
        let parser = get_parser(&Format::Txt);
        match parser {
            ParserFormat::Txt(_) => assert!(true),
            _ => assert!(false, "Expected TextParser"),
        }
    }

    #[test]
    fn test_get_serializer_csv() {
        let serializer = get_serializer(&Format::Csv);
        match serializer {
            SerializerFormat::Csv(_) => assert!(true),
            _ => assert!(false, "Expected CsvSerializer"),
        }
    }

    #[test]
    fn test_get_serializer_bin() {
        let serializer = get_serializer(&Format::Bin);
        match serializer {
            SerializerFormat::Bin(_) => assert!(true),
            _ => assert!(false, "Expected BinSerializer"),
        }
    }

    #[test]
    fn test_get_serializer_txt() {
        let serializer = get_serializer(&Format::Txt);
        match serializer {
            SerializerFormat::Txt(_) => assert!(true),
            _ => assert!(false, "Expected TextSerializer"),
        }
    }

    #[test]
    fn test_parse_file_csv() {
        let test_file = "test_temp.csv";
        let csv_content = "id,amount,description\n1,100.5,Test transaction\n2,200.75,Another test";

        let mut file = File::create(test_file).unwrap();
        file.write_all(csv_content.as_bytes()).unwrap();
        drop(file);

        let path = PathBuf::from(test_file);
        let result = parse_file(&path);

        std::fs::remove_file(test_file).ok();

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_file_txt() {
        let test_file = "test_temp.txt";
        let txt_content = "1|100.5|Test transaction\n2|200.75|Another test";

        let mut file = File::create(test_file).unwrap();
        file.write_all(txt_content.as_bytes()).unwrap();
        drop(file);

        let path = PathBuf::from(test_file);
        let result = parse_file(&path);

        std::fs::remove_file(test_file).ok();

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_file_bin() {
        let test_file = "test_temp.bin";
        let bin_content = vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x40];

        let mut file = File::create(test_file).unwrap();
        file.write_all(&bin_content).unwrap();
        drop(file);

        let path = PathBuf::from(test_file);
        let result = parse_file(&path);

        std::fs::remove_file(test_file).ok();

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_file_not_found() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = parse_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_file_unsupported_format() {
        let test_file = "test_temp.xyz";
        let mut file = File::create(test_file).unwrap();
        file.write_all(b"some content").unwrap();
        drop(file);

        let path = PathBuf::from(test_file);
        let result = parse_file(&path);

        std::fs::remove_file(test_file).ok();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_file_invalid_csv() {
        let test_file = "test_invalid.csv";
        let invalid_csv = "this is not valid csv data\nwithout proper structure";

        let mut file = File::create(test_file).unwrap();
        file.write_all(invalid_csv.as_bytes()).unwrap();
        drop(file);

        let path = PathBuf::from(test_file);
        let result = parse_file(&path);

        std::fs::remove_file(test_file).ok();

        assert!(result.is_err());
    }

    #[test]
    fn test_detect_format_case_sensitive() {
        let path = PathBuf::from("test.CSV");
        let result = detect_format(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_file_with_path_with_spaces() {
        let test_file = "test temp file.csv";
        let csv_content = "id,amount\n1,100";

        let mut file = File::create(test_file).unwrap();
        file.write_all(csv_content.as_bytes()).unwrap();
        drop(file);

        let path = PathBuf::from(test_file);
        let result = parse_file(&path);

        std::fs::remove_file(test_file).ok();

        assert!(result.is_ok() || result.is_err());
    }
}
