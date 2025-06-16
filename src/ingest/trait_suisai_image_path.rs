use crate::_utils::run_command::ShellReturn;
use crate::db::models::photo::NewPhoto;
use crate::sh;
use chrono::NaiveDateTime;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use xxhash_rust::xxh3::xxh3_128;


/// A trait providing methods to extract metadata from an image file path
/// and convert it into a database-compatible format.
///
/// This trait is designed to read EXIF metadata from image files using `ExifTool`
/// and prepare it for insertion into the database. It handles various camera and
/// lens metadata fields, as well as essential photo attributes like timestamps,
/// file hash, and image dimensions.
///
/// All methods in this trait have a default fallback value if metadata cannot be
/// read, ensuring database operations won't fail due to missing EXIF data.
///
/// The trait is primarily implemented for `PathBuf` to work directly with filesystem paths.
pub trait SuisaiImagePath {
    /// Gets the `xxh3_128` hash of the image
    fn get_hash(&self) -> String;

    /// Size on disk of the image in KB
    fn get_size_on_disk(&self) -> i32;

    /// The date/time the photo was taken, in local time
    fn get_photo_date(&self) -> NaiveDateTime;

    /// The timezone where the photo was taken, as a UTC offset. Defaults to JST (UTC+9).
    fn get_photo_timezone(&self) -> String;

    /// Returns a `Vec<i32>` of length 2 representing the dimensions of the image (x, y)
    fn get_resolution(&self) -> Vec<i16>;

    /// The MIME type of the image
    fn get_mime(&self) -> String;

    /// The model of the camera used to take the image
    fn get_camera_model(&self) -> String;

    /// The model of the lens used to take the image
    fn get_lens_model(&self) -> String;

    /// The shutter count of the camera when the image was taken.
    /// Might not be unique for cameras with electronic shutter.
    fn get_shutter_count(&self) -> i32;

    /// The focal length used to take the image, in mm
    fn get_focal_length(&self) -> i16;

    /// ISO sensitivity of the camera when the image was taken
    fn get_iso(&self) -> i32;

    /// The shutter speed used to take the photo. Usually expressed as a fraction.
    fn get_shutter_speed(&self) -> String;

    /// The aperture setting (f-stop) used to take the photo
    fn get_aperture(&self) -> f32;

    /// Returns a `crate::db::models::NewPhoto`. Does not populate the `thumbnail` field
    fn to_db_entry(&self) -> NewPhoto;
}

impl SuisaiImagePath for PathBuf {
    fn get_hash(&self) -> String {
        let data = fs::read(self).unwrap_or_default();
        let hash = xxh3_128(&data);
        format!("{hash:032x}")
    }

    fn get_size_on_disk(&self) -> i32 {
        let metadata = fs::metadata(self);
        (match metadata {
            Ok(metadata) => metadata.len().div_ceil(1024),
            Err(_) => 0,
        }) as i32
    }

    fn get_photo_date(&self) -> NaiveDateTime {
        let result: ShellReturn = sh!("exiftool -DateTimeOriginal -fast2 -s3 {}", self.to_string_lossy());

        if result.err_code == 0 {
            if let Ok(ndt) = NaiveDateTime::parse_from_str(result.stdout.trim(), "%Y:%m:%d %H:%M:%S") {
                return ndt;
            }
        }

        NaiveDateTime::from_timestamp(0, 0)
    }

    fn get_photo_timezone(&self) -> String {
        let result = sh!("exiftool -s3 -fast2 -OffsetTimeOriginal {}", self.to_string_lossy());

        match result.err_code {
            0 => {
                let tz = result.stdout.trim();
                if tz.len() == 6 && (tz.starts_with('+') || tz.starts_with('-')) {
                    tz.to_string()
                } else {
                    "+09:00".to_string()
                }
            }
            _ => "+09:00".to_string(),
        }
    }


    fn get_resolution(&self) -> Vec<i16> {
        let result = sh!("exiftool -fast2 -s3 -ImageWidth -ImageHeight {}", self.to_string_lossy());

        if result.err_code == 0 {
            let lines: Vec<&str> = result.stdout.lines().collect();
            if lines.len() >= 2 {
                return vec![
                    lines[0].trim().parse::<i16>().unwrap_or(0),
                    lines[1].trim().parse::<i16>().unwrap_or(0),
                ];
            }
        }

        vec![0,0]
    }

    fn get_mime(&self) -> String {
        let result = sh!("exiftool -s3 -fast2 -MIMEType {}", self.to_string_lossy());

        match result.err_code {
            0 => result.stdout.trim().to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    fn get_camera_model(&self) -> String {
        let result = sh!("exiftool -s3 -fast2 -Model {}", self.to_string_lossy());

        match result.err_code {
            0 => result.stdout.trim().to_string(),
            _ => "Unknown Camera".to_string(),
        }
    }

    fn get_lens_model(&self) -> String {
        let result = sh!("exiftool -s3 -fast2 -LensModel {}", self.to_string_lossy());

        // Try `-LensModel` first
        if result.err_code == 0 {
            let lens_model = result.stdout.trim().to_string();
            if !lens_model.is_empty() {
                return lens_model;
            }
        }

        // Try `-Lens` if `-LensModel` returns nothing
        let result = sh!("exiftool -s3 -fast2 -Lens {}", self.to_string_lossy());
        match result.err_code {
            0 => result.stdout.trim().to_string(),
            _ => "Unknown Lens".to_string(),
        }
    }

    fn get_shutter_count(&self) -> i32 {
        let tags = ["ImageCount", "ShutterCount", "Canon:ShutterCount"];

        // Try a bunch of tags because metadata may be inconsistent across various camera brands
        for tag in tags {
            let result = sh!("exiftool -s3 -fast1 -{} {}", tag, self.to_string_lossy());

            if result.err_code == 0 {
                if let Ok(count) = result.stdout.trim().parse::<i32>() {
                    if count != 0 {
                        return count;
                    }
                }
            }
        }
        
        0
    }

    fn get_focal_length(&self) -> i16 {
        let result = sh!("exiftool -s3 -fast2 -FocalLength {}", self.to_string_lossy());

        match result.err_code {
            0 => {
                // result.stdout might look like "50.0 mm"
                let trimmed = result.stdout.split_whitespace().next().unwrap_or("0");
                trimmed.parse::<f32>().unwrap_or(0.0).round() as i16
            }
            _ => 0,
        }
    }


    fn get_iso(&self) -> i32 {
        let result = sh!("exiftool -s3 -fast2 -ISO {}", self.to_string_lossy());

        match result.err_code {
            0 => result.stdout.split_whitespace().next().unwrap_or("0").parse::<i32>().unwrap_or(0),
            _ => 0,
        }
    }


    fn get_shutter_speed(&self) -> String {
        let result = sh!("exiftool -s3 -fast2 -ShutterSpeed {}", self.to_string_lossy());

        match result.err_code {
            0 => result.stdout.trim().to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn get_aperture(&self) -> f32 {
        let result = sh!("exiftool -s3 -fast2 -Aperture {}", self.to_string_lossy());

        match result.err_code {
            0 => result.stdout.split_whitespace().next().unwrap_or("0").parse::<f32>().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn to_db_entry(&self) -> NewPhoto {
        NewPhoto {
            thumbnail_path: "".to_string(),
            hash: self.get_hash(),
            file_name: self.file_name().unwrap_or_default().to_string_lossy().to_string(),
            file_path: self.to_string_lossy().to_string(),
            size_on_disk: self.get_size_on_disk(),
            photo_date: self.get_photo_date(),
            photo_timezone: self.get_photo_timezone(),
            resolution: self.get_resolution().into_iter().map(Some).collect(),
            mime_type: self.get_mime(),
            camera_model: self.get_camera_model(),
            lens_model: self.get_lens_model(),
            shutter_count: self.get_shutter_count(),
            focal_length: self.get_focal_length(),
            iso: self.get_iso(),
            shutter_speed: self.get_shutter_speed(),
            aperture: self.get_aperture(),
        }
    }
}