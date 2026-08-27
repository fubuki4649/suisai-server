-- Collections Table
CREATE TABLE collections (
    id CHAR(36) PRIMARY KEY,
    parent_id CHAR(36),         -- NULL = unfiled
    label VARCHAR(255) NOT NULL,

    -- Constraints and Keys
    CONSTRAINT label_not_unfiled CHECK (label != 'unfiled'),
    CONSTRAINT label_prefix CHECK (LOWER(label) NOT LIKE 'suisai_%'),
    FOREIGN KEY (parent_id) REFERENCES collections(id) ON DELETE SET NULL
);

-- Assets Table
CREATE TABLE assets (
    -- System Metadata
    id CHAR(36) PRIMARY KEY,
    parent_id CHAR(36),         -- NULL = root level album / unfiled
    thumbnail_path TEXT,
    hash CHAR(32) NOT NULL,

    -- Technical Metadata
    file_name VARCHAR(255) NOT NULL,
    size_on_disk INT NOT NULL,
    photo_date TIMESTAMP NOT NULL,
    photo_timezone VARCHAR(6) NOT NULL,
    resolution_width INT NOT NULL,
    resolution_height INT NOT NULL,
    mime_type VARCHAR(32) NOT NULL,
    camera_model VARCHAR(256) NOT NULL,
    lens_model VARCHAR(256) NOT NULL,
    shutter_count INT NOT NULL,
    focal_length SMALLINT NOT NULL,
    iso INT NOT NULL,
    shutter_speed VARCHAR(32) NOT NULL,
    aperture FLOAT NOT NULL,

    -- Constraints and Keys
    CONSTRAINT uq_hash UNIQUE (hash),
    FOREIGN KEY (parent_id) REFERENCES collections (id) ON DELETE SET NULL
);
