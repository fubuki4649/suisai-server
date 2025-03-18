-- This file should undo anything in `up.sql`
-- Your SQL goes here

-- Drop indexes first
DROP INDEX IF EXISTS idx_photos_photo_date;

-- Drop the photos table
DROP TABLE IF EXISTS photos;