use crate::{
    BinError, Error, Parse, Tx, TxAmount, TxDescription, TxFromUserId, TxId, TxStatus, TxTimestamp,
    TxToUserId, TxType,
};
use std::io::Read;

const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];

pub struct BinParser;

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
            1,
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
}
