//! Core library for transaction processing
//!
//! This library provides functionality for parsing and serializing
//! transaction data in three formats: CSV, Binary, and Text.
//!
//! # Features
//! - Automatic format detection
//! - Support for multiple file formats
//! - Type-safe transaction handling
//! - Comprehensive error handling
//!
//! # Example
//! ```no_run
//! use core::{parse_file, Format};
//! use std::path::PathBuf;
//!
//! let transactions = parse_file(&PathBuf::from("data.csv")).unwrap();
//! println!("Loaded {} transactions", transactions.len());
//! ```

mod bin_format;
mod constants;
mod csv_format;
mod error;
mod txt_format;
mod utils;

use std::io::{Read, Write};
use std::str::FromStr;

pub use bin_format::{BinParser, BinSerializer};
pub use constants::{
    BIN, CSV, DEPOSIT_TYPE, DEPOSIT_TYPE_INDEX, FAILURE_STATUS, FAILURE_STATUS_INDEX,
    PENDING_STATUS, PENDING_STATUS_INDEX, SUCCESS_STATUS, SUCCESS_STATUS_INDEX, TEXT,
    TRANSFER_TYPE, TRANSFER_TYPE_INDEX, TXT, WITHDRAWAL_TYPE, WITHDRAWAL_TYPE_INDEX,
};
pub use csv_format::{CsvParser, CsvSerializer};
pub use error::*;
pub use txt_format::{TextParser, TextSerializer};
pub use utils::{detect_format, get_parser, get_serializer, parse_file};

/// Trait for parsing transaction data from a reader
///
/// Implementations of this trait parse transaction data from various formats
/// and return a vector of `Tx` objects.
///
/// # Example
/// ```
/// use core::{Parse, CsvParser, Tx};
/// use std::io::Cursor;
///
/// let data = b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\
///              1,DEPOSIT,0,100,1000,1633036860000,SUCCESS,\"Test\"";
/// let mut reader = Cursor::new(data);
/// let parser = CsvParser::new();
/// let transactions = parser.parse(&mut reader).unwrap();
/// assert_eq!(transactions.len(), 1);
/// ```
pub trait Parse {
    /// Parses transaction data from a reader
    ///
    /// # Arguments
    /// * `reader` - A readable source of data
    ///
    /// # Returns
    /// A vector of parsed transactions or an error
    fn parse<R: Read>(&self, reader: &mut R) -> Result<Vec<Tx>, Error>;
}

/// Trait for serializing transaction data to a writer
///
/// Implementations of this trait serialize a slice of `Tx` objects
/// into a specific format and write them to a writer.
///
/// # Example
/// ```
/// use core::{Serialize, CsvSerializer, Tx, TxId, TxType, TxFromUserId, TxToUserId,
///            TxDescription, TxStatus, TxTimestamp, TxAmount};
///
/// let tx = Tx::new(
///     TxId(1),
///     TxType::Deposit,
///     TxFromUserId(0),
///     TxToUserId(100),
///     TxDescription("Test".to_string()),
///     TxStatus::Success,
///     TxTimestamp(1633036860000),
///     TxAmount(1000),
/// );
/// let mut buffer = Vec::new();
/// let serializer = CsvSerializer::new();
/// serializer.serialize(&mut buffer, &[tx]).unwrap();
/// assert!(!buffer.is_empty());
/// ```
pub trait Serialize {
    /// Serializes transaction data to a writer
    ///
    /// # Arguments
    /// * `writer` - A writable destination for the serialized data
    /// * `txs` - A slice of transactions to serialize
    ///
    /// # Returns
    /// `Ok(())` on success or an error
    fn serialize<W: Write>(&self, writer: &mut W, txs: &[Tx]) -> Result<(), Error>;
}

/// Supported file formats
#[derive(Debug, PartialEq)]
pub enum Format {
    /// CSV (Comma-Separated Values) format
    Csv,
    /// Custom binary format
    Bin,
    /// Text format with key-value pairs
    Txt,
}

impl FromStr for Format {
    type Err = String;

    /// Parses a format from a string (case-insensitive)
    ///
    /// # Supported values
    /// * "csv" -> Format::Csv
    /// * "bin" -> Format::Bin
    /// * "txt" or "text" -> Format::Txt
    ///
    /// # Example
    /// ```
    /// use core::Format;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Format::from_str("csv").unwrap(), Format::Csv);
    /// assert_eq!(Format::from_str("BIN").unwrap(), Format::Bin);
    /// assert_eq!(Format::from_str("txt").unwrap(), Format::Txt);
    /// assert_eq!(Format::from_str("text").unwrap(), Format::Txt);
    /// assert!(Format::from_str("unknown").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            CSV => Ok(Format::Csv),
            BIN => Ok(Format::Bin),
            TXT | TEXT => Ok(Format::Txt),
            _ => Err(format!("Unknown format: '{}'. Supported: csv, bin, txt", s)),
        }
    }
}

/// Parser enum that delegates to format-specific parsers
pub enum ParserFormat {
    /// CSV parser wrapper
    Csv(CsvParser),
    /// Binary parser wrapper
    Bin(BinParser),
    /// Text parser wrapper
    Txt(TextParser),
}

impl Parse for ParserFormat {
    /// Delegates parsing to the wrapped parser
    fn parse<R: std::io::Read>(&self, reader: &mut R) -> Result<Vec<Tx>, Error> {
        match self {
            ParserFormat::Csv(p) => p.parse(reader),
            ParserFormat::Bin(p) => p.parse(reader),
            ParserFormat::Txt(p) => p.parse(reader),
        }
    }
}

/// Serializer enum that delegates to format-specific serializers
pub enum SerializerFormat {
    /// CSV serializer wrapper
    Csv(CsvSerializer),
    /// Binary serializer wrapper
    Bin(BinSerializer),
    /// Text serializer wrapper
    Txt(TextSerializer),
}

impl Serialize for SerializerFormat {
    /// Delegates serialization to the wrapped serializer
    fn serialize<W: std::io::Write>(
        &self,
        writer: &mut W,
        transactions: &[Tx],
    ) -> Result<(), Error> {
        match self {
            SerializerFormat::Csv(s) => s.serialize(writer, transactions),
            SerializerFormat::Bin(s) => s.serialize(writer, transactions),
            SerializerFormat::Txt(s) => s.serialize(writer, transactions),
        }
    }
}

/// Transaction type enumeration
#[derive(Debug, PartialEq, Clone)]
pub enum TxType {
    /// Deposit operation (adding funds)
    Deposit,
    /// Transfer operation between users
    Transfer,
    /// Withdrawal operation (removing funds)
    Withdrawal,
}

impl FromStr for TxType {
    type Err = InvalidTxType;

    /// Parses a transaction type from a string
    ///
    /// # Example
    /// ```
    /// use core::TxType;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(TxType::from_str("DEPOSIT").unwrap(), TxType::Deposit);
    /// assert_eq!(TxType::from_str("TRANSFER").unwrap(), TxType::Transfer);
    /// assert_eq!(TxType::from_str("WITHDRAWAL").unwrap(), TxType::Withdrawal);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            DEPOSIT_TYPE => Ok(TxType::Deposit),
            TRANSFER_TYPE => Ok(TxType::Transfer),
            WITHDRAWAL_TYPE => Ok(TxType::Withdrawal),
            _ => Err(InvalidTxType(s.to_string())),
        }
    }
}

impl TryFrom<u8> for TxType {
    type Error = InvalidTxType;

    /// Converts a byte value to a transaction type (for binary format)
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            DEPOSIT_TYPE_INDEX => Ok(TxType::Deposit),
            TRANSFER_TYPE_INDEX => Ok(TxType::Transfer),
            WITHDRAWAL_TYPE_INDEX => Ok(TxType::Withdrawal),
            unknown => Err(InvalidTxType(format!("Unknown tx type byte: {}", unknown))),
        }
    }
}

impl From<&TxType> for u8 {
    /// Converts a transaction type to a byte value (for binary format)
    fn from(tx_type: &TxType) -> Self {
        match tx_type {
            TxType::Deposit => DEPOSIT_TYPE_INDEX,
            TxType::Transfer => TRANSFER_TYPE_INDEX,
            TxType::Withdrawal => WITHDRAWAL_TYPE_INDEX,
        }
    }
}

/// Transaction status enumeration
#[derive(Debug, PartialEq, Clone)]
pub enum TxStatus {
    /// Transaction is pending processing
    Pending,
    /// Transaction completed successfully
    Success,
    /// Transaction failed
    Failure,
}

impl FromStr for TxStatus {
    type Err = InvalidStatus;

    /// Parses a status from a string
    ///
    /// # Example
    /// ```
    /// use core::TxStatus;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(TxStatus::from_str("PENDING").unwrap(), TxStatus::Pending);
    /// assert_eq!(TxStatus::from_str("SUCCESS").unwrap(), TxStatus::Success);
    /// assert_eq!(TxStatus::from_str("FAILURE").unwrap(), TxStatus::Failure);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            PENDING_STATUS => Ok(TxStatus::Pending),
            SUCCESS_STATUS => Ok(TxStatus::Success),
            FAILURE_STATUS => Ok(TxStatus::Failure),
            _ => Err(InvalidStatus(s.to_string())),
        }
    }
}

impl TryFrom<u8> for TxStatus {
    type Error = InvalidStatus;

    /// Converts a byte value to a status (for binary format)
    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            SUCCESS_STATUS_INDEX => Ok(TxStatus::Success),
            FAILURE_STATUS_INDEX => Ok(TxStatus::Failure),
            PENDING_STATUS_INDEX => Ok(TxStatus::Pending),
            unknown => Err(InvalidStatus(format!("Unknown status byte: {}", unknown))),
        }
    }
}

impl From<&TxStatus> for u8 {
    /// Converts a status to a byte value (for binary format)
    fn from(status: &TxStatus) -> Self {
        match status {
            TxStatus::Success => SUCCESS_STATUS_INDEX,
            TxStatus::Failure => FAILURE_STATUS_INDEX,
            TxStatus::Pending => PENDING_STATUS_INDEX,
        }
    }
}

// Type aliases for transaction fields
#[derive(Debug, PartialEq, Clone)]
pub struct TxId(pub u64);

#[derive(Debug, PartialEq, Clone)]
pub struct TxFromUserId(pub u64);

#[derive(Debug, PartialEq, Clone)]
pub struct TxToUserId(pub u64);

#[derive(Debug, PartialEq, Clone)]
pub struct TxAmount(pub i64);

#[derive(Debug, PartialEq, Clone)]
pub struct TxTimestamp(pub u64);

#[derive(Debug, PartialEq, Clone)]
pub struct TxDescription(pub String);

/// Main transaction structure
///
/// Represents a financial transaction with all associated metadata.
///
/// # Example
/// ```
/// use core::{Tx, TxId, TxType, TxFromUserId, TxToUserId,
///            TxDescription, TxStatus, TxTimestamp, TxAmount};
///
/// let tx = Tx::new(
///     TxId(1),
///     TxType::Deposit,
///     TxFromUserId(0),
///     TxToUserId(100),
///     TxDescription("Test transaction".to_string()),
///     TxStatus::Success,
///     TxTimestamp(1633036860000),
///     TxAmount(1000),
/// );
///
/// assert_eq!(tx.tx_id.0, 1);
/// assert_eq!(tx.tx_type, TxType::Deposit);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Tx {
    /// Transaction type (Deposit, Transfer, Withdrawal)
    pub tx_type: TxType,
    /// Unique transaction identifier
    pub tx_id: TxId,
    /// Transaction amount (positive for deposits/transfers, negative for withdrawals)
    pub amount: TxAmount,
    /// Transaction status (Pending, Success, Failure)
    pub status: TxStatus,
    /// Source user identifier (0 for deposits)
    pub from_user_id: TxFromUserId,
    /// Transaction timestamp (Unix milliseconds)
    pub timestamp: TxTimestamp,
    /// Transaction description
    pub description: TxDescription,
    /// Destination user identifier
    pub to_user_id: TxToUserId,
}

impl Tx {
    /// Creates a new transaction
    ///
    /// # Arguments
    /// * `tx_id` - Unique transaction identifier
    /// * `tx_type` - Type of transaction
    /// * `from_user_id` - Source user identifier
    /// * `to_user_id` - Destination user identifier
    /// * `description` - Transaction description
    /// * `status` - Transaction status
    /// * `timestamp` - Transaction timestamp
    /// * `amount` - Transaction amount
    ///
    /// # Example
    /// ```
    /// use core::{Tx, TxId, TxType, TxFromUserId, TxToUserId,
    ///            TxDescription, TxStatus, TxTimestamp, TxAmount};
    ///
    /// let tx = Tx::new(
    ///     TxId(42),
    ///     TxType::Transfer,
    ///     TxFromUserId(10),
    ///     TxToUserId(20),
    ///     TxDescription("Payment".to_string()),
    ///     TxStatus::Pending,
    ///     TxTimestamp(1633036860000),
    ///     TxAmount(500),
    /// );
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tx_id: TxId,
        tx_type: TxType,
        from_user_id: TxFromUserId,
        to_user_id: TxToUserId,
        description: TxDescription,
        status: TxStatus,
        timestamp: TxTimestamp,
        amount: TxAmount,
    ) -> Self {
        Self {
            tx_id,
            timestamp,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            description,
            status,
        }
    }
}
