-- Your SQL goes here
CREATE TABLE photos (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    hash CHAR(32) NOT NULL,
    file_name VARCHAR(64) NOT NULL,
    size_on_disk INT NOT NULL,
    photo_date TIMESTAMP NOT NULL,
    photo_timezone VARCHAR(6) NOT NULL,
    resolution_width SMALLINT NOT NULL,
    resolution_height SMALLINT NOT NULL,
    mime_type VARCHAR(32) NOT NULL,
    camera_model VARCHAR(256) NOT NULL,
    lens_model VARCHAR(256) NOT NULL,
    shutter_count INT NOT NULL,
    focal_length SMALLINT NOT NULL,
    iso INT NOT NULL,
    shutter_speed VARCHAR(32) NOT NULL,
    aperture FLOAT NOT NULL,
    -- Uniqueness constraints (also create indexes automatically)
    UNIQUE KEY uq_hash (hash),
    UNIQUE KEY uq_file_name (file_name(64))
);

CREATE TABLE albums (
    id INT AUTO_INCREMENT PRIMARY KEY,
    album_name VARCHAR(255) NOT NULL,
    CONSTRAINT album_name_not_unfiled CHECK (album_name != 'unfiled')
);

CREATE TABLE thumbnails (
    id BIGINT NOT NULL,
    thumbnail_path TEXT NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT fk_thumbnail_photo
        FOREIGN KEY (id) REFERENCES photos(id)
            ON DELETE CASCADE
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
