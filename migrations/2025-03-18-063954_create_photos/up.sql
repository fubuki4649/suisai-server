-- Your SQL goes here
CREATE TABLE photos (
    id BIGSERIAL PRIMARY KEY,
    hash VARCHAR(32) NOT NULL,
    thumbnail_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    size_on_disk INTEGER NOT NULL,
    photo_date TIMESTAMP NOT NULL,
    photo_timezone VARCHAR(6) NOT NULL,
    resolution SMALLINT[] NOT NULL CHECK (
        array_length(resolution, 1) = 2 AND array_position(resolution, NULL) IS NULL
    ),
    mime_type TEXT NOT NULL,
    camera_model TEXT NOT NULL,
    lens_model TEXT NOT NULL,
    shutter_count INTEGER NOT NULL,
    focal_length SMALLINT NOT NULL,
    iso INTEGER NOT NULL,
    shutter_speed TEXT NOT NULL,
    aperture REAL NOT NULL
);

CREATE UNIQUE INDEX idx_photos_file_path_lower_unique ON photos(LOWER(file_path));
CREATE UNIQUE INDEX idx_photos_hash_unique ON photos(hash);
CREATE INDEX idx_photos_photo_date ON photos(photo_date);