mod bin_format;
mod csv_format;
mod error;
mod txt_format;

use std::io::{Read, Write};
use std::str::FromStr;

pub use error::*;
pub use bin_format::{BinParser, BinSerializer};
pub use csv_format::{CsvParser, CsvSerializer};
pub use txt_format::{TextParser, TextSerializer};

pub const DEPOSIT_TYPE: &str = "DEPOSIT";
pub const WITHDRAWAL_TYPE: &str = "WITHDRAWAL";
pub const TRANSFER_TYPE: &str = "TRANSFER";

pub const PENDING_STATUS: &str = "PENDING";
pub const SUCCESS_STATUS: &str = "SUCCESS";
pub const FAILURE_STATUS: &str = "FAILURE";

pub const CSV: &str = "csv";
pub const BIN: &str = "bin";
pub const TXT: &str = "txt";
pub const TEXT: &str = "text";

const DEPOSIT_TYPE_INDEX: u8 = 0;
const WITHDRAWAL_TYPE_INDEX: u8 = 1;
const TRANSFER_TYPE_INDEX: u8 = 2;

const SUCCESS_STATUS_INDEX: u8 = 0;
const FAILURE_STATUS_INDEX: u8 = 1;
const PENDING_STATUS_INDEX: u8 = 2;

pub fn get_parser(format: &Format) -> ParserFormat {
    match format {
        Format::Csv => ParserFormat::Csv(CsvParser::new()),
        Format::Bin => ParserFormat::Bin(BinParser::new()),
        Format::Txt => ParserFormat::Txt(TextParser::new()),
    }
}

pub fn get_serializer(format: &Format) -> SerializerFormat {
    match format {
        Format::Csv => SerializerFormat::Csv(CsvSerializer::new()),
        Format::Bin => SerializerFormat::Bin(BinSerializer::new()),
        Format::Txt => SerializerFormat::Txt(TextSerializer::new()),
    }
}

pub trait Parse {
    fn parse<R: Read>(&self, reader: &mut R) -> Result<Vec<Tx>, Error>;
}

pub trait Serialize {
    fn serialize<W: Write>(&self, writer: &mut W, txs: &[Tx]) -> Result<(), Error>;
}

#[derive(Debug, PartialEq)]
pub enum Format {
    Csv,
    Bin,
    Txt,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            CSV => Ok(Format::Csv),
            BIN => Ok(Format::Bin),
            TXT | TEXT => Ok(Format::Txt),
            _ => Err(format!("Unknown format: '{}'. Supported: csv, bin, txt", s)),
        }
    }
}

pub enum ParserFormat {
    Csv(CsvParser),
    Bin(BinParser),
    Txt(TextParser),
}

impl Parse for ParserFormat {
    fn parse<R: std::io::Read>(&self, reader: &mut R) -> Result<Vec<Tx>, Error> {
        match self {
            ParserFormat::Csv(p) => p.parse(reader),
            ParserFormat::Bin(p) => p.parse(reader),
            ParserFormat::Txt(p) => p.parse(reader),
        }
    }
}


pub enum SerializerFormat {
    Csv(CsvSerializer),
    Bin(BinSerializer),
    Txt(TextSerializer),
}

impl Serialize for SerializerFormat {
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

#[derive(Debug, PartialEq, Clone)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

impl FromStr for TxType {
    type Err = InvalidTxType;

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
    fn from(tx_type: &TxType) -> Self {
        match tx_type {
            TxType::Deposit => DEPOSIT_TYPE_INDEX,
            TxType::Transfer => TRANSFER_TYPE_INDEX,
            TxType::Withdrawal => WITHDRAWAL_TYPE_INDEX,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TxStatus {
    Pending,
    Success,
    Failure,
}

impl FromStr for TxStatus {
    type Err = InvalidStatus;

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
    fn from(status: &TxStatus) -> Self {
        match status {
            TxStatus::Success => SUCCESS_STATUS_INDEX,
            TxStatus::Failure => FAILURE_STATUS_INDEX,
            TxStatus::Pending => PENDING_STATUS_INDEX,
        }
    }
}

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

#[derive(Debug, PartialEq, Clone)]
pub struct Tx {
    pub tx_type: TxType,
    pub tx_id: TxId,
    pub amount: TxAmount,
    pub status: TxStatus,
    pub from_user_id: TxFromUserId,
    pub timestamp: TxTimestamp,
    pub description: TxDescription,
    pub to_user_id: TxToUserId,
}

impl Tx {
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
