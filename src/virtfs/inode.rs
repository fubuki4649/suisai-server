use fuser::FileAttr;
use lru::LruCache;
use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::num::NonZeroUsize;


pub(super) struct InodeManager {
    attr_cache: LruCache<u64, FileAttr>,
    inode_map: HashMap<u64, Inode>,
    next_inode: u64,
    next_inode_generation: u32,
}

impl InodeManager {
    pub(super) fn new() -> Self {
        let cache_size = if let Ok(cache_size) = (|| {
            Ok::<usize, anyhow::Error>(env::var("FILE_METADATA_CACHE_SIZE")?.parse::<usize>()?)
        })() {
            cache_size
        } else {
            1048576
        };

        InodeManager {
            attr_cache: LruCache::new(NonZeroUsize::new(cache_size).unwrap()),
            inode_map: HashMap::new(),
            next_inode: 1,
            next_inode_generation: 0,
        }
    }


    pub(super) fn exists(&self, ino: u64) -> bool {
        self.inode_map.contains_key(&ino)
    }

    pub(super) fn get(&self, ino: u64) -> Option<&Inode> {
        self.inode_map.get(&ino)
    }
    
    pub(super) fn get_attr(&mut self, ino: u64) -> Option<&FileAttr> {
        if self.inode_map.contains_key(&ino) {
            self.attr_cache.get(&ino)
            // TODO : Actually Read FileAttr on cache miss
        } else {
            None
        }
    }
}


pub(super) struct Inode {
    real_path: OsString,
    children: Option<HashMap<OsString, u64>>,
}

impl Inode {
    fn new(is_directory: bool) -> Self {
        Inode {
            real_path: OsString::new(),
            children: if is_directory { Some(HashMap::new()) } else { None },
        }
    }

}