// @generated automatically by Diesel CLI.

diesel::table! {
    album_album_join (parent_id, album_id) {
        parent_id -> Integer,
        album_id -> Integer,
    }
}

diesel::table! {
    album_photo_join (parent_id, photo_id) {
        parent_id -> Integer,
        photo_id -> Bigint,
    }
}

diesel::table! {
    albums (id) {
        id -> Integer,
        #[max_length = 255]
        album_name -> Varchar,
    }
}

diesel::table! {
    photos (id) {
        id -> Bigint,
        #[max_length = 32]
        hash -> Char,
        #[max_length = 64]
        file_name -> Varchar,
        size_on_disk -> Integer,
        photo_date -> Timestamp,
        photo_timezone -> Varchar,
        resolution_width -> Smallint,
        resolution_height -> Smallint,
        #[max_length = 32]
        mime_type -> Varchar,
        #[max_length = 256]
        camera_model -> Varchar,
        #[max_length = 256]
        lens_model -> Varchar,
        shutter_count -> Integer,
        focal_length -> Smallint,
        iso -> Integer,
        shutter_speed -> Varchar,
        aperture -> Float,
    }
}

diesel::table! {
    thumbnails (id) {
        id -> Bigint,
        thumbnail_path -> Text,
    }
}

diesel::joinable!(album_photo_join -> albums (parent_id));
diesel::joinable!(album_photo_join -> photos (photo_id));
diesel::joinable!(thumbnails -> photos (id));

diesel::allow_tables_to_appear_in_same_query!(
    album_album_join,
    album_photo_join,
    albums,
    photos,
    thumbnails,
);
