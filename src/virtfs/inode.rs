use std::collections::HashMap;
use std::ffi::OsString;
use std::time::SystemTime;
use fuser::FileType;

const INODE_CHILDREN_TTL: u64 = 30;

pub(super) struct Inode {
    // Inode metadata
    pub(super) name: OsString,
    pub(super) kind: FileType,
    pub(super) generation: u64,
    pub(super) real_path: Option<OsString>,

    // Tree structure
    pub(super) parent: u64,
    pub(super) children: HashMap<OsString, u64>,
    pub(super) last_hydrated: SystemTime,
}

impl Inode {
    pub(crate) fn new(parent: u64, name: OsString, kind: FileType, generation: u64, real_path: Option<OsString>) -> Self {
        Inode {
            name,
            kind,
            generation,
            real_path,
            parent,
            children: HashMap::new(),
            last_hydrated: SystemTime::UNIX_EPOCH,
        }
    }

    pub(crate) fn children_hydrated(&self) -> bool {
        match self.last_hydrated.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() < INODE_CHILDREN_TTL,
            Err(_) => false,
        }
    }
}