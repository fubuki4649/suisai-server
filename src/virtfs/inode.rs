use std::collections::HashMap;
use std::ffi::OsString;
use std::time::SystemTime;
use fuser::FileType;

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
}