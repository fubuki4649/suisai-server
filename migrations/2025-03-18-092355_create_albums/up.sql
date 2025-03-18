-- Your SQL goes here

-- Create albums table
CREATE TABLE albums (
    id SERIAL PRIMARY KEY,
    album_name TEXT NOT NULL
);

-- Create join table for albums and photos
CREATE TABLE album_photos (
    album_id INT NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    photo_id BIGINT NOT NULL REFERENCES photos(id) ON DELETE CASCADE,
    PRIMARY KEY (album_id, photo_id)
);

-- Create indexes for faster lookups
CREATE INDEX idx_album_photos_album_id ON album_photos(album_id);
CREATE INDEX idx_album_photos_photo_id ON album_photos(photo_id);