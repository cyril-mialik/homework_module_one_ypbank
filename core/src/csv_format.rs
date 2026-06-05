use crate::{
    CsvError, Error, InvalidStatus, InvalidTxType, Parse, ParseNumberError, Tx, TxAmount,
    TxDescription, TxFromUserId, TxId, TxStatus, TxTimestamp, TxToUserId, TxType,
};
use std::io::{BufRead, BufReader, Read};

const PARTS_TOTAL_LEN: usize = 8;
const ID_PART: usize = 0;
const TYPE_PART: usize = 1;
const FROM_USER_ID_PART: usize = 2;
const TO_USER_ID_PART: usize = 3;
const AMOUNT_PART: usize = 4;
const TIMESTAMP_PART: usize = 5;
const STATUS_PART: usize = 6;
const DESCRIPTION_PART: usize = 7;

fn parse_number_part<T: std::str::FromStr>(
    part: &str,
    field_name: &str,
) -> Result<T, ParseNumberError> {
    part.parse::<T>().map_err(|_| ParseNumberError {
        field: field_name.to_string(),
        raw: part.to_string(),
    })
}

fn parse_tx_type(part: &str) -> Result<TxType, InvalidTxType> {
    let part = part.parse::<TxType>()?;

    Ok(part)
}

fn parse_status(part: &str) -> Result<TxStatus, InvalidStatus> {
    let part = part.parse::<TxStatus>()?;

    Ok(part)
}

fn parse_description(description: &str) -> String {
    if description.starts_with('"') && description.ends_with('"') {
        return description[1..description.len() - 1].to_string();
    }

    description.to_string()
}

pub struct CsvParser;

impl CsvParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parse for CsvParser {
    fn parse<R: Read>(&self, reader: &mut R) -> Result<Vec<Tx>, Error> {
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();
        let mut txs = Vec::new();

        let header = lines.next().ok_or(CsvError::InvalidHeader)??;
        let expected_header =
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";

        if header.trim() != expected_header {
            return Err(CsvError::InvalidHeader.into());
        }

        for line in lines {
            let line = line?;
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();

            if parts.len() != PARTS_TOTAL_LEN {
                return Err(CsvError::InvalidFieldCount {
                    expected: PARTS_TOTAL_LEN,
                    actual: parts.len(),
                }
                .into());
            }

            let tx_type = parse_tx_type(parts[TYPE_PART])?;
            let status = parse_status(parts[STATUS_PART])?;
            let description = parse_description(parts[DESCRIPTION_PART].trim());
            let tx_id = parse_number_part::<u64>(parts[ID_PART], "ID_PART")?;
            let from_user_id = parse_number_part::<u64>(parts[FROM_USER_ID_PART], "FROM_USER_ID")?;
            let to_user_id = parse_number_part::<u64>(parts[TO_USER_ID_PART], "TO_USER_ID")?;
            let amount = parse_number_part::<i64>(parts[AMOUNT_PART], "AMOUNT")?;
            let timestamp = parse_number_part::<u64>(parts[TIMESTAMP_PART],"TIMESTAMP")?;

            txs.push(Tx::new(
                TxId(tx_id),
                tx_type,
                TxFromUserId(from_user_id),
                TxToUserId(to_user_id),
                TxDescription(description),
                status,
                TxTimestamp(timestamp),
                TxAmount(amount),
            ));
        }

        Ok(txs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parser_valid() {
        let csv_data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,"Record number 1"
1000000000000001,TRANSFER,9223372036854775807,9223372036854775807,200,1633036920000,PENDING,"Record number 2"
"#;

        let parser = CsvParser::new();
        let mut reader = csv_data.as_bytes();
        let result = parser.parse(&mut reader).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].tx_id, TxId(1000000000000000));
        assert_eq!(result[0].tx_type, TxType::Deposit);
        assert_eq!(result[0].status, TxStatus::Failure);
        assert_eq!(result[1].tx_id, TxId(1000000000000001));
        assert_eq!(result[1].tx_type, TxType::Transfer);
        assert_eq!(result[1].status, TxStatus::Pending);
    }

    #[test]
    fn test_csv_parser_invalid_header() {
        let csv_data = r#"WRONG_HEADER
1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE,"Record"
"#;

        let parser = CsvParser::new();
        let mut reader = csv_data.as_bytes();
        let result = parser.parse(&mut reader);

        assert!(matches!(result, Err(Error::Csv(CsvError::InvalidHeader))));
    }

    #[test]
    fn test_csv_parser_invalid_field_count() {
        let csv_data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1000000000000000,DEPOSIT,0,9223372036854775807,100,1633036860000,FAILURE
"#;

        let parser = CsvParser::new();
        let mut reader = csv_data.as_bytes();
        let result = parser.parse(&mut reader);

        assert!(matches!(
            result,
            Err(Error::Csv(CsvError::InvalidFieldCount {
                expected: 8,
                actual: 7
            }))
        ));
    }
}
