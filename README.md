### CLI Parser

__Поддерживаемые форматы:__

- csv — CSV формат
- bin — бинарный формат
- txt или text — текстовый формат

```bash
converter <INPUT_FORMAT> <OUTPUT_FORMAT> <INPUT_FILE> <OUTPUT_FILE>
```

_example_
```bash
# Конвертация CSV в бинарный формат
cargo run -p converter -- csv bin input.csv output.bin

# Конвертация бинарного в текстовый формат
cargo run -p converter -- bin txt input.bin output.txt

# Конвертация текстового в CSV
cargo run -p converter -- txt csv input.txt output.csv
```

_example with binary_
```bash
# cargo build --release
./target/release/converter csv bin data/transactions.csv data/transactions.bin
./target/release/converter bin txt data/transactions.bin data/transactions.txt
```

### test
```bash
# Запустить все тесты во всех крейтах
cargo test --workspace

# Запустить тесты с выводом результатов (включая успешные)
cargo test --workspace -- --nocapture

# Тесты библиотеки core
cargo test -p core

# Тесты конвертера
cargo test -p converter

# Тесты с конкретным названием
cargo test -p core test_bin_parser_valid
cargo test -p converter test_csv_to_bin_conversion

# Тесты бинарного формата
cargo test -p core bin_format::

# Тесты CSV формата
cargo test -p core csv_format::

# Тесты текстового формата
cargo test -p core txt_format::

# Конкретный тест
cargo test -p core test_bin_serializer_roundtrip
```

### build
```bash
# Сборка всей рабочей области
cargo build --workspace

# Сборка только библиотеки core
cargo build -p core

# Сборка только конвертера
cargo build -p converter

# Сборка с оптимизацией (release)
cargo build --release
```

### structure
```text
.
├── core/               # Библиотека с парсерами и сериализаторами
│   ├── src/
│   │   ├── lib.rs      # Основные типы и трейты
│   │   ├── bin_format.rs   # Парсер и сериализатор для BIN
│   │   ├── csv_format.rs   # Парсер и сериализатор для CSV
│   │   ├── txt_format.rs   # Парсер и сериализатор для TXT
│   │   └── error.rs        # Типы ошибок
│   └── Cargo.toml
└── converter/          # Утилита для конвертации файлов
    ├── src/
    │   └── main.rs     # CLI интерфейс
    └── Cargo.toml
```
