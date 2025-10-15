use fuser::FileType;
use lru::LruCache;
use std::collections::HashMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::num::NonZeroUsize;
use crate::virtfs::fileattr_cache::AttributeCache;


// Structs
pub(super) struct VirtualFs {
    pub inodes: HashMap<u64, Inode>,
    pub attributes: AttributeCache,
    pub(crate) next_inode: u64,
    pub(crate) next_inode_generation: u64,
}

pub(super) struct Inode {
    pub(super) name: OsString,
    pub(super) kind: FileType,
    pub(super) parent: u64,
    pub(super) children: HashMap<OsString, u64>,
    pub(super) generation: u64,
    pub(super) real_path: Option<OsString>,
}


// Impls
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
            inodes: HashMap::new(),
            attributes: AttributeCache(LruCache::new(NonZeroUsize::new(cache_size).unwrap())),
            next_inode: 1,
            next_inode_generation: 0,
        };

        // Create root node (ino=1) with parent = self
        virtfs.create_inode(OsString::from(""), FileType::Directory, 1, None);

        virtfs
    }

    pub(super) fn create_inode(&mut self, name: OsString, filetype: FileType, parent: u64, real_path: Option<OsString>) {
        // Create Inode
        self.inodes.insert(
            self.next_inode,
            Inode::new(name, filetype, parent, self.next_inode_generation, real_path),
        );

        // Update ino / inode generation tracking
        self.next_inode += 1;
        if self.next_inode > u64::MAX {
            self.next_inode_generation += 1;
        }
    }

    /// Returns `u64` inode number if found; `None` if either `ino` or `name` are not found
    pub(super) fn get_child(&self, ino: u64, name: &OsStr) -> Option<u64> {
        // TODO : Replace Placeholder
        match self.inodes.get(&ino) {
            Some(inode) => inode.children.get(name).copied(),
            None => None,
        }
    }

    /// Returns a `&HashMap<OsString, u64>` of all of `ino`'s children; `None` if `ino` is not found
    pub(super) fn get_children(&self, ino: u64) -> Option<&HashMap<OsString, u64>> {
        // TODO : Replace Placeholder
        match self.inodes.get(&ino) {
            Some(inode) => Some(&inode.children),
            None => None,
        }
    }
}

impl Inode {
    pub(crate) fn new(name: OsString, kind: FileType, parent: u64, generation: u64, real_path: Option<OsString>) -> Self {
        Inode {
            name,
            kind,
            parent,
            children: HashMap::new(),
            generation,
            real_path,
        }
    }
}
