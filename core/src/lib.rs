mod bin_format;
mod csv_format;
mod error;
mod txt_format;

pub use bin_format::BinParser;
pub use csv_format::CsvParser;
pub use error::*;
pub use txt_format::TextParser;

const DEPOSIT_TYPE: &str = "DEPOSIT";
const WITHDRAWAL_TYPE: &str = "WITHDRAWAL";
const TRANSFER_TYPE: &str = "TRANSFER";

const DEPOSIT_TYPE_INDEX: u8 = 0;
const WITHDRAWAL_TYPE_INDEX: u8 = 1;
const TRANSFER_TYPE_INDEX: u8 = 2;

const PENDING_STATUS: &str = "PENDING";
const SUCCESS_STATUS: &str = "SUCCESS";
const FAILURE_STATUS: &str = "FAILURE";

const SUCCESS_STATUS_INDEX: u8 = 0;
const FAILURE_STATUS_INDEX: u8 = 1;
const PENDING_STATUS_INDEX: u8 = 2;

pub trait Parse {
    fn parse<R: std::io::Read>(&self, reader: &mut R) -> Result<Vec<Tx>, Error>;
}

#[derive(Debug, PartialEq)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

impl std::str::FromStr for TxType {
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

#[derive(Debug, PartialEq)]
pub enum TxStatus {
    Pending,
    Success,
    Failure,
}

impl std::str::FromStr for TxStatus {
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

#[derive(Debug, PartialEq)]
pub struct TxId(pub u64);

#[derive(Debug, PartialEq)]
pub struct TxFromUserId(pub u64);

#[derive(Debug, PartialEq)]
pub struct TxToUserId(pub u64);

#[derive(Debug, PartialEq)]
pub struct TxAmount(pub i64);

#[derive(Debug, PartialEq)]
pub struct TxTimestamp(pub u64);

#[derive(Debug, PartialEq)]
pub struct TxDescription(pub String);

#[derive(Debug, PartialEq)]
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
