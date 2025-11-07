use crate::virtfs::attribute_cache::AttributeCache;
use crate::virtfs::inode_tree::InodeTree;
use lru::LruCache;
use std::env;
use std::num::NonZeroUsize;

pub(super) struct VirtualFs {
    pub inodes: InodeTree,
    pub attributes: AttributeCache,
}


impl VirtualFs {
    pub(super) fn new() -> Self {
        // Init Cache
        let cache_size = if let Ok(cache_size) = (|| {
            Ok::<usize, anyhow::Error>(env::var("FILE_ATTRIBUTE_CACHE_SIZE")?.parse::<usize>()?)
        })() {
            cache_size
        } else {
            1048576
        };

        // Create struct
        let mut virtfs = VirtualFs {
            inodes: InodeTree::new(),
            attributes: AttributeCache(LruCache::new(NonZeroUsize::new(cache_size).unwrap())),
        };

        virtfs
    }
}