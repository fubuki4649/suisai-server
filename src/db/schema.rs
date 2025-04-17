// @generated automatically by Diesel CLI.

diesel::table! {
    album_photos (album_id, photo_id) {
        album_id -> Int4,
        photo_id -> Int8,
    }
}

diesel::table! {
    albums (id) {
        id -> Int4,
        album_name -> Text,
    }
}

diesel::table! {
    photos (id) {
        id -> Int8,
        thumbnail_url -> Text,
        file_name -> Text,
        file_path -> Text,
        size_on_disk -> Text,
        photo_date -> Timestamp,
        #[max_length = 6]
        photo_timezone -> Varchar,
        resolution -> Array<Nullable<Int2>>,
        mime_type -> Text,
        camera_model -> Text,
        lens_model -> Text,
        shutter_count -> Int4,
        focal_length -> Int2,
        iso -> Int4,
        shutter_speed -> Text,
        aperture -> Float4,
    }
}

diesel::joinable!(album_photos -> albums (album_id));
diesel::joinable!(album_photos -> photos (photo_id));

diesel::allow_tables_to_appear_in_same_query!(
    album_photos,
    albums,
    photos,
);
