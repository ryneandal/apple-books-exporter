# Apple Books Data Export

CLI for exporting Apple Books library data from local macOS SQLite databases as JSON or CSV. Opens databases read-only and does not mutate Apple Books data.

**Requirements:** macOS with Apple Books installed (reads the local `BKLibrary` SQLite library only).

## Install

**Homebrew**

```bash
brew tap ryneandal/books
brew install apple-books-data-export
```

**Pre-built binary** — tagged releases publish a universal (Apple Silicon + Intel) binary and checksum on [GitHub Releases](https://github.com/ryneandal/apple-books-exporter/releases).

**From source** — needs a recent stable Rust toolchain (this crate uses Rust 2024 edition):

```bash
cargo install --path .
```

## Usage

The CLI prints a short banner (version and tested Books build) to **stderr** on every run. Export data goes to **stdout** unless you pass `--output`. When piping JSON to another tool, redirect stderr if you want a clean stream, for example:

```bash
apple-books-data-export export --format json 2>/dev/null | jq .
```

### Global options

These apply to every subcommand (`discover`, `inspect`, `export`):

- **`--db`**: Path to a specific SQLite file. If omitted, the tool searches common Apple Books container paths for a valid `BKLibrary*.sqlite` (or another `.sqlite` file with the expected schema).
- **`--debug`**: Verbose messages during database discovery (written to stderr).

### `discover`

Search locally for the Apple Books library database. If found, prints the database path to stdout.

```bash
apple-books-data-export discover
```

### `inspect`

Validate the library database and print the database path, validation result, required table name, required columns, and row count.

```bash
apple-books-data-export inspect --db /path/to/BKLibrary.sqlite
```

### `export`

Export library rows as JSON (default) or CSV.

Options:

- **`--format`**: `json` or `csv`. Defaults to JSON.
- **`--output`**: Write to this file instead of stdout.
- **`--pretty`**: Pretty-print JSON (ignored for CSV).

```bash
apple-books-data-export export --format csv --output books.csv
apple-books-data-export export --format json --output books.json --pretty
```

Unless `--db` is set, the binary searches these locations for a valid library database:

- `~/Library/Containers/com.apple.iBooksX`
- `~/Library/Containers/com.apple.BKAgentService`
- `~/Library/Group Containers/group.com.apple.iBooks`

### Export fields

Each row is one library asset. JSON and CSV use the same logical columns (JSON uses RFC 3339 datetimes where present):

| Field | Notes |
| --- | --- |
| `title` | Required string |
| `author` | Optional |
| `status` | `finished`, `in_progress`, or `not_started_or_unknown` (derived from Apple Books flags and progress) |
| `reading_progress` | Optional fraction |
| `high_watermark_progress` | Optional fraction |
| `finished_at` | Optional UTC datetime |
| `last_opened_at` | Optional UTC datetime |
| `last_engaged_at` | Optional UTC datetime |
| `library_record_created_at` | Optional UTC datetime |
| `asset_guid` | Optional string |
| `genre` | Optional string |

### Compatibility

The CLI banner reports the Apple Books build used for verification during development (currently **Apple Books v8.5 (6570)**). Newer or older Books versions may still work if the `ZBKLIBRARYASSET` schema matches; use `inspect` to confirm the database opens and required columns exist.

## Development

- Run the binary via Cargo: `cargo run -- --help` (subcommands: `cargo run -- discover`, `cargo run -- export --format json`, and so on).
- Tests: `cargo test`.
- Release build: `cargo build --release`.

### Homebrew tap and release automation

The Homebrew formula lives in a separate tap repository: **[ryneandal/homebrew-books](https://github.com/ryneandal/homebrew-books)**. The tap short name is `books`, so `brew tap ryneandal/books` clones that repo and makes formulas such as `apple-books-data-export` available to `brew install`.

**Tap bump (optional)**: On release, the tap repo is checked out, the formula is updated(`Formula/apple-books-data-export.rb`) with the new release URL and `sha256`.

Maintainers: create a fine-grained or classic personal access token with `contents:write` on `ryneandal/homebrew-books`, add it as `HOMEBREW_TAP_TOKEN` under **Settings → Secrets and variables → Actions** for this repo.

## License

MIT — see [LICENSE](LICENSE).
