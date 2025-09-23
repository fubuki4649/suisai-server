use std::io;
use std::path::Path;
use fuser::MountOption;
use crate::virtfs::fs_struct::HelloFS;

pub fn mount_fuse(mountpoint: &str) -> io::Result<()> {
    let mount_path = Path::new(mountpoint);

    if !mount_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Mountpoint {mountpoint} does not exist")));
    }

    match fuser::mount2(
        HelloFS,
        mount_path,
        &[MountOption::RO, MountOption::FSName("suisai".to_string())],
    ) {
        Ok(_) => {
            println!("Successfully mounted FUSE at {mountpoint}");
            Ok(())
        }
        Err(e) => Err(e)
    }
}