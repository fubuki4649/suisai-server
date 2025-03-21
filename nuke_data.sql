-- Nukes all data from the DB
-- Intended for testing/dev use only
TRUNCATE TABLE albums RESTART IDENTITY CASCADE;
TRUNCATE TABLE photos RESTART IDENTITY CASCADE;
