use crate::virtfs::virtfs::VirtualFs;
use fuser::MountOption;
use std::io;
use std::path::Path;

pub fn mount_fuse(mountpoint: &str) -> io::Result<()> {
    let mount_path = Path::new(mountpoint);

    if !mount_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Mountpoint {mountpoint} does not exist")));
    }

    match fuser::mount2(
        VirtualFs::new(),
        mount_path,
        &[
            MountOption::RO,
            MountOption::AllowOther,
            MountOption::AutoUnmount,
            MountOption::FSName("suisai".to_string())
        ],
    ) {
        Ok(_) => {
            println!("Successfully mounted FUSE at {mountpoint}");
            Ok(())
        }
        Err(e) => Err(e)
    }
}