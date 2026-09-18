<div align="center">

# 水彩 suisai-server

Backend server for suisai

[![GPLv3](https://img.shields.io/badge/license-GPLv3-green)](https://www.gnu.org/licenses/gpl-3.0.en.html#license-text)

</div>


## Setup

Make a copy of `example.env` as `.env`, and fill in the fields:

```shell
cp example.env .env
```

Ensure `STORAGE_ROOT`, `THUMBNAIL_ROOT`, and `DATABASE_URL` are configured properly.

This project uses SQLite. Configure it with `DATABASE_URL` in `.env`:

```env
DATABASE_URL="sqlite://suisai.db?mode=rwc"
```

The database file and tables (`collections` & `assets`) are automatically created by SeaORM on initial server startup.

## Installation

Set up `.env` first. Then, modify `User=` and `Group=` in `suisai.service` as necessary. Then

```shell
make install
sudo systemctl enable --now suisai
```

## Development

Start the web server (powered by Axum & SeaORM):

```shell
cargo run -- start-server
```

Ingest raw photo files from a directory:

```shell
cargo run -- ingest /path/to/raws
```
