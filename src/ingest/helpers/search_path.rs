use infer::get_from_path;
use infer::MatcherType::Image;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;

/// Recursively traverses a directory, and sends paths for all applicable assets inside it and its children to a channel
///
/// # Arguments
///
/// * `src` - The root path to begin searching from
///
/// # Returns
///
/// (), or a `tokio::sync::mpsc::error::SendError<PathBuf>`
pub fn search_path_for_assets(src: &Path, sender: &Sender<PathBuf>) -> Result<(), SendError<PathBuf>> {
    if src.is_file() {
        // Check if file is an image using infer's type detection and matcher comparison
        if Some(Image) == get_from_path(src).ok().flatten().map(|t| t.matcher_type()) {
            sender.blocking_send(PathBuf::from(src))?;
        }
    } else if src.is_dir() {
        // For directories, get iterator over directory entries
        if let Ok(read_dir) = src.read_dir() {
            // Iterate through directory entries, skipping any that return errors
            for child in read_dir.flatten() {
                // Recursively process each child path
                search_path_for_assets(child.path().as_path(), sender)?;
            }
        }
    }
    
    Ok(())
}
