use std::env;
use std::fs;
use std::path::Path;

pub fn check_directories() -> Result<(), anyhow::Error> {
    let storage_root = env::var("STORAGE_ROOT").map_err(|_| anyhow::anyhow!("$STORAGE_ROOT not set"))?;

    // Check if `$STORAGE_ROOT`, `$STORAGE_ROOT/thumbnails` and `$STORAGE_ROOT/raws`, 
    // and `$STORAGE_ROOT/associated_files` exist as directories.
    let paths = vec![
        format!("{}/thumbnails", storage_root),
        format!("{}/raws", storage_root),
        format!("{}/associated_files", storage_root),
    ];

    for path in paths {
        let path = Path::new(&path);
        if path.exists() {
            if path.is_dir() {
                println!("Found existing directory: {}", path.display());
            } else {
                return Err(anyhow::anyhow!("{} exists but is not a directory", path.display()));
            }
        } else {
            fs::create_dir_all(path)?;
            println!("Created new directory: {}", path.display());
        }
    }

    Ok(())
}