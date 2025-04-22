use crate::_utils::run_command::ShellReturn;
use crate::sh;
use anyhow::anyhow;
use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;


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
/// * The dcraw command fails to process the raw image
/// * The cjpeg command fails to create the JPEG
///
/// # External Dependencies
///
/// Requires the following command line tools to be installed:
/// * dcraw - For raw image processing
/// * cjpeg - For JPEG compression
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
pub fn extract_thumbnail_full(path: &str, output_dir: &str, filename: &str) -> anyhow::Result<()> {
    create_dir_all(output_dir).map_err(|e| anyhow!("Failed to create thumbnail directory {}: {}", output_dir, e))?;

    let result = sh!("dcraw -c -w -q 3 {} | cjpeg > {}", path, Path::new(output_dir).join(filename).to_str().unwrap());
    match result.err_code {
        0 => Ok(()),
        err_code => Err(anyhow!("Error {}: {}", err_code, result.stderr)),
    }
}