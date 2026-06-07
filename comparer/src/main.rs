use clap::Parser;
use core::{Format, Parse, Tx, get_parser};
use std::cmp::min;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(name = "comparer")]
#[command(about = "Compare transaction files between CSV, binary, and text formats", long_about = None)]
struct Cli {
    left_file: PathBuf,
    right_file: PathBuf,

    #[arg(long)]
    strict: bool,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug)]
struct Difference {
    index: usize,
    field: String,
    left_value: String,
    right_value: String,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if !cli.left_file.exists() {
        return Err(format!("File not found: {}", cli.left_file.display()).into());
    }

    if !cli.right_file.exists() {
        return Err(format!("File not found: {}", cli.right_file.display()).into());
    }

    let input_transactions = parse_file(&cli.left_file)?;
    let output_transactions = parse_file(&cli.right_file)?;

    if input_transactions.is_empty() && output_transactions.is_empty() {
        eprintln!("Warning: Both files have no transactions");
    }

    let differences = compare_transactions(
        &input_transactions,
        &output_transactions,
        cli.verbose,
        cli.strict,
    );

    if differences.is_empty() {
        println!("OK: files are identical");
        println!(
            "Successfully compared {} transactions",
            input_transactions.len()
        );

        Ok(())
    } else {
        println!("ERROR: files differ");

        for diff in &differences {
            println!(
                "Transaction #{}: field '{}' differs\n  File input: {}\n  File output: {}",
                diff.index + 1,
                diff.field,
                diff.left_value,
                diff.right_value,
            );
        }

        Err(format!("Found {} difference(s)", differences.len()).into())
    }
}

fn parse_file(path: &PathBuf) -> Result<Vec<Tx>, Box<dyn std::error::Error>> {
    let format = detect_format(path)?;
    let parser = get_parser(&format);
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let transactions = parser
        .parse(&mut reader)
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

    Ok(transactions)
}

fn detect_format(path: &Path) -> Result<Format, String> {
    match path.extension() {
        Some(ext) => match ext.to_str() {
            Some("bin") => Ok(Format::Bin),
            Some("csv") => Ok(Format::Csv),
            Some("txt") => Ok(Format::Txt),
            _ => Err(format!("Unsupported file extension: {:?}", ext)),
        },
        _ => Err("File has no extension".to_string()),
    }
}

fn compare_transactions(
    left_txs: &[Tx],
    right_txs: &[Tx],
    verbose: bool,
    strict: bool,
) -> Vec<Difference> {
    let mut differences = Vec::new();
    let left_txs_len = left_txs.len();
    let right_txs_len = right_txs.len();

    if left_txs_len != right_txs_len {
        if strict {
            differences.push(Difference {
                index: 0,
                field: "record_count".to_string(),
                left_value: left_txs_len.to_string(),
                right_value: right_txs_len.to_string(),
            });

            return differences;
        }

        eprintln!(
            "Warning: different record counts ({} vs {}), comparing first {} records",
            left_txs_len,
            right_txs_len,
            min(left_txs_len, right_txs_len)
        );
    }

    let min_len = min(left_txs_len, right_txs_len);

    for i in 0..min_len {
        let left_tx = &left_txs[i];
        let right_tx = &right_txs[i];

        macro_rules! compare_field {
            ($field:ident, $name:expr) => {
                if left_tx.$field != right_tx.$field {
                    differences.push(Difference {
                        index: i,
                        field: $name.to_string(),
                        left_value: format!("{:?}", left_tx.$field),
                        right_value: format!("{:?}", right_tx.$field),
                    });
                }
            };
        }

        compare_field!(tx_id, "tx_id");
        compare_field!(tx_type, "tx_type");
        compare_field!(from_user_id, "from_user_id");
        compare_field!(to_user_id, "to_user_id");
        compare_field!(amount, "amount");
        compare_field!(timestamp, "timestamp");
        compare_field!(status, "status");
        compare_field!(description, "description");

        if !verbose && !differences.is_empty() {
            return differences;
        }
    }

    if left_txs_len != right_txs_len && !strict && differences.is_empty() {
        differences.push(Difference {
            index: min_len,
            field: "extra_records".to_string(),
            left_value: format!(
                "{} extra records",
                left_txs_len.saturating_sub(right_txs_len)
            ),
            right_value: format!(
                "{} extra records",
                right_txs_len.saturating_sub(left_txs_len)
            ),
        });
    }

    differences
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        BinParser, BinSerializer, CsvParser, CsvSerializer, Serialize, TextParser, TextSerializer,
        TxAmount, TxDescription, TxFromUserId, TxId, TxStatus, TxTimestamp, TxToUserId, TxType,
    };
    use std::io::Cursor;

    fn create_test_transactions() -> Vec<Tx> {
        vec![
            Tx::new(
                TxId(1000000000000000),
                TxType::Deposit,
                TxFromUserId(0),
                TxToUserId(9223372036854775807),
                TxDescription("Record number 1".to_string()),
                TxStatus::Failure,
                TxTimestamp(1633036860000),
                TxAmount(100),
            ),
            Tx::new(
                TxId(1000000000000001),
                TxType::Transfer,
                TxFromUserId(9223372036854775807),
                TxToUserId(9223372036854775807),
                TxDescription("Record number 2".to_string()),
                TxStatus::Pending,
                TxTimestamp(1633036920000),
                TxAmount(200),
            ),
            Tx::new(
                TxId(1000000000000002),
                TxType::Withdrawal,
                TxFromUserId(100500),
                TxToUserId(0),
                TxDescription("ATM withdrawal".to_string()),
                TxStatus::Success,
                TxTimestamp(1633036980000),
                TxAmount(-5000),
            ),
        ]
    }

    #[test]
    fn test_detect_format_bin() {
        let path = PathBuf::from("test.bin");
        assert_eq!(detect_format(&path).unwrap(), Format::Bin);
    }

    #[test]
    fn test_detect_format_csv() {
        let path = PathBuf::from("test.csv");
        assert_eq!(detect_format(&path).unwrap(), Format::Csv);
    }

    #[test]
    fn test_detect_format_txt() {
        let path = PathBuf::from("test.txt");
        assert_eq!(detect_format(&path).unwrap(), Format::Txt);
    }

    #[test]
    fn test_detect_format_unsupported_extension() {
        let path = PathBuf::from("test.xyz");
        assert!(detect_format(&path).is_err());
    }

    #[test]
    fn test_detect_format_no_extension() {
        let path = PathBuf::from("test");
        assert!(detect_format(&path).is_err());
    }

    #[test]
    fn test_compare_identical_transactions() {
        let txs = create_test_transactions();
        let differences = compare_transactions(&txs, &txs, false, false);
        assert!(differences.is_empty());
    }

    #[test]
    fn test_compare_different_tx_id() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();
        txs2[0].tx_id = TxId(9999999999999999);

        let differences = compare_transactions(&txs1, &txs2, false, false);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "tx_id");
    }

    #[test]
    fn test_compare_different_tx_type() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();
        txs2[0].tx_type = TxType::Withdrawal;

        let differences = compare_transactions(&txs1, &txs2, false, false);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "tx_type");
    }

    #[test]
    fn test_compare_different_amount() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();
        txs2[1].amount = TxAmount(99999);

        let differences = compare_transactions(&txs1, &txs2, false, false);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "amount");
    }

    #[test]
    fn test_compare_different_status() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();
        txs2[1].status = TxStatus::Success;

        let differences = compare_transactions(&txs1, &txs2, false, false);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "status");
    }

    #[test]
    fn test_compare_different_description() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();
        txs2[0].description = TxDescription("Different".to_string());

        let differences = compare_transactions(&txs1, &txs2, false, false);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "description");
    }

    #[test]
    fn test_compare_multiple_differences_without_verbose() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();
        txs2[0].tx_id = TxId(1);
        txs2[0].amount = TxAmount(999);
        txs2[1].status = TxStatus::Success;

        let differences = compare_transactions(&txs1, &txs2, false, false);

        assert!(differences.len() >= 1);
        assert!(differences.len() <= 2);
    }

    #[test]
    fn test_compare_multiple_differences_with_verbose() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();
        txs2[0].tx_id = TxId(1);
        txs2[0].amount = TxAmount(999);
        txs2[1].status = TxStatus::Success;

        let differences = compare_transactions(&txs1, &txs2, true, false);

        assert!(differences.len() >= 3);
    }

    #[test]
    fn test_compare_different_record_counts_strict_mode() {
        let txs1 = create_test_transactions();
        let txs2 = vec![create_test_transactions()[0].clone()];

        let differences = compare_transactions(&txs1, &txs2, false, true);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "record_count");
    }

    #[test]
    fn test_compare_different_record_counts_non_strict() {
        let txs1 = create_test_transactions();
        let txs2 = vec![
            create_test_transactions()[0].clone(),
            create_test_transactions()[1].clone(),
        ];

        let differences = compare_transactions(&txs1, &txs2, false, false);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "extra_records");
    }

    #[test]
    fn test_compare_empty_arrays() {
        let txs1: Vec<Tx> = vec![];
        let txs2: Vec<Tx> = vec![];

        let differences = compare_transactions(&txs1, &txs2, false, false);
        assert!(differences.is_empty());
    }

    #[test]
    fn test_compare_one_empty_one_non_empty_strict() {
        let txs1 = create_test_transactions();
        let txs2 = vec![];

        let differences = compare_transactions(&txs1, &txs2, false, true);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "record_count");
    }

    #[test]
    fn test_parse_file_from_buffer_instead_of_real_file() {
        let transactions = create_test_transactions();

        let mut buffer = Vec::new();
        let bin_serializer = BinSerializer::new();
        bin_serializer
            .serialize(&mut buffer, &transactions)
            .unwrap();

        let mut cursor = Cursor::new(buffer);
        let bin_parser = BinParser::new();
        let parsed = bin_parser.parse(&mut cursor).unwrap();

        assert_eq!(parsed.len(), transactions.len());
        assert_eq!(parsed[0].tx_id, transactions[0].tx_id);
    }

    #[test]
    fn test_compare_after_format_conversion_in_memory() {
        let original = create_test_transactions();

        let mut csv_buffer = Vec::new();
        let csv_serializer = CsvSerializer::new();
        csv_serializer
            .serialize(&mut csv_buffer, &original)
            .unwrap();

        let mut csv_reader = Cursor::new(csv_buffer);
        let csv_parser = CsvParser::new();
        let from_csv = csv_parser.parse(&mut csv_reader).unwrap();

        let mut bin_buffer = Vec::new();
        let bin_serializer = BinSerializer::new();
        bin_serializer
            .serialize(&mut bin_buffer, &original)
            .unwrap();

        let mut bin_reader = Cursor::new(bin_buffer);
        let bin_parser = BinParser::new();
        let from_bin = bin_parser.parse(&mut bin_reader).unwrap();

        let differences = compare_transactions(&from_csv, &from_bin, false, false);
        assert!(differences.is_empty());
    }

    #[test]
    fn test_roundtrip_csv_to_bin_to_csv() {
        let original = create_test_transactions();

        let mut csv_buffer = Vec::new();
        let csv_serializer = CsvSerializer::new();
        csv_serializer
            .serialize(&mut csv_buffer, &original)
            .unwrap();

        let mut csv_reader = Cursor::new(csv_buffer);
        let csv_parser = CsvParser::new();
        let parsed_from_csv = csv_parser.parse(&mut csv_reader).unwrap();

        let mut bin_buffer = Vec::new();
        let bin_serializer = BinSerializer::new();
        bin_serializer
            .serialize(&mut bin_buffer, &parsed_from_csv)
            .unwrap();

        let mut bin_reader = Cursor::new(bin_buffer);
        let bin_parser = BinParser::new();
        let parsed_from_bin = bin_parser.parse(&mut bin_reader).unwrap();

        let differences = compare_transactions(&original, &parsed_from_bin, false, false);
        assert!(differences.is_empty());
    }

    #[test]
    fn test_roundtrip_txt_to_csv_to_txt() {
        let original = create_test_transactions();

        let mut txt_buffer = Vec::new();
        let txt_serializer = TextSerializer::new();
        txt_serializer
            .serialize(&mut txt_buffer, &original)
            .unwrap();

        let mut txt_reader = Cursor::new(txt_buffer);
        let txt_parser = TextParser::new();
        let parsed_from_txt = txt_parser.parse(&mut txt_reader).unwrap();

        let mut csv_buffer = Vec::new();
        let csv_serializer = CsvSerializer::new();
        csv_serializer
            .serialize(&mut csv_buffer, &parsed_from_txt)
            .unwrap();

        let mut csv_reader = Cursor::new(csv_buffer);
        let csv_parser = CsvParser::new();
        let parsed_from_csv = csv_parser.parse(&mut csv_reader).unwrap();

        assert_eq!(original.len(), parsed_from_csv.len());

        for (i, (orig, parsed)) in original.iter().zip(parsed_from_csv.iter()).enumerate() {
            assert_eq!(
                orig.tx_id, parsed.tx_id,
                "Transaction {}: tx_id mismatch",
                i
            );
            assert_eq!(
                orig.tx_type, parsed.tx_type,
                "Transaction {}: tx_type mismatch",
                i
            );
            assert_eq!(
                orig.from_user_id, parsed.from_user_id,
                "Transaction {}: from_user_id mismatch",
                i
            );
            assert_eq!(
                orig.to_user_id, parsed.to_user_id,
                "Transaction {}: to_user_id mismatch",
                i
            );
            assert_eq!(
                orig.timestamp, parsed.timestamp,
                "Transaction {}: timestamp mismatch",
                i
            );
            assert_eq!(
                orig.status, parsed.status,
                "Transaction {}: status mismatch",
                i
            );
            assert_eq!(
                orig.description, parsed.description,
                "Transaction {}: description mismatch",
                i
            );

            match orig.tx_type {
                TxType::Withdrawal => assert_eq!(
                    orig.amount.0.abs(),
                    parsed.amount.0,
                    "Transaction {}: amount mismatch (Withdrawal: absolute value expected)",
                    i
                ),
                _ => assert_eq!(
                    orig.amount, parsed.amount,
                    "Transaction {}: amount mismatch",
                    i
                )
            }
        }
    }

    #[test]
    fn test_different_after_modification() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();
        txs2[0].amount = TxAmount(99999);

        let mut buffer1 = Vec::new();
        let mut buffer2 = Vec::new();
        let serializer = BinSerializer::new();
        serializer.serialize(&mut buffer1, &txs1).unwrap();
        serializer.serialize(&mut buffer2, &txs2).unwrap();

        let mut reader1 = Cursor::new(buffer1);
        let mut reader2 = Cursor::new(buffer2);
        let parser = BinParser::new();
        let parsed1 = parser.parse(&mut reader1).unwrap();
        let parsed2 = parser.parse(&mut reader2).unwrap();

        let differences = compare_transactions(&parsed1, &parsed2, false, false);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].field, "amount");
    }

    #[test]
    fn test_compare_all_fields_different_verbose() {
        let txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();

        txs2[0].tx_id = TxId(1);
        txs2[0].tx_type = TxType::Withdrawal;
        txs2[0].from_user_id = TxFromUserId(2);
        txs2[0].to_user_id = TxToUserId(3);
        txs2[0].amount = TxAmount(4);
        txs2[0].timestamp = TxTimestamp(5);
        txs2[0].status = TxStatus::Success;
        txs2[0].description = TxDescription("Different".to_string());

        let differences = compare_transactions(&txs1, &txs2, true, false);
        assert_eq!(differences.len(), 8);

        let fields: Vec<&str> = differences.iter().map(|d| d.field.as_str()).collect();
        assert!(fields.contains(&"tx_id"));
        assert!(fields.contains(&"tx_type"));
        assert!(fields.contains(&"from_user_id"));
        assert!(fields.contains(&"to_user_id"));
        assert!(fields.contains(&"amount"));
        assert!(fields.contains(&"timestamp"));
        assert!(fields.contains(&"status"));
        assert!(fields.contains(&"description"));
    }

    #[test]
    fn test_compare_with_withdrawal_negative_amount() {
        let mut txs1 = create_test_transactions();
        let mut txs2 = create_test_transactions();

        txs1[2].amount = TxAmount(-5000);
        txs2[2].amount = TxAmount(-5000);

        let differences = compare_transactions(&txs1, &txs2, false, false);
        assert!(differences.is_empty());
    }
}

