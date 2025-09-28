use fuser::{FileAttr, FileType};
use lru::LruCache;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BLKSIZE: u32 = 4096;

pub(super) struct InodeManager {
    attr_cache: LruCache<u64, FileAttr>,
    inode_map: HashMap<u64, Inode>,
    next_inode: u64,
    next_inode_generation: u32,
}

impl InodeManager {
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
        let mut manager = InodeManager {
            attr_cache: LruCache::new(NonZeroUsize::new(cache_size).unwrap()),
            inode_map: HashMap::new(),
            next_inode: 1,
            next_inode_generation: 0,
        };

        // Create root node (ino=1)
        manager.create_node(None);

        manager
    }

    pub(super) fn create_node(&mut self, real_path: Option<OsString>) {
        // Create Inode
        self.inode_map.insert(
            self.next_inode,
            Inode::new(self.next_inode_generation, real_path),
        );

        // Update State
        self.next_inode += 1;
        if self.next_inode > u64::MAX {
            self.next_inode_generation += 1;
        }
    }

    pub(super) fn exists(&self, ino: u64) -> bool {
        self.inode_map.contains_key(&ino)
    }

    pub(super) fn get(&self, ino: u64) -> Option<&Inode> {
        self.inode_map.get(&ino)
    }

    pub(super) fn get_attr(&mut self, ino: u64) -> Option<&FileAttr> {
        match self.attr_cache.contains(&ino) {
            true => self.attr_cache.get(&ino),
            false => {
                match &self.inode_map.get(&ino)?.real_path {
                    // Read file attributes for inodes with a real path (files)
                    Some(path) => {
                        let metadata = std::fs::metadata(path).ok()?;
                        let attr = FileAttr {
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
                                _ => return None,
                            },
                            perm: (metadata.mode() & 0o7777) as u16,
                            nlink: metadata.nlink() as u32,
                            uid: metadata.uid(),
                            gid: metadata.gid(),
                            rdev: metadata.rdev() as u32,
                            blksize: BLKSIZE,
                            flags: 0,
                        };

                        self.attr_cache.put(ino, attr);
                        self.attr_cache.get(&ino)
                    },
                    // Return attributes for a directory (directories do not correspond to real paths)
                    None => {
                        let attr = FileAttr {
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
                        };

                        self.attr_cache.put(ino, attr);
                        self.attr_cache.get(&ino)
                    }
                }

            }
        }
    }
}

pub(super) struct Inode {
    generation: u32,
    real_path: Option<OsString>,
    children: Option<HashMap<OsString, u64>>,
}

impl Inode {
    fn new(generation: u32, real_path: Option<OsString>) -> Self {
        let is_dir = real_path.is_none();
        Inode {
            generation,
            real_path,
            children: if is_dir { Some(HashMap::new()) } else { None },
        }
    }
}
