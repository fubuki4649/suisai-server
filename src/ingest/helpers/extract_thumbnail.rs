use anyhow::{anyhow, Context, Result};
use mozjpeg_rs::Encoder;
use rawlib::{extract_image_with_options, DecodeOptions};
use std::fs::{create_dir_all, remove_file, File};
use std::io::BufWriter;
use std::path::Path;

const JPEG_QUALITY: u8 = 82;

/// Renders and creates a high-efficiency full-resolution JPEG from a camera RAW image file.
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
pub fn extract_thumbnail<P: AsRef<Path>, Q: AsRef<Path>>(input: P, output: Q) -> Result<()> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if let Some(parent) = output_path.parent() {
        create_dir_all(parent)
            .with_context(|| format!("Failed to create thumbnail directory {}", parent.display()))?;
    }

    let decode_options = DecodeOptions {
        half_size: false,
        demosaic_quality: 2,
        output_bps: 8,
        no_auto_bright: true,
        output_color: 1,
        linear_gamma: false,
        use_camera_wb: true,
    };

    let image = extract_image_with_options(input_path, &decode_options)
        .map_err(|e| anyhow!("Failed to decode RAW from {}: {e}", input_path.display()))?;

    let file = File::create(output_path)
        .with_context(|| format!("Failed to create JPEG output file {}", output_path.display()))?;
    let writer = BufWriter::new(file);

    let encoder = Encoder::fastest().quality(JPEG_QUALITY);

    if let Err(e) = encoder.encode_rgb_to_writer(&image.data, image.width as u32, image.height as u32, writer) {
        let _ = remove_file(output_path);
        return Err(anyhow!("Failed to encode JPEG thumbnail for {}: {e}", output_path.display()));
    }

    Ok(())
}