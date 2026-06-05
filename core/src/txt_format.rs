use crate::{
    Error, Parse, ParseNumberError, TextError, Tx, TxAmount, TxDescription, TxFromUserId, TxId,
    TxStatus, TxTimestamp, TxToUserId, TxType,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

fn parse_required_number_field<T: std::str::FromStr>(
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

fn parse_required_text_field<T: std::str::FromStr>(
    fields: &HashMap<String, String>, 
    field_name: &str
) -> Result<T, TextError> {
    let raw = fields
        .get(field_name)
        .ok_or_else(|| TextError::MissingField(field_name.to_string()))?;
    
    raw.parse::<T>().map_err(|_| TextError::MissingField(field_name.to_string()))
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

        assert!(matches!(result, Err(Error::Text(TextError::MissingField(f))) if f == "TX_ID"));
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
}
