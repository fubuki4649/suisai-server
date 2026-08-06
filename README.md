<div align="center">

# 水彩 suisai-server

Backend server for suisai

[![GPLv3](https://img.shields.io/badge/license-GPLv3-green)](https://www.gnu.org/licenses/gpl-3.0.en.html#license-text)

</div>

## Dependencies

- `exiftool` - for extracting EXIF metadata
- `dcraw` - for reading raw files (thumbnail generation)
- `cjpeg` - for encoding to JPEG (thumbnail generation)

## Setup

Make a copy of `example.env` as `.env`, and fill in the fields:

```bash
cp example.env .env
```

Ensure `STORAGE_ROOT`, `THUMBNAIL_ROOT`, and `DATABASE_URL` are configured properly.

For SQLite, set `DATABASE_URL` in `.env`:

```env
DATABASE_URL="sqlite://suisai.db?mode=rwc"
```

The database file and tables (`collections` & `assets`) are automatically created by SeaORM on initial server startup.

## Development

Start the web server (powered by Axum & SeaORM):

```bash
cargo run -- start-server
```

Ingest raw photo files from a directory:

```bash
cargo run -- ingest /path/to/raws
```
