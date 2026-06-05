use crate::{
    BinError, Error, Parse, Serialize, Tx, TxAmount, TxDescription, TxFromUserId, TxId, TxStatus,
    TxTimestamp, TxToUserId, TxType,
};
use std::io::{Read, Write};

const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];

pub struct BinParser;

impl Default for BinParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parse for BinParser {
    fn parse<R: Read>(&self, reader: &mut R) -> Result<Vec<Tx>, Error> {
        let mut txs = Vec::new();

        while let Some(tx) = Self::parse_single_record(reader)? {
            txs.push(tx);
        }

        Ok(txs)
    }
}

impl BinParser {
    pub fn new() -> Self {
        Self
    }

    fn read_u64_be<R: Read>(reader: &mut R) -> Result<u64, Error> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;

        Ok(u64::from_be_bytes(buf))
    }

    fn read_i64_be<R: Read>(reader: &mut R) -> Result<i64, Error> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;

        Ok(i64::from_be_bytes(buf))
    }

    fn read_u32_be<R: Read>(reader: &mut R) -> Result<u32, Error> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;

        Ok(u32::from_be_bytes(buf))
    }

    fn read_u8<R: Read>(reader: &mut R) -> Result<u8, Error> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    fn parse_single_record<R: Read>(reader: &mut R) -> Result<Option<Tx>, Error> {
        let mut magic = [0u8; 4];

        if let Err(e) = reader.read_exact(&mut magic) {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }

            return Err(e.into());
        }

        if magic != MAGIC {
            return Err(BinError::InvalidMagic(magic).into());
        }

        let record_size = Self::read_u32_be(reader)?;

        let mut record_data = vec![0u8; record_size as usize];
        reader.read_exact(&mut record_data)?;

        let mut data = record_data.as_slice();

        let tx_id = TxId(Self::read_u64_be(&mut data)?);

        let tx_type_byte = Self::read_u8(&mut data)?;
        let tx_type =
            TxType::try_from(tx_type_byte).map_err(|_| BinError::InvalidTxType(tx_type_byte))?;

        let from_user_id = TxFromUserId(Self::read_u64_be(&mut data)?);
        let to_user_id = TxToUserId(Self::read_u64_be(&mut data)?);
        let amount = TxAmount(Self::read_i64_be(&mut data)?);
        let timestamp = TxTimestamp(Self::read_u64_be(&mut data)?);

        let status_byte = Self::read_u8(&mut data)?;
        let status =
            TxStatus::try_from(status_byte).map_err(|_| BinError::InvalidStatus(status_byte))?;

        let desc_len = Self::read_u32_be(&mut data)?;

        if desc_len as usize > data.len() {
            return Err(BinError::DescriptionLengthMismatch {
                expected: desc_len,
                actual: data.len(),
            }
            .into());
        }

        let desc_bytes = &data[..desc_len as usize];
        let description = TxDescription(String::from_utf8_lossy(desc_bytes).to_string());

        Ok(Some(Tx::new(
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            description,
            status,
            timestamp,
            amount,
        )))
    }
}

pub struct BinSerializer;

impl BinSerializer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BinSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for BinSerializer {
    fn serialize<W: Write>(&self, writer: &mut W, transactions: &[Tx]) -> Result<(), Error> {
        for tx in transactions {
            writer.write_all(&MAGIC)?;

            let mut body = Vec::new();

            body.write_all(&tx.tx_id.0.to_be_bytes())?;
            body.write_all(&[u8::from(&tx.tx_type)])?;
            body.write_all(&tx.from_user_id.0.to_be_bytes())?;
            body.write_all(&tx.to_user_id.0.to_be_bytes())?;
            body.write_all(&tx.amount.0.to_be_bytes())?;
            body.write_all(&tx.timestamp.0.to_be_bytes())?;
            body.write_all(&[u8::from(&tx.status)])?;

            let desc_bytes = tx.description.0.as_bytes();
            let desc_len = desc_bytes.len() as u32;

            body.write_all(&desc_len.to_be_bytes())?;
            body.write_all(desc_bytes)?;

            let record_size = body.len() as u32;

            writer.write_all(&record_size.to_be_bytes())?;
            writer.write_all(&body)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_bin_record(
        tx_id: u64,
        tx_type: u8,
        from_user_id: u64,
        to_user_id: u64,
        amount: i64,
        timestamp: u64,
        status: u8,
        description: &str,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&MAGIC);

        let desc_bytes = description.as_bytes();
        let fixed_fields_size = 8 + 1 + 8 + 8 + 8 + 8 + 1 + 4;
        let record_size = fixed_fields_size + desc_bytes.len();
        data.extend_from_slice(&(record_size as u32).to_be_bytes());

        data.extend_from_slice(&tx_id.to_be_bytes());
        data.push(tx_type);
        data.extend_from_slice(&from_user_id.to_be_bytes());
        data.extend_from_slice(&to_user_id.to_be_bytes());
        data.extend_from_slice(&amount.to_be_bytes());
        data.extend_from_slice(&timestamp.to_be_bytes());
        data.push(status);
        data.extend_from_slice(&(desc_bytes.len() as u32).to_be_bytes());
        data.extend_from_slice(desc_bytes);

        data
    }

    #[test]
    fn test_bin_parser_valid() {
        let record1 = create_bin_record(
            1000000000000000,
            0,
            0,
            9223372036854775807,
            100,
            1633036860000,
            1,
            "Record number 1",
        );
        let record2 = create_bin_record(
            1000000000000001,
            2,
            9223372036854775807,
            9223372036854775807,
            200,
            1633036920000,
            2,
            "Record number 2",
        );

        let mut data = Vec::new();
        data.extend_from_slice(&record1);
        data.extend_from_slice(&record2);

        let parser = BinParser::new();
        let mut reader = data.as_slice();
        let result = parser.parse(&mut reader).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].tx_id, TxId(1000000000000000));
        assert_eq!(result[0].tx_type, TxType::Deposit);
        assert_eq!(result[0].status, TxStatus::Failure);
        assert_eq!(result[1].tx_id, TxId(1000000000000001));
        assert_eq!(result[1].tx_type, TxType::Transfer);
        assert_eq!(result[1].status, TxStatus::Pending);
        assert_eq!(result[0].description.0, "Record number 1");
        assert_eq!(result[1].description.0, "Record number 2");
    }

    #[test]
    fn test_bin_parser_invalid_magic() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        data.extend_from_slice(&(32u32).to_be_bytes());

        let parser = BinParser::new();
        let mut reader = data.as_slice();
        let result = parser.parse(&mut reader);

        assert!(matches!(result, Err(Error::Bin(BinError::InvalidMagic(_)))));
    }

    #[test]
    fn test_bin_parser_empty() {
        let data: Vec<u8> = vec![];

        let parser = BinParser::new();
        let mut reader = data.as_slice();
        let result = parser.parse(&mut reader).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_bin_serializer_basic() {
        let tx1 = Tx::new(
            TxId(1000000000000000),
            TxType::Deposit,
            TxFromUserId(0),
            TxToUserId(9223372036854775807),
            TxDescription("Record number 1".to_string()),
            TxStatus::Failure,
            TxTimestamp(1633036860000),
            TxAmount(100),
        );

        let tx2 = Tx::new(
            TxId(1000000000000001),
            TxType::Transfer,
            TxFromUserId(9223372036854775807),
            TxToUserId(9223372036854775807),
            TxDescription("Record number 2".to_string()),
            TxStatus::Pending,
            TxTimestamp(1633036920000),
            TxAmount(200),
        );

        let transactions = vec![tx1, tx2];

        let serializer = BinSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &transactions).unwrap();

        let parser = BinParser::new();
        let mut reader = buffer.as_slice();
        let parsed = parser.parse(&mut reader).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].tx_id, transactions[0].tx_id);
        assert_eq!(parsed[0].tx_type, transactions[0].tx_type);
        assert_eq!(parsed[0].amount, transactions[0].amount);
        assert_eq!(parsed[0].description, transactions[0].description);

        assert_eq!(parsed[1].tx_id, transactions[1].tx_id);
        assert_eq!(parsed[1].tx_type, transactions[1].tx_type);
        assert_eq!(parsed[1].status, transactions[1].status);
    }

    #[test]
    fn test_bin_serializer_empty() {
        let transactions: Vec<Tx> = vec![];

        let serializer = BinSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &transactions).unwrap();

        assert!(buffer.is_empty());
    }

    #[test]
    fn test_bin_serializer_single_record() {
        let tx = Tx::new(
            TxId(123456789),
            TxType::Withdrawal,
            TxFromUserId(987654321),
            TxToUserId(0),
            TxDescription("ATM withdrawal".to_string()),
            TxStatus::Success,
            TxTimestamp(1633036800000),
            TxAmount(-5000),
        );

        let transactions = vec![tx];

        let serializer = BinSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &transactions).unwrap();

        let mut cursor = buffer.as_slice();
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic).unwrap();
        assert_eq!(magic, MAGIC);

        let record_size = u32::from_be_bytes({
            let mut buf = [0u8; 4];
            cursor.read_exact(&mut buf).unwrap();
            buf
        });

        assert_eq!(record_size as usize, cursor.len());

        let parser = BinParser::new();
        let mut reader = buffer.as_slice();
        let parsed = parser.parse(&mut reader).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].tx_id, TxId(123456789));
        assert_eq!(parsed[0].tx_type, TxType::Withdrawal);
        assert_eq!(parsed[0].amount, TxAmount(-5000));
        assert_eq!(parsed[0].description.0, "ATM withdrawal");
    }

    #[test]
    fn test_bin_serializer_roundtrip() {
        let original_txs = vec![
            Tx::new(
                TxId(1001),
                TxType::Deposit,
                TxFromUserId(0),
                TxToUserId(501),
                TxDescription("Initial deposit".to_string()),
                TxStatus::Success,
                TxTimestamp(1672531200000),
                TxAmount(50000),
            ),
            Tx::new(
                TxId(1002),
                TxType::Transfer,
                TxFromUserId(501),
                TxToUserId(502),
                TxDescription("Payment for services".to_string()),
                TxStatus::Failure,
                TxTimestamp(1672534800000),
                TxAmount(15000),
            ),
            Tx::new(
                TxId(1003),
                TxType::Withdrawal,
                TxFromUserId(502),
                TxToUserId(0),
                TxDescription("ATM withdrawal".to_string()),
                TxStatus::Pending,
                TxTimestamp(1672538400000),
                TxAmount(-1000),
            ),
        ];

        let serializer = BinSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &original_txs).unwrap();

        let parser = BinParser::new();
        let mut reader = buffer.as_slice();
        let parsed_txs = parser.parse(&mut reader).unwrap();

        assert_eq!(parsed_txs.len(), original_txs.len());

        for (original, parsed) in original_txs.iter().zip(parsed_txs.iter()) {
            assert_eq!(original.tx_id, parsed.tx_id);
            assert_eq!(original.tx_type, parsed.tx_type);
            assert_eq!(original.from_user_id, parsed.from_user_id);
            assert_eq!(original.to_user_id, parsed.to_user_id);
            assert_eq!(original.amount, parsed.amount);
            assert_eq!(original.timestamp, parsed.timestamp);
            assert_eq!(original.status, parsed.status);
            assert_eq!(original.description, parsed.description);
        }
    }
}
