use fuser::{FileAttr, FileType};
use lru::LruCache;
use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{Error, ErrorKind, Result};
use std::num::NonZeroUsize;
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BLKSIZE: u32 = 4096;


pub(super) struct VirtualFs {
    pub inodes: HashMap<u64, Inode>,
    pub attributes: AttributeCache,
    next_inode: u64,
    next_inode_generation: u64,
}

impl VirtualFs {
    pub(super) fn new() -> Self {
        // Init Cache
        let cache_size = if let Ok(cache_size) = (|| {
            Ok::<usize, anyhow::Error>(env::var("FILE_METADATA_CACHE_SIZE")?.parse::<usize>()?)
        })() {
            cache_size
        } else {
            1048576
        };

        // Create struct
        let mut virtfs = VirtualFs {
            inodes: HashMap::new(),
            attributes: AttributeCache(LruCache::new(NonZeroUsize::new(cache_size).unwrap())),
            next_inode: 1,
            next_inode_generation: 0,
        };

        // Create root node (ino=1)
        virtfs.create_node(None);

        virtfs
    }

    pub(super) fn create_node(&mut self, real_path: Option<OsString>) {
        // Create Inode
        self.inodes.insert(
            self.next_inode,
            Inode::new(self.next_inode_generation, real_path),
        );

        // Update State
        self.next_inode += 1;
        if self.next_inode > u64::MAX {
            self.next_inode_generation += 1;
        }
    }
}


pub(super) struct AttributeCache(LruCache<u64, FileAttr>);

impl AttributeCache {
    pub(super) fn contains(&self, ino: &u64) -> bool {
        self.0.contains(ino)
    }

    pub(super) fn get(&mut self, ino: u64, real_path: Option<&OsString>) -> Result<&FileAttr> {
        self.0.try_get_or_insert(ino, || {
            match real_path {
                // Read file attributes for inodes with a real path (files)
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
                // Return attributes for a directory (directories do not correspond to real paths)
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


pub(super) struct Inode {
    pub(super) generation: u64,
    pub(super) real_path: Option<OsString>,
}

impl Inode {
    fn new(generation: u64, real_path: Option<OsString>) -> Self {
        Inode {
            generation,
            real_path,
        }
    }

    // Returns Inode #
    pub(super) fn get_child(&self, name: &OsStr) -> Option<u64> {
        todo!()
    }

    // Return Hashmap<File name, Inode number>
    pub(super) fn get_children(&self) -> Option<HashMap<OsString, u64>> {
        todo!()
    }
}
