use fuser::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request};
use libc::{EIO, ENOENT};
use std::ffi::OsStr;
use std::time::{Duration, SystemTime};

const TTL: Duration = Duration::from_secs(1);
const HELLO_TXT_CONTENT: &str = "Hello, World!\n";
const INODE_ROOT: u64 = 1;
const INODE_HELLO: u64 = 2;

// Metadata for "hello.txt"
fn hello_file_attr() -> FileAttr {
    FileAttr {
        ino: INODE_HELLO,
        size: HELLO_TXT_CONTENT.len() as u64,
        blocks: 1,
        atime: SystemTime::now(),
        mtime: SystemTime::now(),
        ctime: SystemTime::now(),
        crtime: SystemTime::now(),
        kind: FileType::RegularFile,
        perm: 0o644,
        nlink: 1,
        uid: unsafe { libc::geteuid() },
        gid: unsafe { libc::getegid() },
        rdev: 0,
        blksize: 512,
        flags: 0,
    }
}

// Metadata for root directory
fn root_dir_attr() -> FileAttr {
    FileAttr {
        ino: INODE_ROOT,
        size: 0,
        blocks: 0,
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
        blksize: 512,
        flags: 0,
    }
}

pub(super) struct HelloFS;

impl Filesystem for HelloFS {
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == INODE_ROOT && name == "hello.txt" {
            reply.entry(&TTL, &hello_file_attr(), 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        match ino {
            INODE_ROOT => reply.attr(&TTL, &root_dir_attr()),
            INODE_HELLO => reply.attr(&TTL, &hello_file_attr()),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        if ino != INODE_HELLO {
            reply.error(EIO);
            return;
        }
        let data = HELLO_TXT_CONTENT.as_bytes();
        let start = std::cmp::min(offset as usize, data.len());
        let end = std::cmp::min(start + size as usize, data.len());
        reply.data(&data[start..end]);
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != INODE_ROOT {
            reply.error(ENOENT);
            return;
        }
        let entries = [
            (INODE_ROOT, FileType::Directory, "."),
            (INODE_ROOT, FileType::Directory, ".."),
            (INODE_HELLO, FileType::RegularFile, "hello.txt"),
        ];
        for (i, entry) in entries.iter().enumerate().skip(offset as usize) {
            let (ino, kind, name) = entry;
            if reply.add(*ino, (i + 1) as i64, *kind, name) {
                break;
            }
        }
        reply.ok();
    }
}
