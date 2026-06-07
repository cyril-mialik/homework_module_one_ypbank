//! Error handling for the core library
//!
//! This module defines all error types used throughout the core library,
//! including format-specific errors (binary, CSV, text) and general
//! parsing errors for transactions, types, and statuses.

use std::fmt;

/// Error indicating an invalid transaction type string
#[derive(Debug)]
pub struct InvalidTxType(pub String);

/// Error indicating an invalid status string
#[derive(Debug)]
pub struct InvalidStatus(pub String);

/// Error indicating a failure to parse a numeric field
pub struct ParseNumberError {
    /// Name of the field that failed to parse
    pub field: String,
    /// Raw string value that couldn't be parsed
    pub raw: String,
}

// Error trait implementations for all error types
impl std::error::Error for Error {}
impl std::error::Error for BinError {}
impl std::error::Error for CsvError {}
impl std::error::Error for TextError {}

/// Main error enum for the core library
///
/// Wraps all possible errors that can occur during parsing and serialization
pub enum Error {
    /// I/O operation error
    Io(std::io::Error),
    /// Binary format error
    Bin(BinError),
    /// CSV format error
    Csv(CsvError),
    /// Text format error
    Text(TextError),
    /// Invalid transaction type error
    InvalidTxType(InvalidTxType),
    /// Invalid status error
    InvalidStatus(InvalidStatus),
    /// Number parsing error
    ParseNumberError(ParseNumberError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Bin(e) => write!(f, "Binary format error: {}", e),
            Error::Csv(e) => write!(f, "CSV format error: {}", e),
            Error::Text(e) => write!(f, "Text format error: {}", e),
            Error::InvalidTxType(e) => write!(f, "Invalid transaction type: {}", e.0),
            Error::InvalidStatus(e) => write!(f, "Invalid status: {}", e.0),
            Error::ParseNumberError(e) => write!(f, "Failed to parse {}: '{}'", e.field, e.raw),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "Io({:?})", e),
            Error::Bin(e) => write!(f, "Bin({:?})", e),
            Error::Csv(e) => write!(f, "Csv({:?})", e),
            Error::Text(e) => write!(f, "Text({:?})", e),
            Error::InvalidTxType(e) => write!(f, "InvalidTxType({:?})", e.0),
            Error::InvalidStatus(e) => write!(f, "InvalidStatus({:?})", e.0),
            Error::ParseNumberError(e) => write!(
                f,
                "ParseNumberError {{ field: {:?}, raw: {:?} }}",
                e.field, e.raw
            ),
        }
    }
}

// From trait implementations for automatic error conversion
impl From<BinError> for Error {
    fn from(err: BinError) -> Self {
        Error::Bin(err)
    }
}

impl From<CsvError> for Error {
    fn from(err: CsvError) -> Self {
        Error::Csv(err)
    }
}

impl From<TextError> for Error {
    fn from(err: TextError) -> Self {
        Error::Text(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<InvalidTxType> for Error {
    fn from(err: InvalidTxType) -> Self {
        Error::InvalidTxType(err)
    }
}

impl From<InvalidStatus> for Error {
    fn from(err: InvalidStatus) -> Self {
        Error::InvalidStatus(err)
    }
}

impl From<ParseNumberError> for Error {
    fn from(err: ParseNumberError) -> Self {
        Error::ParseNumberError(err)
    }
}

/// Binary format-specific errors
pub enum BinError {
    /// Invalid magic bytes at the beginning of a record
    InvalidMagic([u8; 4]),
    /// Record size mismatch between header and actual data
    InvalidRecordSize { expected: u32, actual: u32 },
    /// Description length doesn't match the declared length
    DescriptionLengthMismatch { expected: u32, actual: usize },
    /// Invalid transaction type byte value
    InvalidTxType(u8),
    /// Invalid status byte value
    InvalidStatus(u8),
}

impl fmt::Display for BinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinError::InvalidMagic(magic) => write!(f, "Invalid magic bytes: {:02X?}", magic),
            BinError::InvalidRecordSize { expected, actual } => write!(
                f,
                "Invalid record size: expected {}, got {}",
                expected, actual
            ),
            BinError::DescriptionLengthMismatch { expected, actual } => write!(
                f,
                "Description length mismatch: expected {}, got {}",
                expected, actual
            ),
            BinError::InvalidTxType(byte) => write!(f, "Invalid transaction type byte: {}", byte),
            BinError::InvalidStatus(byte) => write!(f, "Invalid status byte: {}", byte),
        }
    }
}

impl fmt::Debug for BinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinError::InvalidMagic(magic) => write!(f, "InvalidMagic({:?})", magic),
            BinError::InvalidRecordSize { expected, actual } => write!(
                f,
                "InvalidRecordSize {{ expected: {}, actual: {} }}",
                expected, actual
            ),
            BinError::DescriptionLengthMismatch { expected, actual } => write!(
                f,
                "DescriptionLengthMismatch {{ expected: {}, actual: {} }}",
                expected, actual
            ),
            BinError::InvalidTxType(byte) => write!(f, "InvalidTxType({})", byte),
            BinError::InvalidStatus(byte) => write!(f, "InvalidStatus({})", byte),
        }
    }
}

/// CSV format-specific errors
pub enum CsvError {
    /// Invalid or missing CSV header
    InvalidHeader,
    /// Wrong number of fields in a CSV row
    InvalidFieldCount { expected: usize, actual: usize },
    /// Invalid description field format
    InvalidDescriptionFormat(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::InvalidHeader => write!(f, "Invalid CSV header"),
            CsvError::InvalidFieldCount { expected, actual } => write!(
                f,
                "Invalid field count: expected {}, got {}",
                expected, actual
            ),
            CsvError::InvalidDescriptionFormat(field) => {
                write!(f, "Invalid description format: {}", field)
            }
        }
    }
}

impl fmt::Debug for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::InvalidHeader => write!(f, "InvalidHeader"),
            CsvError::InvalidFieldCount { expected, actual } => write!(
                f,
                "InvalidFieldCount {{ expected: {}, actual: {} }}",
                expected, actual
            ),
            CsvError::InvalidDescriptionFormat(field) => {
                write!(f, "InvalidDescriptionFormat({:?})", field)
            }
        }
    }
}

/// Text format-specific errors
pub enum TextError {
    /// Required field is missing
    MissingField(String),
    /// Duplicate field found in a record
    DuplicateField(String),
    /// Invalid line format (doesn't contain a colon)
    InvalidLineFormat(String),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextError::MissingField(field) => write!(f, "Missing required field: {}", field),
            TextError::DuplicateField(field) => write!(f, "Duplicate field: {}", field),
            TextError::InvalidLineFormat(line) => write!(f, "Invalid line format: {}", line),
        }
    }
}

impl fmt::Debug for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextError::MissingField(field) => write!(f, "MissingField({:?})", field),
            TextError::DuplicateField(field) => write!(f, "DuplicateField({:?})", field),
            TextError::InvalidLineFormat(line) => write!(f, "InvalidLineFormat({:?})", line),
        }
    }
}
