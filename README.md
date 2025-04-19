<div align="center">

# 水彩 suisai-server

Backend server for suisai

[![GPLv3](https://img.shields.io/badge/license-GPLv3-green)](https://www.gnu.org/licenses/gpl-3.0.en.html#license-text)

</div>

## Dependencies

- `exiftool` - for extracting exif data
- `dcraw` - for reading raw files (thumbnail generation)
- `cjpeg` - for encoding to jpeg (thumbnail generation)

## Setup

### Creating the Database

Install PostgreSQL and create a cluster

#### Arch Linux

    sudo pacman -S postgresql
    sudo -u postgres initdb -D /var/lib/postgres/data

#### Debian / Ubuntu

    sudo apt install postgresql postgresql-client
    # Postgres on Debian automatically creates a cluster for you


Start and enable postgresql

    sudo systemctl enable --now postgresql

Enter psql

    sudo -u postgres psql

And create a user

    CREATE USER yourusername WITH PASSWORD 'yourpassword';
    CREATE DATABASE yourdb OWNER yourusername;
    ALTER USER yourusername WITH SUPERUSER;

Use `\l` and `\du` to verify that the table and user have been created respectively. Use `\q` to exit psql.

### Setting up Database

Create the env file—this will allow Diesel to connect to Postgres

    echo DATABASE_URL=postgres://yourusername:yourpassword@localhost/yourdb > .env

Then, install the Diesel CLI and run migrations to create the necessary tables and columns

    cargo install diesel_cli --no-default-features --features postgres
    diesel migration run


### Creating the `.env` file

Create a `.env` file with the following variables

`$DATABASE_URL` - `postgres://<username>:<password>@<hostname>/<tablename>`

`$STORAGE_ROOT` - Specifies where to store all data (see below)


## Directory Structure

The default storage structure for `Suisai`. Symlinks may be used to relocate subdirectories to another location, volume, etc.

```
$STORAGE_ROOT/
├── thumbs/
├── raws/
└── associated_files/
```