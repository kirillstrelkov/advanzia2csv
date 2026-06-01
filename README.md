# advanzia2csv

Convert [Advanzia](https://gebührenfrei.de/) statements from PDF to CSV.

## Usage

```bash
advanzia2csv [OPTIONS] <INPUT> <OUTPUT>
```

### Arguments

- `<INPUT>` — Path to PDF file or folder that contains PDF files
- `<OUTPUT>` — Path to output CSV file

### Options

- `--swap-sign` — Swap sign of the amount
- `-l, --log-level <LOG_LEVEL>` — Log level [default: info] [possible values: error, warn, info, debug, trace]
- `-h, --help` — Print help
- `-V, --version` — Print version
