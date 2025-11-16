<div align="center">

# 水彩 suisai-server

Backend server for suisai

[![GPLv3](https://img.shields.io/badge/license-GPLv3-green)](https://www.gnu.org/licenses/gpl-3.0.en.html#license-text)

</div>

## Dependencies

- `exiftool` - for extracting exif data
- `dcraw` - for reading raw files (thumbnail generation)
- `cjpeg` - for encoding to jpeg (thumbnail generation)
- `libmariadbclient` - for connecting to mariadb

## Setup

Set up MariaDB, and make a copy of `example.env` as `.env`, and fill in the fields

Then, install the Diesel CLI and run migrations to create the necessary tables and columns

    cargo install diesel_cli --no-default-features --features mysql
    diesel migration run
