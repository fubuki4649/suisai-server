-- This file should undo anything in `up.sql`

-- Drop indexes first
DROP INDEX IF EXISTS idx_album_photos_album_id;
DROP INDEX IF EXISTS idx_album_photos_photo_id;

-- Drop join table
DROP TABLE IF EXISTS album_photos;

-- Drop albums table
DROP TABLE IF EXISTS albums;