use fuser::{FileAttr, FileType};
use lru::LruCache;
use std::ffi::OsString;
use std::io;
use std::io::{Error, ErrorKind};
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BLKSIZE: u32 = 4096;


pub(super) struct AttributeCache(pub(super) LruCache<u64, FileAttr>);

impl AttributeCache {
    pub(super) fn contains(&self, ino: &u64) -> bool {
        self.0.contains(ino)
    }

    // TODO: The function signature is a bit of a hack, using Option<&OsString> to also encode for inode type
    //       This is a bit of a hack, but it works so I'm leaving it here for now
    pub(super) fn get(&mut self, ino: u64, real_path: Option<&OsString>) -> io::Result<&FileAttr> {
        self.0.try_get_or_insert(ino, || {
            match real_path {
                // Read file attributes for inodes with a real path (assume its a file)
                Some(path) => {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        Ok(FileAttr {
                            ino,
                            size: metadata.len(),
                            blocks: metadata.blocks(),
                            atime: metadata.accessed().unwrap_or(UNIX_EPOCH),
                            mtime: metadata.modified().unwrap_or(UNIX_EPOCH),
                            ctime: UNIX_EPOCH + Duration::from_secs(metadata.ctime() as u64),
                            crtime: metadata.created().unwrap_or(UNIX_EPOCH),
                            kind: match metadata.file_type() {
                                ft if ft.is_dir() => FileType::Directory,
                                ft if ft.is_file() => FileType::RegularFile,
                                ft if ft.is_symlink() => FileType::Symlink,
                                _ => return Err(Error::new(ErrorKind::InvalidData, "Unknown file type; File is neither a directory, file, or symlink.")),
                            },
                            perm: (metadata.mode() & 0o7777) as u16,
                            nlink: metadata.nlink() as u32,
                            uid: metadata.uid(),
                            gid: metadata.gid(),
                            rdev: metadata.rdev() as u32,
                            blksize: BLKSIZE,
                            flags: 0,
                        })
                    } else {
                        Err(Error::new(ErrorKind::InvalidFilename, "Failed to read metadata. Either bad path or permission issue"))
                    }
                },
                // Return attributes for a directory (assume its a directory)
                None => {
                    Ok(FileAttr {
                        ino,
                        size: BLKSIZE as u64,
                        blocks: (BLKSIZE / 512) as u64,
                        atime: SystemTime::now(),
                        mtime: SystemTime::now(),
                        ctime: SystemTime::now(),
                        crtime: SystemTime::now(),
                        kind: FileType::Directory,
                        perm: 0o755,
                        nlink: 2,
                        uid: unsafe { libc::geteuid() },
                        gid: unsafe { libc::getegid() },
                        rdev: 0,
                        blksize: BLKSIZE,
                        flags: 0,
                    })
                }
            }
        })
    }
}