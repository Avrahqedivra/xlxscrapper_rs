# XLX Dashboard Scraper

A Rust scraper that periodically fetches XLX dashboard pages, parses their data tables into JSON, and writes to disk only when the content has actually changed — making it easy to trigger downstream processes via file-watch.

---

## Features

- Scrapes multiple targets in sequence, each with its own URL, interval, and output directory
- Parses tables anchored by `form[name="frmFilterCallSign"]` inside the nearest ancestor `<table>` whose class contains `"table"`
- Extracts both cell text and image `src` — image columns appear as `columnName_img` in the JSON
- SHA-256 content hashing — the output file date only changes when the data changes
- Stable JSON serialisation via `BTreeMap` — keys are always sorted, preventing false hash mismatches
- Per-target exponential back-off on repeated errors
- HTML parsing via `scraper`; HTTP via `reqwest`

---

## Requirements

- Rust 1.70 or later
- Cargo

---

## Installation

```bash
cargo build --release
```

---

## Project structure

```
src/
  main.rs              # entry point, scrape loop, hash-gated file write
  mods/
    downloader.rs      # DownloaderConfig, PageDownloader (HTTP + timeout)
Cargo.toml
config.json            # scrape targets (see below)
```

---

## Dependencies

Add these to `Cargo.toml`:

```toml
[dependencies]
scraper   = "0.19"
serde     = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow    = "1"
reqwest   = { version = "0.11", features = ["blocking"] }
sha2      = "0.10"
```

---

## Configuration

Edit `config.json` at the project root. It is a JSON array — one object per scrape target:

```json
[
  {
    "url": "https://xlx.reflector.com/index.php",
    "interval_seconds": 5,
    "output_dir": "Dir/subDir1",
    "output_file": "filename.json",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    "timeout_seconds": 30
  },
  {
    "url": "https://xlx.reflector2.com/index.php",
    "interval_seconds": 5,
    "output_dir": "Dir/subDir2",
    "output_file": "filename.json",
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    "timeout_seconds": 30
  }  
]
```

| Field | Type | Description |
|---|---|---|
| `url` | string | Full URL of the XLX dashboard page |
| `interval_seconds` | number | Seconds to wait between scrapes for this target |
| `output_dir` | string | Directory where the output file is written |
| `output_file` | string | Filename for the JSON output (e.g. `filename.json`) |
| `user_agent` | string | `User-Agent` header sent with every request |
| `timeout_seconds` | number | HTTP request timeout in seconds |

A custom config path can be passed as a CLI argument (see Usage below).

---

## Usage

### Development

```bash
cargo run
```

### Production

```bash
cargo build --release
./target/release/xlxscrapper
```

### Custom config path

```bash
cargo run -- /path/to/my-config.json
# or after build:
./target/release/xlxscrapper /path/to/my-config.json
```

---

## Output

Each target writes a single file at:

```
<output_dir>/<output_file>
```

The file contains a JSON array of row objects. Each key is a column header from the table. Columns that contain an image produce two keys:

```json
[
  {
    "Callsign": "W1AW",
    "Flag": "US",
    "Flag_img": "/images/flags/us.png",
    "Status": "Online"
  }
]
```

Keys are always written in alphabetical order (courtesy of `BTreeMap`), ensuring the SHA-256 hash is stable across scrapes with identical data.

The file is only written — and its modification date only updated — when the scraped content differs from the previous scrape. This makes it safe to watch with any file-system watcher.

---

## How content-change detection works

On every scrape the serialised JSON is hashed with SHA-256. The hash is held in memory per target (`Target.last_hash`). If it matches the previous hash the write is skipped entirely:

```
[target=0] 71 rows scraped
  unchanged — skipped write

[target=0] 71 rows scraped
  saved → table_data/xlx933/xlx933.json
```

On the first run `last_hash` is `None`, so the file is always written at startup — ensuring your watcher has a file to read immediately.

---

## Adding a new target

1. Add an entry to `config.json`.
2. Restart the scraper — no code change needed.
3. 