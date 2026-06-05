use clap::Parser;
use core::{
    BinParser, BinSerializer, CsvParser, CsvSerializer, Parse, Serialize, TextParser,
    TextSerializer, Tx,
};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

const CSV: &str = "csv";
const BIN: &str = "bin";
const TXT: &str = "txt";
const TEXT: &str = "text";

#[derive(Parser)]
#[command(name = "converter")]
#[command(about = "Convert transaction files between CSV, binary, and text formats", long_about = None)]
struct Cli {
    input_format: String,
    output_format: String,
    input_file: PathBuf,
    output_file: PathBuf,
}

#[derive(Debug, PartialEq)]
enum Format {
    Csv,
    Bin,
    Txt,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            CSV => Ok(Format::Csv),
            BIN => Ok(Format::Bin),
            TXT | TEXT => Ok(Format::Txt),
            _ => Err(format!("Unknown format: '{}'. Supported: csv, bin, txt", s)),
        }
    }
}

enum ParserEnum {
    Csv(CsvParser),
    Bin(BinParser),
    Txt(TextParser),
}

impl Parse for ParserEnum {
    fn parse<R: std::io::Read>(&self, reader: &mut R) -> Result<Vec<Tx>, core::Error> {
        match self {
            ParserEnum::Csv(p) => p.parse(reader),
            ParserEnum::Bin(p) => p.parse(reader),
            ParserEnum::Txt(p) => p.parse(reader),
        }
    }
}

fn get_parser(format: &Format) -> ParserEnum {
    match format {
        Format::Csv => ParserEnum::Csv(CsvParser::new()),
        Format::Bin => ParserEnum::Bin(BinParser::new()),
        Format::Txt => ParserEnum::Txt(TextParser::new()),
    }
}

enum SerializerEnum {
    Csv(CsvSerializer),
    Bin(BinSerializer),
    Txt(TextSerializer),
}

impl Serialize for SerializerEnum {
    fn serialize<W: std::io::Write>(
        &self,
        writer: &mut W,
        transactions: &[Tx],
    ) -> Result<(), core::Error> {
        match self {
            SerializerEnum::Csv(s) => s.serialize(writer, transactions),
            SerializerEnum::Bin(s) => s.serialize(writer, transactions),
            SerializerEnum::Txt(s) => s.serialize(writer, transactions),
        }
    }
}

fn get_serializer(format: &Format) -> SerializerEnum {
    match format {
        Format::Csv => SerializerEnum::Csv(CsvSerializer::new()),
        Format::Bin => SerializerEnum::Bin(BinSerializer::new()),
        Format::Txt => SerializerEnum::Txt(TextSerializer::new()),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let input_format: Format = cli.input_format.parse()?;
    let output_format: Format = cli.output_format.parse()?;

    if !cli.input_file.exists() {
        return Err(format!("Input file not found: {}", cli.input_file.display()).into());
    }

    let input_file = File::open(&cli.input_file)?;
    let mut reader = BufReader::new(input_file);

    let parser = get_parser(&input_format);
    let transactions = parser.parse(&mut reader)?;

    if transactions.is_empty() {
        eprintln!("Warning: No transactions found in input file");
    }

    let output_file = File::create(&cli.output_file)?;
    let mut writer = BufWriter::new(output_file);

    let serializer = get_serializer(&output_format);
    serializer.serialize(&mut writer, &transactions)?;

    writer.flush()?;

    println!("Successfully converted {} transactions", transactions.len());
    println!(
        "  Input:  {} ({})",
        cli.input_file.display(),
        cli.input_format
    );
    println!(
        "  Output: {} ({})",
        cli.output_file.display(),
        cli.output_format
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{
        BinParser, BinSerializer, CsvParser, CsvSerializer, TextParser, TextSerializer, Tx,
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
        ]
    }

    #[test]
    fn test_format_parsing() {
        assert_eq!("csv".parse::<Format>().unwrap(), Format::Csv);
        assert_eq!("bin".parse::<Format>().unwrap(), Format::Bin);
        assert_eq!("txt".parse::<Format>().unwrap(), Format::Txt);
        assert_eq!("text".parse::<Format>().unwrap(), Format::Txt);
        assert_eq!("CSV".parse::<Format>().unwrap(), Format::Csv);
        assert_eq!("BIN".parse::<Format>().unwrap(), Format::Bin);
        assert_eq!("TXT".parse::<Format>().unwrap(), Format::Txt);

        assert!("unknown".parse::<Format>().is_err());
        assert!("".parse::<Format>().is_err());
    }

    #[test]
    fn test_get_parser() {
        assert!(matches!(get_parser(&Format::Csv), ParserEnum::Csv(_)));
        assert!(matches!(get_parser(&Format::Bin), ParserEnum::Bin(_)));
        assert!(matches!(get_parser(&Format::Txt), ParserEnum::Txt(_)));
    }

    #[test]
    fn test_get_serializer() {
        assert!(matches!(
            get_serializer(&Format::Csv),
            SerializerEnum::Csv(_)
        ));
        assert!(matches!(
            get_serializer(&Format::Bin),
            SerializerEnum::Bin(_)
        ));
        assert!(matches!(
            get_serializer(&Format::Txt),
            SerializerEnum::Txt(_)
        ));
    }

    #[test]
    fn test_csv_to_bin_conversion() {
        let transactions = create_test_transactions();

        let mut csv_buffer = Vec::new();
        let csv_serializer = CsvSerializer::new();
        csv_serializer
            .serialize(&mut csv_buffer, &transactions)
            .unwrap();

        let mut csv_reader = Cursor::new(csv_buffer);
        let csv_parser = CsvParser::new();
        let parsed_transactions = csv_parser.parse(&mut csv_reader).unwrap();

        let mut bin_buffer = Vec::new();
        let bin_serializer = BinSerializer::new();
        bin_serializer
            .serialize(&mut bin_buffer, &parsed_transactions)
            .unwrap();

        let mut bin_reader = Cursor::new(bin_buffer);
        let bin_parser = BinParser::new();
        let result = bin_parser.parse(&mut bin_reader).unwrap();

        assert_eq!(result.len(), transactions.len());
        assert_eq!(result[0].tx_id, transactions[0].tx_id);
        assert_eq!(result[0].tx_type, transactions[0].tx_type);
        assert_eq!(result[1].description, transactions[1].description);
    }

    #[test]
    fn test_bin_to_txt_conversion() {
        let transactions = create_test_transactions();

        let mut bin_buffer = Vec::new();
        let bin_serializer = BinSerializer::new();
        bin_serializer
            .serialize(&mut bin_buffer, &transactions)
            .unwrap();

        let mut bin_reader = Cursor::new(bin_buffer);
        let bin_parser = BinParser::new();
        let parsed_transactions = bin_parser.parse(&mut bin_reader).unwrap();

        let mut txt_buffer = Vec::new();
        let txt_serializer = TextSerializer::new();
        txt_serializer
            .serialize(&mut txt_buffer, &parsed_transactions)
            .unwrap();

        let mut txt_reader = Cursor::new(txt_buffer);
        let txt_parser = TextParser::new();
        let result = txt_parser.parse(&mut txt_reader).unwrap();

        assert_eq!(result.len(), transactions.len());
        assert_eq!(result[0].amount, transactions[0].amount);
        assert_eq!(result[1].status, transactions[1].status);
    }

    #[test]
    fn test_txt_to_csv_conversion() {
        let transactions = create_test_transactions();

        let mut txt_buffer = Vec::new();
        let txt_serializer = TextSerializer::new();
        txt_serializer
            .serialize(&mut txt_buffer, &transactions)
            .unwrap();

        let mut txt_reader = Cursor::new(txt_buffer);
        let txt_parser = TextParser::new();
        let parsed_transactions = txt_parser.parse(&mut txt_reader).unwrap();

        let mut csv_buffer = Vec::new();
        let csv_serializer = CsvSerializer::new();
        csv_serializer
            .serialize(&mut csv_buffer, &parsed_transactions)
            .unwrap();

        let mut csv_reader = Cursor::new(csv_buffer);
        let csv_parser = CsvParser::new();
        let result = csv_parser.parse(&mut csv_reader).unwrap();

        assert_eq!(result.len(), transactions.len());
        assert_eq!(result[0].tx_id, transactions[0].tx_id);
        assert_eq!(result[0].tx_type, transactions[0].tx_type);
        assert_eq!(result[1].from_user_id, transactions[1].from_user_id);
    }

    #[test]
    fn test_roundtrip_all_formats() {
        let original = create_test_transactions();

        let formats = [Format::Csv, Format::Bin, Format::Txt];

        for from_format in &formats {
            for to_format in &formats {
                if from_format == to_format {
                    continue;
                }

                let mut buffer1 = Vec::new();
                let serializer1 = get_serializer(from_format);
                serializer1.serialize(&mut buffer1, &original).unwrap();

                let mut reader1 = Cursor::new(&buffer1);
                let parser1 = get_parser(from_format);
                let intermediate = parser1.parse(&mut reader1).unwrap();

                let mut buffer2 = Vec::new();
                let serializer2 = get_serializer(to_format);
                serializer2.serialize(&mut buffer2, &intermediate).unwrap();

                let mut reader2 = Cursor::new(&buffer2);
                let parser2 = get_parser(to_format);
                let result = parser2.parse(&mut reader2).unwrap();

                assert_eq!(
                    result.len(),
                    original.len(),
                    "Failed: {} -> {}",
                    match from_format {
                        Format::Csv => "CSV",
                        Format::Bin => "BIN",
                        Format::Txt => "TXT",
                    },
                    match to_format {
                        Format::Csv => "CSV",
                        Format::Bin => "BIN",
                        Format::Txt => "TXT",
                    }
                );

                for (orig, res) in original.iter().zip(result.iter()) {
                    assert_eq!(orig.tx_id, res.tx_id);
                    assert_eq!(orig.tx_type, res.tx_type);
                    assert_eq!(orig.amount, res.amount);
                    assert_eq!(orig.status, res.status);
                    assert_eq!(orig.description, res.description);
                }
            }
        }
    }

    #[test]
    fn test_parser_enum_delegation() {
        let transactions = create_test_transactions();

        let mut csv_buffer = Vec::new();
        CsvSerializer::new()
            .serialize(&mut csv_buffer, &transactions)
            .unwrap();
        let mut csv_reader = Cursor::new(csv_buffer);
        let parser_enum = ParserEnum::Csv(CsvParser::new());
        let result = parser_enum.parse(&mut csv_reader).unwrap();
        assert_eq!(result.len(), 2);

        let mut txt_buffer = Vec::new();
        let serializer_enum = SerializerEnum::Txt(TextSerializer::new());
        serializer_enum
            .serialize(&mut txt_buffer, &transactions)
            .unwrap();
        assert!(!txt_buffer.is_empty());
    }
}
