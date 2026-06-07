use clap::Parser;
use core::{Tx, parse_file};
use std::cmp::min;
use std::path::PathBuf;
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
        TxAmount, TxDescription, TxFromUserId, TxId, TxStatus, TxTimestamp, TxToUserId, TxType,
    };

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
}
