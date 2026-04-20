-- Photos Table
CREATE TABLE photos (
    id TEXT PRIMARY KEY,            -- Stored as TEXT instead of BLOB for type cast convenience
    hash TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    size_on_disk INTEGER NOT NULL,
    photo_date TEXT NOT NULL,       -- SQLite stores dates as ISO8601 strings
    photo_timezone TEXT NOT NULL,
    resolution_width INTEGER NOT NULL,
    resolution_height INTEGER NOT NULL,
    mime_type TEXT NOT NULL,
    camera_model TEXT NOT NULL,
    lens_model TEXT NOT NULL,
    shutter_count INTEGER NOT NULL,
    focal_length INTEGER NOT NULL,
    iso INTEGER NOT NULL,
    shutter_speed TEXT NOT NULL,
    aperture REAL NOT NULL
);

-- Albums Table
CREATE TABLE albums (
    id TEXT PRIMARY KEY,
    album_name TEXT NOT NULL CHECK (album_name != 'unfiled')
);

-- Sharing States
CREATE TABLE sharing_states (
    id TEXT PRIMARY KEY,
    photo_id TEXT,
    album_id TEXT,
    FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE,
    FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE,
    CHECK (
        (photo_id IS NOT NULL AND album_id IS NULL) OR
        (photo_id IS NULL AND album_id IS NOT NULL)
    )
);

-- Thumbnails (1-to-1)
CREATE TABLE thumbnails (
    id TEXT PRIMARY KEY REFERENCES photos(id) ON DELETE CASCADE,
    thumbnail_path TEXT NOT NULL
);

-- Join Tables
CREATE TABLE album_photo_join (
    parent_id TEXT NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    photo_id TEXT NOT NULL REFERENCES photos(id) ON DELETE CASCADE,
    PRIMARY KEY (parent_id, photo_id)
);

CREATE TABLE album_album_join (
    parent_id TEXT NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    album_id TEXT NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    PRIMARY KEY (parent_id, album_id)
);

-- Manual Indexes (SQLite handles PRIMARY KEY and UNIQUE automatically)
CREATE INDEX idx_photo_id ON album_photo_join(photo_id);
CREATE INDEX idx_album_id ON album_album_join(album_id);