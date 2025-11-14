-- Your SQL goes here
CREATE TABLE photos (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    hash VARCHAR(32) NOT NULL,
    thumbnail_path TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_name TEXT NOT NULL,
    size_on_disk INT NOT NULL,
    photo_date TIMESTAMP NOT NULL,
    photo_timezone VARCHAR(6) NOT NULL,
    resolution_width SMALLINT NOT NULL,
    resolution_height SMALLINT NOT NULL,
    mime_type TEXT NOT NULL,
    camera_model TEXT NOT NULL,
    lens_model TEXT NOT NULL,
    shutter_count INT NOT NULL,
    focal_length SMALLINT NOT NULL,
    iso INT NOT NULL,
    shutter_speed VARCHAR(32) NOT NULL,
    aperture FLOAT NOT NULL,
    -- Uniqueness constraints (also create indexes automatically)
    UNIQUE KEY uq_hash (hash),
    UNIQUE KEY uq_file_name (file_name)
);

CREATE TABLE albums (
    id INT AUTO_INCREMENT PRIMARY KEY,
    album_name VARCHAR(255) NOT NULL
);

CREATE TABLE album_photo_join (
    parent_id INT NOT NULL,
    photo_id BIGINT NOT NULL,
    PRIMARY KEY (parent_id, photo_id),
    CONSTRAINT fk_album_photo_parent
        FOREIGN KEY (parent_id) REFERENCES albums(id)
            ON DELETE CASCADE,
    CONSTRAINT fk_album_photo_photo
        FOREIGN KEY (photo_id) REFERENCES photos(id)
            ON DELETE CASCADE,
    INDEX idx_photo_id (photo_id)
);

CREATE TABLE album_album_join (
    parent_id INT NOT NULL,
    album_id INT NOT NULL,
    PRIMARY KEY (parent_id, album_id),
    CONSTRAINT fk_album_album_parent
        FOREIGN KEY (parent_id) REFERENCES albums(id)
            ON DELETE CASCADE,
    CONSTRAINT fk_album_album_album
        FOREIGN KEY (album_id) REFERENCES albums(id)
            ON DELETE CASCADE,
    INDEX idx_album_id (album_id)
);
