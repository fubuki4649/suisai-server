use infer::get_from_path;
use infer::MatcherType::Image;
use std::path::{Path, PathBuf};


/// Recursively traverses a directory to find all image files within
///
/// This function walks through the provided path and all subdirectories to identify
/// and collect paths to image files based on their file type.
///
/// # Arguments
///
/// * `src` - The root path to begin searching from
///
/// # Returns
///
/// A vector of `PathBuf` containing paths to all found image files
///
pub fn get_image_paths(src: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    get_paths_recurse(src, &mut v);
    v
}

fn get_paths_recurse(src: &Path, paths: &mut Vec<PathBuf>) {
    if src.is_file() {
        // Check if file is an image using infer's type detection and matcher comparison
        if Some(Image) == get_from_path(src).ok().flatten().map(|t| t.matcher_type()) {
            // Add the image file's path to the vector
            paths.push(src.to_path_buf());
        }
    } else if src.is_dir() {
        // For directories, get iterator over directory entries
        if let Ok(read_dir) = src.read_dir() {
            // Iterate through directory entries, skipping any that return errors
            for child in read_dir.flatten() {
                // Recursively process each child path
                get_paths_recurse(child.path().as_path(), paths);
            }
        }
    }
}