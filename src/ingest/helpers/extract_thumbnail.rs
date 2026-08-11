use anyhow::{anyhow, Result};
use std::fs::create_dir_all;
use std::path::Path;

/// Extracts and creates a JPEG thumbnail from a raw image file.
///
/// # Arguments
///
/// * `path` - Path to the input raw image file
/// * `output_dir` - Directory where the thumbnail should be saved 
/// * `filename` - Desired filename for the output JPEG thumbnail
///
/// # Returns
///
/// Returns `Ok(())` if the thumbnail was successfully created, otherwise returns 
/// an error with details about what went wrong.
///
/// # Errors
///
/// This function will return an error if:
/// * The output directory cannot be created
/// * The thumbnail extraction fails
///
/// # Example
///
/// ```no_run
/// extract_thumbnail_full(
///     "photo.NEF",
///     "/thumbnails/2024/",
///     "photo.jpeg"
/// )?;
/// ```
pub fn extract_thumbnail_full(path: &str, output_dir: &str, filename: &str) -> Result<()> {
    create_dir_all(output_dir).map_err(|e| anyhow!("Failed to create thumbnail directory {output_dir}: {e}"))?;

    let output_path = Path::new(output_dir).join(filename);
    rawlib::extract_thumbnail_to_file(path, &output_path)
        .map_err(|e| anyhow!("Failed to extract thumbnail from {path}: {e}"))?;

    Ok(())
}