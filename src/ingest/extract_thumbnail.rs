use std::fs::create_dir_all;
use std::path::Path;
use crate::_utils::run_command::ShellReturn;
use crate::sh;
use std::process::Command;
use anyhow::anyhow;

pub fn extract_thumbnail_full(path: &str, output_dir: &str, filename: &str) -> anyhow::Result<()> {
    create_dir_all(output_dir).map_err(|e| anyhow!("Failed to create thumbnail directory {}: {}", output_dir, e))?;
    
    let result = sh!("dcraw -c -w -q 3 {} | cjpeg > {}", path, Path::new(output_dir).join(filename).to_str().unwrap());
    match result.err_code {
        0 => Ok(()),
        err_code => Err(anyhow!("Error {}: {}", err_code, result.stderr)),
    }
}