-- Your SQL goes here
CREATE TABLE photos (
    id BIGSERIAL PRIMARY KEY,
    thumbnail_url TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    size_on_disk TEXT NOT NULL,
    photo_date TIMESTAMP NOT NULL,
    photo_timezone TEXT NOT NULL,
    resolution INTEGER[] NOT NULL CHECK (array_length(resolution, 1) = 2),
    mime_type TEXT NOT NULL,
    camera_model TEXT NOT NULL,
    lens_model TEXT NOT NULL,
    shutter_count INTEGER NOT NULL,
    focal_length INTEGER NOT NULL,
    iso INTEGER NOT NULL,
    shutter_speed TEXT NOT NULL,
    aperture NUMERIC NOT NULL
);

CREATE INDEX idx_photos_photo_date ON photos(photo_date);