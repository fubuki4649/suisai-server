use chrono::NaiveDateTime;
use std::path::PathBuf;

pub trait SuisaiImage {
    /// Size on disk of the image in KB
    fn get_size_on_disk(&self) -> i32;

    /// The date the photo was taken
    fn get_photo_date(&self) -> NaiveDateTime;

    // TODO: get_photo_timezone

    /// Returns a `Vec<i32>` of length 2 representing the dimensions of the image (x, y)
    fn get_resolution(&self) -> Vec<i32>;

    /// The MIME type of the image
    fn get_mime(&self) -> String;

    /// The model of the camera used to take the image
    fn get_camera(&self) -> String;

    /// The model of the lens used to take the image
    fn get_lens(&self) -> String;

    /// The shutter count of the camera when the image was taken.
    /// Might not be unique for cameras with electronic shutter.
    fn get_shutter_count(&self) -> i32;

    /// The focal length used to take the image, in mm
    fn get_focal_length(&self) -> i32;

    /// ISO sensitivity of the camera when the image was taken
    fn get_iso(&self) -> i32;

    /// The shutter speed used to take the photo. Usually expressed as a fraction.
    fn get_shutter_speed(&self) -> String;

    /// The aperture setting (f-stop) used to take the photo
    fn get_aperture(&self) -> f32;
}

impl SuisaiImage for PathBuf {
    fn get_size_on_disk(&self) -> i32 {
        todo!()
    }

    fn get_photo_date(&self) -> NaiveDateTime {
        todo!()
    }

    fn get_resolution(&self) -> Vec<i32> {
        todo!()
    }

    fn get_mime(&self) -> String {
        todo!()
    }

    fn get_camera(&self) -> String {
        todo!()
    }

    fn get_lens(&self) -> String {
        todo!()
    }

    fn get_shutter_count(&self) -> i32 {
        todo!()
    }

    fn get_focal_length(&self) -> i32 {
        todo!()
    }

    fn get_iso(&self) -> i32 {
        todo!()
    }

    fn get_shutter_speed(&self) -> String {
        todo!()
    }

    fn get_aperture(&self) -> f32 {
        todo!()
    }
}