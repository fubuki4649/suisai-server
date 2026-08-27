use anyhow::{anyhow, Context, Result};
use jpeg_encoder::{ColorType, Encoder};
use rawlib::{extract_image_with_options, DecodeOptions};
use std::fs::{create_dir_all, remove_file};
use std::path::Path;

const JPEG_QUALITY: u8 = 88;

/// Renders and creates a full-resolution JPEG from a camera RAW image file.
///
/// # Arguments
///
/// * `input` - Path to the source raw image file
/// * `output` - Path to the destination JPEG file
///
/// # Returns
///
/// Returns `Ok(())` if the image was successfully decoded and saved, otherwise returns 
/// an error with details about what went wrong.
///
/// # Errors
///
/// This function will return an error if:
/// * The output directory cannot be created
/// * The RAW decoding fails
/// * The JPEG encoding fails
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// extract_thumbnail_full(
///     Path::new("photo.NEF"),
///     Path::new("/thumbnails/2024/photo.jpeg")
/// )?;
/// ```
pub fn extract_thumbnail_full<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("Failed to create thumbnail directory {}", parent.display()))?;
    }

    let decode_options = DecodeOptions {
        half_size: false,
        demosaic_quality: 3,
        output_bps: 8,
        no_auto_bright: false,
        output_color: 1,
        linear_gamma: false,
        use_camera_wb: true,
    };

    let image = extract_image_with_options(input_path, &decode_options)
        .map_err(|e| anyhow!("Failed to decode RAW from {}: {e}", input_path.display()))?;

    let mut encoder = Encoder::new_file(output_path, JPEG_QUALITY)
        .with_context(|| format!("Failed to create JPEG output file {}", output_path.display()))?;

    encoder.set_progressive(true);

    if let Err(e) = encoder.encode(&image.data, image.width, image.height, ColorType::Rgb) {
        let _ = remove_file(output_path);
        return Err(anyhow!("Failed to encode JPEG thumbnail for {}: {e}", output_path.display()));
    }

    Ok(())
}