# Apple Books Data Export

CLI for exporting Apple Books library data from local macOS SQLite databases as JSON or CSV. Opens databases read-only and does not mutate Apple Books data.

## Install

```bash
brew tap ryneandal/books
brew install apple-books-data-export
```

## Usage

Simple usage:

```bash
apple-books-data-export export --format csv --output books.csv
```

Note that unless specified, the data is printed to stdout

### Additional commands

`discover`: Search locally for the Apple Books library database. If found, the path to the database is printed to stdout.

```bash
apple-books-data-export discover
```

`inspect`: Validate the Apple Books library database and print the database path, validation result, table presence, required columns, and row count.

```bash
apple-books-data-export inspect --db /path/to/BKLibrary.sqlite
```

`export`: Export the Apple Books library data as JSON or CSV.

Options:

- `--format`: The format to export the data as. Defaults to JSON.
- `--output`: The path to the output file. Defaults to stdout.
- `--pretty`: Pretty print the JSON output. Defaults to false.

```bash
apple-books-data-export export --format csv --output books.csv
apple-books-data-export export --format json --output books.json --pretty
```

Note that unless the sqlite database is explicitly provided via the `--db` option, the binary automatically searches the following paths for a valid `BKLibrary*.sqlite` database (or another `.sqlite` file with the expected schema):

- `~/Library/Containers/com.apple.iBooksX`
- `~/Library/Containers/com.apple.BKAgentService`
- `~/Library/Group Containers/group.com.apple.iBooks`

## Development

- The project is built using Cargo and can be run with `cargo run -- --help`.
- Tests can be run with `cargo test`.
- The project can be built with `cargo build` and installed with `cargo install --path .`
