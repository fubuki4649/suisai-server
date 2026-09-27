use infer::get_from_path;
use infer::MatcherType::Image;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;

/// Recursively traverses a directory, and sends `(absolute_path, relative_path)` pairs for all
/// applicable assets inside it and its children to a channel.
///
/// The `relative_path` is computed relative to `src`, preserving directory structure information.
///
/// # Arguments
///
/// * `src` - The root path to begin searching from
///
/// # Returns
///
/// (), or a `tokio::sync::mpsc::error::SendError`
pub fn search_path_for_assets(src: &Path, sender: &Sender<(PathBuf, PathBuf)>) -> Result<(), SendError<(PathBuf, PathBuf)>> {
    search_recursive(src, src, sender)
}

fn search_recursive(root: &Path, current: &Path, sender: &Sender<(PathBuf, PathBuf)>) -> Result<(), SendError<(PathBuf, PathBuf)>> {
    if current.is_file() {
        // Check if file is an image using infer's type detection and matcher comparison
        if Some(Image) == get_from_path(current).ok().flatten().map(|t| t.matcher_type()) {
            let rel = current.strip_prefix(root).unwrap_or(current).to_path_buf();
            sender.blocking_send((current.to_path_buf(), rel))?;
        }
    } else if current.is_dir() {
        // For directories, get iterator over directory entries
        if let Ok(read_dir) = current.read_dir() {
            // Iterate through directory entries, skipping any that return errors
            for child in read_dir.flatten() {
                // Recursively process each child path
                search_recursive(root, child.path().as_path(), sender)?;
            }
        }
    }

    Ok(())
}
