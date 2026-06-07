use crate::{
    DEPOSIT_TYPE, Error, FAILURE_STATUS, PENDING_STATUS, Parse, ParseNumberError, SUCCESS_STATUS,
    Serialize, TRANSFER_TYPE, TextError, Tx, TxAmount, TxDescription, TxFromUserId, TxId, TxStatus,
    TxTimestamp, TxToUserId, TxType, WITHDRAWAL_TYPE,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::str::FromStr;

fn parse_required_number_field<T: FromStr>(
    fields: &HashMap<String, String>,
    field_name: &str,
) -> Result<T, ParseNumberError> {
    let raw = fields.get(field_name).ok_or_else(|| ParseNumberError {
        field: field_name.to_string(),
        raw: format!("Missing {} field", field_name),
    })?;

    raw.parse::<T>().map_err(|_| ParseNumberError {
        field: field_name.to_string(),
        raw: raw.clone(),
    })
}

fn parse_required_text_field<T: FromStr>(
    fields: &HashMap<String, String>,
    field_name: &str,
) -> Result<T, TextError> {
    let raw = fields
        .get(field_name)
        .ok_or_else(|| TextError::MissingField(field_name.to_string()))?;

    raw.parse::<T>()
        .map_err(|_| TextError::MissingField(field_name.to_string()))
}

fn parse_description(fields: &HashMap<String, String>) -> Result<String, TextError> {
    let description = fields
        .get("DESCRIPTION")
        .ok_or_else(|| TextError::MissingField("DESCRIPTION".to_string()))?;

    Ok(description.trim_matches('"').to_string())
}

pub struct TextParser;

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TextParser {
    pub fn new() -> Self {
        Self
    }

    fn parse_record(lines: &[String]) -> Result<Tx, Error> {
        let mut fields = HashMap::new();

        for line in lines {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ':').collect();

            if parts.len() != 2 {
                return Err(TextError::InvalidLineFormat(line.to_string()).into());
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            if fields.contains_key(key) {
                return Err(TextError::DuplicateField(key.to_string()).into());
            }

            fields.insert(key.to_string(), value.to_string());
        }

        let tx_id = parse_required_number_field::<u64>(&fields, "TX_ID")?;
        let from_user_id = parse_required_number_field::<u64>(&fields, "FROM_USER_ID")?;
        let to_user_id = parse_required_number_field::<u64>(&fields, "TO_USER_ID")?;
        let amount = parse_required_number_field::<i64>(&fields, "AMOUNT")?;
        let timestamp = parse_required_number_field::<u64>(&fields, "TIMESTAMP")?;
        let tx_type = parse_required_text_field::<TxType>(&fields, "TX_TYPE")?;
        let status = parse_required_text_field::<TxStatus>(&fields, "STATUS")?;
        let description = parse_description(&fields)?;

        Ok(Tx::new(
            TxId(tx_id),
            tx_type,
            TxFromUserId(from_user_id),
            TxToUserId(to_user_id),
            TxDescription(description),
            status,
            TxTimestamp(timestamp),
            TxAmount(amount),
        ))
    }
}

impl Parse for TextParser {
    fn parse<R: Read>(&self, reader: &mut R) -> Result<Vec<Tx>, Error> {
        let reader = BufReader::new(reader);
        let mut txs = Vec::new();
        let mut current_record = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() {
                if !current_record.is_empty() {
                    let tx = Self::parse_record(&current_record)?;
                    txs.push(tx);
                    current_record.clear();
                }

                continue;
            }

            if line.starts_with('#') {
                continue;
            }

            current_record.push(line.to_string());
        }

        if !current_record.is_empty() {
            let tx = Self::parse_record(&current_record)?;
            txs.push(tx);
        }

        Ok(txs)
    }
}

pub struct TextSerializer;

impl TextSerializer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for TextSerializer {
    fn serialize<W: Write>(
        &self,
        writer: &mut W,
        transactions: &[Tx],
    ) -> Result<(), Error> {
        for (idx, tx) in transactions.iter().enumerate() {
            if idx > 0 {
                writer.write_all(b"\n")?
            }

            writer.write_all(format!("# Record {}\n", idx + 1).as_bytes())?;
            writer.write_all(format!("TX_ID: {}\n", tx.tx_id.0).as_bytes())?;

            let tx_type_str = match tx.tx_type {
                TxType::Deposit => DEPOSIT_TYPE,
                TxType::Transfer => TRANSFER_TYPE,
                TxType::Withdrawal => WITHDRAWAL_TYPE,
            };

            writer.write_all(format!("TX_TYPE: {}\n", tx_type_str).as_bytes())?;
            writer.write_all(format!("FROM_USER_ID: {}\n", tx.from_user_id.0).as_bytes())?;
            writer.write_all(format!("TO_USER_ID: {}\n", tx.to_user_id.0).as_bytes())?;

            let amount_display = match tx.tx_type {
                TxType::Withdrawal => tx.amount.0.abs(),
                _ => tx.amount.0,
            };

            writer.write_all(format!("AMOUNT: {}\n", amount_display).as_bytes())?;
            writer.write_all(format!("TIMESTAMP: {}\n", tx.timestamp.0).as_bytes())?;

            let status_str = match tx.status {
                TxStatus::Success => SUCCESS_STATUS,
                TxStatus::Failure => FAILURE_STATUS,
                TxStatus::Pending => PENDING_STATUS,
            };

            writer.write_all(format!("STATUS: {}\n", status_str).as_bytes())?;
            writer.write_all(format!("DESCRIPTION: \"{}\"\n", tx.description.0).as_bytes())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_parser_valid() {
        let text_data = r#"
            # Record 1 (DEPOSIT)
            TX_TYPE: DEPOSIT
            TO_USER_ID: 9223372036854775807
            FROM_USER_ID: 0
            TIMESTAMP: 1633036860000
            DESCRIPTION: "Record number 1"
            TX_ID: 1000000000000000
            AMOUNT: 100
            STATUS: FAILURE

            # Record 2 (TRANSFER)
            DESCRIPTION: "Record number 2"
            TIMESTAMP: 1633036920000
            STATUS: PENDING
            AMOUNT: 200
            TX_ID: 1000000000000001
            TX_TYPE: TRANSFER
            FROM_USER_ID: 9223372036854775807
            TO_USER_ID: 9223372036854775807
        "#;

        let parser = TextParser::new();
        let mut reader = text_data.as_bytes();
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
    fn test_text_parser_missing_field() {
        let text_data = r#"
        TX_TYPE: DEPOSIT
        TO_USER_ID: 123
        FROM_USER_ID: 0
        TIMESTAMP: 1633036860000
        DESCRIPTION: "test"
        AMOUNT: 100
        STATUS: FAILURE
    "#;

        let parser = TextParser::new();
        let mut reader = text_data.as_bytes();
        let result = parser.parse(&mut reader);

        match result {
            Err(Error::ParseNumberError(e)) => {
                assert_eq!(e.field, "TX_ID");
                assert!(e.raw.contains("Missing"));
            }
            _ => panic!("Expected ParseNumberError, got {:?}", result),
        }
    }

    #[test]
    fn test_text_parser_duplicate_field() {
        let text_data = r#"
            TX_ID: 100
            TX_TYPE: DEPOSIT
            TX_ID: 200
            TO_USER_ID: 123
            FROM_USER_ID: 0
            TIMESTAMP: 1633036860000
            DESCRIPTION: "test"
            AMOUNT: 100
            STATUS: FAILURE
        "#;

        let parser = TextParser::new();
        let mut reader = text_data.as_bytes();
        let result = parser.parse(&mut reader);

        assert!(matches!(result, Err(Error::Text(TextError::DuplicateField(f))) if f == "TX_ID"));
    }

    #[test]
    fn test_text_serializer_basic() {
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

        let serializer = TextSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &transactions).unwrap();

        let result = String::from_utf8(buffer).unwrap();
        let expected = r#"# Record 1
TX_ID: 1000000000000000
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9223372036854775807
AMOUNT: 100
TIMESTAMP: 1633036860000
STATUS: FAILURE
DESCRIPTION: "Record number 1"

# Record 2
TX_ID: 1000000000000001
TX_TYPE: TRANSFER
FROM_USER_ID: 9223372036854775807
TO_USER_ID: 9223372036854775807
AMOUNT: 200
TIMESTAMP: 1633036920000
STATUS: PENDING
DESCRIPTION: "Record number 2"
"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn test_text_serializer_empty() {
        let transactions: Vec<Tx> = vec![];

        let serializer = TextSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &transactions).unwrap();

        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_text_serializer_single_record() {
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

        let serializer = TextSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &transactions).unwrap();

        let result = String::from_utf8(buffer).unwrap();

        assert!(result.contains("AMOUNT: 5000"));
        assert!(!result.contains("AMOUNT: -5000"));

        let expected = r#"# Record 1
TX_ID: 123456789
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 987654321
TO_USER_ID: 0
AMOUNT: 5000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: "ATM withdrawal"
"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn test_text_serializer_with_quotes_in_description() {
        let tx = Tx::new(
            TxId(1),
            TxType::Deposit,
            TxFromUserId(0),
            TxToUserId(100),
            TxDescription("Record with \"quotes\" inside".to_string()),
            TxStatus::Success,
            TxTimestamp(1000),
            TxAmount(100),
        );

        let transactions = vec![tx];

        let serializer = TextSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &transactions).unwrap();

        let result = String::from_utf8(buffer).unwrap();

        assert!(result.contains("DESCRIPTION: \"Record with \"quotes\" inside\""));
    }

    #[test]
    fn test_text_serializer_roundtrip() {
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
        ];

        let serializer = TextSerializer::new();
        let mut buffer = Vec::new();
        serializer.serialize(&mut buffer, &original_txs).unwrap();

        let parser = TextParser::new();
        let mut reader = buffer.as_slice();
        let parsed_txs = parser.parse(&mut reader).unwrap();

        assert_eq!(parsed_txs.len(), original_txs.len());
        assert_eq!(parsed_txs[0].tx_id, original_txs[0].tx_id);
        assert_eq!(parsed_txs[0].tx_type, original_txs[0].tx_type);
        assert_eq!(parsed_txs[0].amount, original_txs[0].amount);
        assert_eq!(parsed_txs[0].description, original_txs[0].description);

        assert_eq!(parsed_txs[1].tx_id, original_txs[1].tx_id);
        assert_eq!(parsed_txs[1].tx_type, original_txs[1].tx_type);
        assert_eq!(parsed_txs[1].amount, original_txs[1].amount);
    }
}
