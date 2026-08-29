use crate::models::asset::NewDbAsset;
use chrono::{DateTime, NaiveDateTime, Utc};
use exiftool_rs::{image_info, ImageInfo};
use std::fs;
use std::path::PathBuf;
use xxhash_rust::xxh3::xxh3_128;

/// A trait providing methods to extract metadata from an image file path
/// and convert it into a database-compatible format.
pub trait SuisaiAsset {
    /// Gets the `xxh3_128` content hash of the asset file
    fn get_hash(&self) -> String;

    /// On-disk size of the asset in KB
    fn get_size_on_disk(&self) -> i64;

    /// The date/time the photo was taken, in UTC
    fn get_photo_date(&self, info: &ImageInfo) -> DateTime<Utc>;

    /// The timezone where the photo was taken, as a UTC offset. Defaults to JST (UTC+9).
    fn get_photo_timezone(&self, info: &ImageInfo) -> String;

    /// Returns a `Vec<i64>` of length 2 representing the dimensions of the image (width, height)
    fn get_resolution(&self, info: &ImageInfo) -> Vec<i64>;

    /// The MIME type of the image
    fn get_mime(&self, info: &ImageInfo) -> String;

    /// The model of the camera used to take the image
    fn get_camera_model(&self, info: &ImageInfo) -> String;

    /// The model of the lens used to take the image
    fn get_lens_model(&self, info: &ImageInfo) -> String;

    /// The shutter count of the camera when the image was taken.
    fn get_shutter_count(&self, info: &ImageInfo) -> i64;

    /// The focal length used to take the image, in mm
    fn get_focal_length(&self, info: &ImageInfo) -> i16;

    /// ISO sensitivity of the camera when the image was taken
    fn get_iso(&self, info: &ImageInfo) -> i64;

    /// The shutter speed used to take the photo
    fn get_shutter_speed(&self, info: &ImageInfo) -> String;

    /// The aperture setting (f-stop) used to take the photo
    fn get_aperture(&self, info: &ImageInfo) -> f32;

    /// Returns a `crate::models::asset::NewDbAsset`.
    fn to_db_entry(&self) -> NewDbAsset;
}

impl SuisaiAsset for PathBuf {
    fn get_hash(&self) -> String {
        format!("{:032x}", xxh3_128(&fs::read(self).unwrap_or_default()))
    }

    fn get_size_on_disk(&self) -> i64 {
        fs::metadata(self).map(|m| m.len().div_ceil(1024) as i64).unwrap_or(0)
    }

    fn get_photo_date(&self, info: &ImageInfo) -> DateTime<Utc> {
        info.get("DateTimeOriginal")
            .and_then(|s| NaiveDateTime::parse_from_str(s.trim(), "%Y:%m:%d %H:%M:%S").ok())
            .map(|ndt| ndt.and_utc())
            .unwrap_or_default()
    }

    fn get_photo_timezone(&self, info: &ImageInfo) -> String {
        info.get("OffsetTimeOriginal")
            .filter(|tz| tz.len() == 6 && (tz.starts_with('+') || tz.starts_with('-')))
            .cloned()
            .unwrap_or_else(|| "+09:00".to_string())
    }

    fn get_resolution(&self, info: &ImageInfo) -> Vec<i64> {
        vec![
            info.get("ImageWidth").and_then(|s| s.parse().ok()).unwrap_or(0),
            info.get("ImageHeight").and_then(|s| s.parse().ok()).unwrap_or(0),
        ]
    }

    fn get_mime(&self, info: &ImageInfo) -> String {
        info.get("MIMEType").cloned().unwrap_or_else(|| "application/octet-stream".to_string())
    }

    fn get_camera_model(&self, info: &ImageInfo) -> String {
        info.get("Model").cloned().unwrap_or_else(|| "Unknown Camera".to_string())
    }

    fn get_lens_model(&self, info: &ImageInfo) -> String {
        ["LensModel", "Lens"].iter()
            .find_map(|&tag| info.get(tag).filter(|s| !s.is_empty()).cloned())
            .unwrap_or_else(|| "Unknown Lens".to_string())
    }

    fn get_shutter_count(&self, info: &ImageInfo) -> i64 {
        const SHUTTER_TAGS: &[&str] = &[
            "ShutterCount",
            "MechanicalShutterCount",
            "ImageCount",
            "ShutterCount2",
            "ShutterCount3",
            "TotalShutterCount",
        ];

        SHUTTER_TAGS
            .iter()
            .filter_map(|&tag| info.get(tag))
            .filter_map(|raw| raw.parse::<i64>().ok())
            .find(|&count| count != 0)
            .unwrap_or(0)
    }

    fn get_focal_length(&self, info: &ImageInfo) -> i16 {
        info.get("FocalLength")
            .and_then(|s| s.split_whitespace().next()?.parse::<f32>().ok())
            .map(|f| f.round() as i16)
            .unwrap_or(0)
    }

    fn get_iso(&self, info: &ImageInfo) -> i64 {
        info.get("ISO")
            .and_then(|s| s.split_whitespace().next()?.parse().ok())
            .unwrap_or(0)
    }

    fn get_shutter_speed(&self, info: &ImageInfo) -> String {
        info.get("ShutterSpeed").cloned().unwrap_or_else(|| "Unknown".to_string())
    }

    fn get_aperture(&self, info: &ImageInfo) -> f32 {
        info.get("Aperture")
            .and_then(|s| s.split_whitespace().next()?.parse().ok())
            .map(|aperture: f32| (aperture * 10.0).round() / 10.0)
            .unwrap_or(0.0)
    }

    fn to_db_entry(&self) -> NewDbAsset {
        let info = image_info(self).unwrap_or_default();
        let res = self.get_resolution(&info);
        NewDbAsset {
            parent_id: None,
            thumbnail_path: None,
            hash: self.get_hash(),
            file_name: self.file_name().unwrap_or_default().to_string_lossy().to_string(),
            size_on_disk: self.get_size_on_disk(),
            photo_date: self.get_photo_date(&info),
            photo_timezone: self.get_photo_timezone(&info),
            resolution_width: res[0],
            resolution_height: res[1],
            mime_type: self.get_mime(&info),
            camera_model: self.get_camera_model(&info),
            lens_model: self.get_lens_model(&info),
            shutter_count: self.get_shutter_count(&info),
            focal_length: self.get_focal_length(&info),
            iso: self.get_iso(&info),
            shutter_speed: self.get_shutter_speed(&info),
            aperture: self.get_aperture(&info),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_sony_shutter_count_extraction() {
        let test_path = PathBuf::from("/home/kaneki/suisai-test/original_raws/_DSC0793.ARW");
        if !test_path.exists() {
            return;
        }

        let info = image_info(&test_path).unwrap();
        let sc = test_path.get_shutter_count(&info);
        assert_eq!(sc, 106164, "Shutter count for _DSC0793.ARW should match Perl exiftool value (106164)");
    }

    #[test]
    fn test_compare_with_perl_exiftool() {
        let json_path = PathBuf::from("/tmp/perl_exiftool_unfiled.json");
        if !json_path.exists() {
            println!("Skipping test_compare_with_perl_exiftool: /tmp/perl_exiftool_unfiled.json not found");
            return;
        }

        let json_str = fs::read_to_string(&json_path).expect("Failed to read JSON");
        let perl_data: Vec<Value> = serde_json::from_str(&json_str).expect("Failed to parse JSON");

        let mut lens_mismatches = Vec::new();
        let mut camera_mismatches = Vec::new();
        let mut shutter_count_mismatches = Vec::new();
        let mut focal_length_mismatches = Vec::new();
        let mut iso_mismatches = Vec::new();
        let mut shutter_speed_mismatches = Vec::new();
        let mut aperture_mismatches = Vec::new();
        let mut date_mismatches = Vec::new();
        let mut tz_mismatches = Vec::new();
        let mut res_mismatches = Vec::new();
        let mut mime_mismatches = Vec::new();

        for item in &perl_data {
            let source_file = item["SourceFile"].as_str().unwrap();
            let p = PathBuf::from(source_file);
            let info = image_info(&p).unwrap();

            // 1. Lens Model
            let rs_lens = p.get_lens_model(&info);
            let perl_lens = item.get("LensModel")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .or_else(|| item.get("Lens").and_then(|v| v.as_str()).filter(|s| !s.is_empty()))
                .unwrap_or("Unknown Lens");
            if rs_lens != perl_lens {
                lens_mismatches.push((source_file.to_string(), rs_lens, perl_lens.to_string()));
            }

            // 2. Camera Model
            let rs_camera = p.get_camera_model(&info);
            let perl_camera = item.get("Model").and_then(|v| v.as_str()).unwrap_or("Unknown Camera");
            if rs_camera != perl_camera {
                camera_mismatches.push((source_file.to_string(), rs_camera, perl_camera.to_string()));
            }

            // 3. Shutter Count
            let rs_sc = p.get_shutter_count(&info);
            let perl_sc = item.get("ImageCount")
                .or_else(|| item.get("ShutterCount"))
                .or_else(|| item.get("Canon:ShutterCount"))
                .and_then(|v| {
                    v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
                })
                .unwrap_or(0);
            if rs_sc != perl_sc {
                shutter_count_mismatches.push((source_file.to_string(), rs_sc, perl_sc));
            }

            // 4. Focal Length
            let rs_fl = p.get_focal_length(&info);
            let perl_fl = item.get("FocalLength")
                .and_then(|v| {
                    if let Some(n) = v.as_f64() {
                        Some(n.round() as i16)
                    } else if let Some(s) = v.as_str() {
                        s.split_whitespace().next().and_then(|part| part.parse::<f32>().ok()).map(|f| f.round() as i16)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            if rs_fl != perl_fl {
                focal_length_mismatches.push((source_file.to_string(), rs_fl, perl_fl));
            }

            // 5. ISO
            let rs_iso = p.get_iso(&info);
            let perl_iso = item.get("ISO")
                .and_then(|v| {
                    v.as_i64().or_else(|| v.as_str().and_then(|s| s.split_whitespace().next().and_then(|p| p.parse().ok())))
                })
                .unwrap_or(0);
            if rs_iso != perl_iso {
                iso_mismatches.push((source_file.to_string(), rs_iso, perl_iso));
            }

            // 6. Shutter Speed
            let rs_ss = p.get_shutter_speed(&info);
            let perl_ss = item.get("ShutterSpeed").and_then(|v| {
                if let Some(n) = v.as_f64() {
                    Some(n.to_string())
                } else {
                    v.as_str().map(|s| s.to_string())
                }
            }).unwrap_or_else(|| "Unknown".to_string());
            if rs_ss != perl_ss {
                shutter_speed_mismatches.push((source_file.to_string(), rs_ss, perl_ss));
            }

            // 7. Aperture
            let rs_ap = p.get_aperture(&info);
            let perl_ap = item.get("Aperture")
                .and_then(|v| {
                    if let Some(n) = v.as_f64() {
                        Some((n as f32 * 10.0).round() / 10.0)
                    } else if let Some(s) = v.as_str() {
                        s.split_whitespace().next().and_then(|part| part.parse::<f32>().ok()).map(|f| (f * 10.0).round() / 10.0)
                    } else {
                        None
                    }
                })
                .unwrap_or(0.0);
            if (rs_ap - perl_ap).abs() > 0.01 {
                aperture_mismatches.push((source_file.to_string(), rs_ap, perl_ap));
            }

            // 8. Photo Date
            let rs_date = p.get_photo_date(&info);
            let perl_date = item.get("DateTimeOriginal")
                .and_then(|v| v.as_str())
                .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S").ok())
                .map(|ndt| ndt.and_utc())
                .unwrap_or_default();
            if rs_date != perl_date {
                date_mismatches.push((source_file.to_string(), rs_date, perl_date));
            }

            // 9. Timezone
            let rs_tz = p.get_photo_timezone(&info);
            let perl_tz = item.get("OffsetTimeOriginal")
                .and_then(|v| v.as_str())
                .filter(|tz| tz.len() == 6 && (tz.starts_with('+') || tz.starts_with('-')))
                .unwrap_or("+09:00");
            if rs_tz != perl_tz {
                tz_mismatches.push((source_file.to_string(), rs_tz, perl_tz.to_string()));
            }

            // 10. Resolution
            let rs_res = p.get_resolution(&info);
            let perl_w = item.get("ImageWidth").and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0);
            let perl_h = item.get("ImageHeight").and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).unwrap_or(0);
            if rs_res != vec![perl_w, perl_h] {
                res_mismatches.push((source_file.to_string(), rs_res, vec![perl_w, perl_h]));
            }

            // 11. MIME
            let rs_mime = p.get_mime(&info);
            let perl_mime = item.get("MIMEType").and_then(|v| v.as_str()).unwrap_or("application/octet-stream");
            if rs_mime != perl_mime {
                mime_mismatches.push((source_file.to_string(), rs_mime, perl_mime.to_string()));
            }
        }

        println!("\n==========================================");
        println!("COMPARISON RESULTS ({} total files):", perl_data.len());
        println!("Lens Model mismatches: {}", lens_mismatches.len());
        for (f, rs, perl) in lens_mismatches.iter().take(10) {
            println!("  [Lens] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("Camera Model mismatches: {}", camera_mismatches.len());
        for (f, rs, perl) in camera_mismatches.iter().take(5) {
            println!("  [Camera] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("Shutter Count mismatches: {}", shutter_count_mismatches.len());
        for (f, rs, perl) in shutter_count_mismatches.iter().take(5) {
            println!("  [ShutterCount] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("Focal Length mismatches: {}", focal_length_mismatches.len());
        for (f, rs, perl) in focal_length_mismatches.iter().take(5) {
            println!("  [FocalLength] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("ISO mismatches: {}", iso_mismatches.len());
        for (f, rs, perl) in iso_mismatches.iter().take(5) {
            println!("  [ISO] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("Shutter Speed mismatches: {}", shutter_speed_mismatches.len());
        for (f, rs, perl) in shutter_speed_mismatches.iter().take(5) {
            println!("  [ShutterSpeed] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("Aperture mismatches: {}", aperture_mismatches.len());
        for (f, rs, perl) in aperture_mismatches.iter().take(5) {
            println!("  [Aperture] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("Photo Date mismatches: {}", date_mismatches.len());
        for (f, rs, perl) in date_mismatches.iter().take(5) {
            println!("  [Date] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("Photo Timezone mismatches: {}", tz_mismatches.len());
        for (f, rs, perl) in tz_mismatches.iter().take(5) {
            println!("  [TZ] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        println!("Resolution mismatches: {}", res_mismatches.len());
        for (f, rs, perl) in res_mismatches.iter().take(5) {
            println!("  [Resolution] {} -> RS: {:?} vs Perl: {:?}", f, rs, perl);
        }

        let p = PathBuf::from("/home/kaneki/suisai-test/storage/unfiled/_DSC0824.ARW");
        let info = image_info(&p).unwrap();
        println!("All keys in exiftool-rs info for _DSC0824.ARW:");
        let mut keys: Vec<_> = info.keys().collect();
        keys.sort();
        for k in keys {
            println!("  {}: {}", k, info.get(k).unwrap());
        }
    }
}