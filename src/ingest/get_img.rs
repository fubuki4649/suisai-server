use infer::get_from_path;
use infer::MatcherType::Image;
use std::path::{Path, PathBuf};

pub fn get_images_recursive(src: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    get_images_as_path(src, &mut v);
    v
}

fn get_images_as_path(src: &Path, paths: &mut Vec<PathBuf>) {

    if src.is_file() {

        if Some(Image) == get_from_path(src).ok().flatten().map(|t| t.matcher_type()) {
            paths.push(src.to_path_buf());
        }

    } else if src.is_dir() {

        if let Ok(read_dir) = src.read_dir() {
            for child in read_dir.flatten() {
                get_images_as_path(child.path().as_path(), paths);
            }
        }

    }

}