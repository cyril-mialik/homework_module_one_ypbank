//! Constants for formats, transaction types, and statuses
//!
//! This module contains all constants used in the core library,
//! including string representations of file formats, transaction types,
//! their statuses, and numeric indices for the binary format.

/// Transaction type "Deposit" (account top-up)
pub const DEPOSIT_TYPE: &str = "DEPOSIT";

/// Transaction type "Withdrawal" (fund withdrawal)
pub const WITHDRAWAL_TYPE: &str = "WITHDRAWAL";

/// Transaction type "Transfer" (between users)
pub const TRANSFER_TYPE: &str = "TRANSFER";

/// Status "Pending" (in progress)
pub const PENDING_STATUS: &str = "PENDING";

/// Status "Success" (completed successfully)
pub const SUCCESS_STATUS: &str = "SUCCESS";

/// Status "Failure" (failed)
pub const FAILURE_STATUS: &str = "FAILURE";

/// File extension for CSV format
pub const CSV: &str = "csv";

/// File extension for binary format
pub const BIN: &str = "bin";

/// File extension for text format
pub const TXT: &str = "txt";

/// Alternative name for text format
pub const TEXT: &str = "text";

/// Numeric index for "Deposit" type in binary format
pub const DEPOSIT_TYPE_INDEX: u8 = 0;

/// Numeric index for "Transfer" type in binary format
pub const TRANSFER_TYPE_INDEX: u8 = 1;

/// Numeric index for "Withdrawal" type in binary format
pub const WITHDRAWAL_TYPE_INDEX: u8 = 2;

/// Numeric index for "Success" status in binary format
pub const SUCCESS_STATUS_INDEX: u8 = 0;

/// Numeric index for "Failure" status in binary format
pub const FAILURE_STATUS_INDEX: u8 = 1;

/// Numeric index for "Pending" status in binary format
pub const PENDING_STATUS_INDEX: u8 = 2;
