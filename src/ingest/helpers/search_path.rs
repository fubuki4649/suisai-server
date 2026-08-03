use std::path::{Path, PathBuf};
use infer::MatcherType::Image;
use infer::get_from_path;

/// Recursively traverses a directory to return a list of all applicable assets inside
///
/// # Arguments
///
/// * `src` - The root path to begin searching from
///
/// # Returns
///
/// A vector of `PathBuf` containing paths to all found applicable assets (currently, just image files)
///
pub fn search_path_for_assets(src: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    search_path_recurse(src, &mut v);
    v
}

fn search_path_recurse(src: &Path, paths: &mut Vec<PathBuf>) {
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
                search_path_recurse(child.path().as_path(), paths);
            }
        }
    }
}