# CLI Parser

__Available formats:__

- __csv__ — CSV format
- __bin__ — binary format
- __txt__ or __text__ — text format

__description__
```bash
converter <INPUT_FORMAT> <OUTPUT_FORMAT> <INPUT_FILE> <OUTPUT_FILE>
```

### dev
__example__
```bash
# Convertation CSV format into binary format
cargo run -p converter -- csv bin input.csv output.bin

# Convertation binary format into text format
cargo run -p converter -- bin txt input.bin output.txt

# Convertation text format into CSV format
cargo run -p converter -- txt csv input.txt output.csv
```

### build
__example__
```bash
# cargo build --release
./target/release/converter csv bin data/transactions.csv data/transactions.bin
./target/release/converter bin txt data/transactions.bin data/transactions.txt
```

