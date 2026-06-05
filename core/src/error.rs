use std::fmt;

pub struct InvalidTxType(pub String);
pub struct InvalidStatus(pub String);
pub struct ParseNumberError {
    pub field: String,
    pub raw: String,
}

impl std::error::Error for Error {}
impl std::error::Error for BinError {}
impl std::error::Error for CsvError {}
impl std::error::Error for TextError {}

pub enum Error {
    Io(std::io::Error),
    Bin(BinError),
    Csv(CsvError),
    Text(TextError),
    InvalidTxType(InvalidTxType),
    InvalidStatus(InvalidStatus),
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

pub enum BinError {
    InvalidMagic([u8; 4]),
    InvalidRecordSize { expected: u32, actual: u32 },
    DescriptionLengthMismatch { expected: u32, actual: usize },
    InvalidTxType(u8),
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

pub enum CsvError {
    InvalidHeader,
    InvalidFieldCount { expected: usize, actual: usize },
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
            CsvError::InvalidDescriptionFormat(field) => write!(f, "Invalid description format: {}", field),
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
            CsvError::InvalidDescriptionFormat(field) => write!(f, "InvalidDescriptionFormat({:?})", field),
        }
    }
}

pub enum TextError {
    MissingField(String),
    DuplicateField(String),
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
