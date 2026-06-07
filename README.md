# CLI

## Parser

__Available formats:__

- __csv__ — CSV format
- __bin__ — binary format
- __txt__ or __text__ — text format

__description__
```bash
converter <INPUT_FILE> <OUTPUT_FILE>
```

### dev
__example__
```bash
# Convertation CSV format into binary format
cargo run -p converter -- input.csv output.bin

# Convertation binary format into text format
cargo run -p converter -- input.bin output.txt

# Convertation text format into CSV format
cargo run -p converter -- input.txt output.csv
```

### build
__example__
```bash
# cargo build --release
./target/release/converter data/transactions.csv data/transactions.bin
./target/release/converter data/transactions.bin data/transactions.txt
```

## Comparer

__Available formats:__

- __csv__ — CSV format
- __bin__ — binary format  
- __txt__ or __text__ — text format

__description__
```bash
comparer <LEFT_FILE> <RIGHT_FILE> [OPTIONS]
```

#### OPTIONS
```text
-v, --verbose | Show all differences (by default shows only first)
--strict	  | Strict mode: different record counts treated as error
```

### dev
__example__
```bash
# Comparing CSV format with binary format
cargo run -p comparer -- left_file.csv right_file.bin

# Comparing binary format with text format
cargo run -p comparer -- left_file.bin right_file.txt

# Comparing text format with CSV format
cargo run -p comparer -- input.txt output.csv
```

### build
__example__
```bash
# cargo build --release
./target/release/comparer data/transactions.csv data/transactions.bin
./target/release/comparer data/transactions.bin data/transactions.txt
```

