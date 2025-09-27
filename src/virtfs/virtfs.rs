use fuser::{FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request};
use libc::{EIO, ENOENT};
use std::ffi::OsStr;
use std::time::{Duration, SystemTime};
use crate::virtfs::inode::InodeManager;

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

pub(super) struct SuisaiMount {
    inodes: InodeManager
}

impl SuisaiMount {
    pub fn new() -> Self{
        SuisaiMount {
            inodes: InodeManager::new()
        }
    }
}

/// Implementation of the FUSE `Filesystem` trait for `SuisaiMount`.
///
/// This implementation defines the behavior of the virtual filesystem from the perspective
/// of the FUSE kernel driver. It handles requests from the kernel to perform filesystem
/// operations like looking up files, reading directories, and getting file attributes.
///
/// The filesystem presented is a simple, read-only filesystem with a root directory
/// containing a single file named "hello.txt".
impl Filesystem for SuisaiMount {
    /// ### FUSE `lookup` operation.
    ///
    /// This method is called by the kernel when it needs to find a file within a directory. Used by
    /// things like `cd` or `ls` with a specific path
    ///
    /// #### Arguments
    /// * `_req`: Request context (unused in this implementation).
    /// * `parent`: The inode of the directory to search in.
    /// * `name`: The file or directory name to look for.
    /// * `reply`: A `ReplyEntry` object to send the result of the lookup (the file's attributes
    ///   or an error).
    ///
    /// In this implementation, it only recognizes "hello.txt" if the parent is the rootfs directory.
    /// For any other name or parent, it returns `ENOENT` (No such file or directory).
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // TODO: Replace Placeholder
        if parent == INODE_ROOT && name == "hello.txt" {
            reply.entry(&TTL, &hello_file_attr(), 0);
        } else {
            reply.error(ENOENT);
        }
    }

    /// ### FUSE `getattr` operation.
    ///
    /// This method is called by the kernel to retrieve the attributes (metadata) of a file or directory,
    /// identified by its `ino` (inode number). It's function is similar to what is returned by `ls -l`
    ///
    /// #### Arguments
    /// * `_req`: Request context (unused).
    /// * `ino`: The inode number of the file or directory.
    /// * `_fh`: File handle (unused).
    /// * `reply`: A `ReplyAttr` object to send the file's attributes or an error.
    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        match self.inodes.get_attr(ino) {
            Some(attr) => reply.attr(&TTL, attr),
            None => reply.error(ENOENT),
        }
    }

    /// ### FUSE `read` operation.
    ///
    /// This method is called when a process opens a file and issues a read request. It's used for
    /// things like `cat`
    ///
    /// /ᐠ ˵> ⩊ <˵ マ
    ///
    /// #### Arguments
    /// * `_req`: Request context (unused).
    /// * `ino`: The inode of the file to read.
    /// * `_fh`: File handle (unused).
    /// * `offset`: The offset within the file to start reading from.
    /// * `size`: The number of bytes to read.
    /// * `_flags`: Read flags (unused).
    /// * `_lock_owner`: Lock owner (unused).
    /// * `reply`: A `ReplyData` object to send the read data or an error.
    ///
    /// This implementation only handles reading from `INODE_HELLO` ("hello.txt"). It returns
    /// the content of `HELLO_TXT_CONTENT`. For any other inode, it returns `EIO` (I/O Error).
    fn read(&mut self, _req: &Request<'_>, ino: u64, _fh: u64, offset: i64, size: u32, _flags: i32, _lock_owner: Option<u64>, reply: ReplyData) {
        // TODO : Replace Placeholder
        if ino != INODE_HELLO {
            reply.error(EIO);
            return;
        }
        let data = HELLO_TXT_CONTENT.as_bytes();
        let start = std::cmp::min(offset as usize, data.len());
        let end = std::cmp::min(start + size as usize, data.len());
        reply.data(&data[start..end]);
    }

    /// ### FUSE `flush` operation.
    ///
    /// This method is called when a file handle is closed. It can be used to write back
    /// any cached data.
    ///
    /// #### Arguments
    /// * `_req`: Request context (unused).
    /// * `_ino`: Inode of the file (unused).
    /// * `_fh`: File handle (unused).
    /// * `_lock_owner`: Lock owner (unused).
    /// * `reply`: A `ReplyEmpty` object to signal completion.
    ///
    /// Since this is a read-only filesystem (for now), there is no data to flush, so it simply
    /// replies with `ok`.
    fn flush(&mut self, _req: &Request<'_>, _ino: u64, _fh: u64, _lock_owner: u64, reply: fuser::ReplyEmpty) {
        // Nothing to flush in a read-only filesystem
        reply.ok();
    }

    /// ### FUSE `readdir` operation.
    ///
    /// This method is called to read the contents of a directory. It's basically `ls`
    ///
    /// #### Arguments
    /// * `_req`: Request context (unused).
    /// * `ino`: The inode of the directory to read.
    /// * `_fh`: File handle (unused).
    /// * `offset`: The offset within the directory to start reading from.
    /// * `reply`: A `ReplyDirectory` object to send the directory entries.
    ///
    /// This implementation only handles reading the root directory (`INODE_ROOT`). It returns
    /// entries for ".", "..", and "hello.txt". For any other inode, it returns `ENOENT`.
    fn readdir(&mut self, _req: &Request<'_>, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        // TODO : Replace Placeholder
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

    /// ### FUSE `getxattr` operation.
    ///
    /// This method is called to get the value of an extended attribute.
    ///
    /// #### Arguments
    /// * `_req`: Request context (unused).
    /// * `_ino`: Inode of the file (unused).
    /// * `_name`: Name of the extended attribute (unused).
    /// * `_size`: Size of the buffer to store the attribute value (unused).
    /// * `reply`: A `ReplyXattr` object to send the result.
    ///
    /// This filesystem does not support extended attributes (for now), so it returns `ENODATA`
    /// (No data available).
    fn getxattr(&mut self, _req: &Request<'_>, _ino: u64, _name: &OsStr, _size: u32, reply: fuser::ReplyXattr) {
        // No extended attributes supported
        reply.error(libc::ENODATA);
    }

    /// ### FUSE `listxattr` operation.
    ///
    /// This method is called to list the names of extended attributes for a file.
    ///
    /// #### Arguments
    /// * `_req`: Request context (unused).
    /// * `_ino`: Inode of the file (unused).
    /// * `_size`: Size of the buffer to store the attribute names (unused).
    /// * `reply`: A `ReplyXattr` object to send the result.
    ///
    /// This filesystem does not support extended attributes (for now), so it returns a size of 0.
    fn listxattr(&mut self, _req: &Request<'_>, _ino: u64, _size: u32, reply: fuser::ReplyXattr) {
        // No extended attributes supported
        reply.size(0);
    }

    /// ### FUSE `access` operation.
    ///
    /// Corresponds to the `access` syscall.
    ///
    /// This method is called to check if a user has permission to perform a certain
    /// operation on a file.
    ///
    /// #### Arguments
    /// * `_req`: Request context (unused).
    /// * `ino`: The inode of the file or directory.
    /// * `_mask`: The access mode to check (unused).
    /// * `reply`: A `ReplyEmpty` object to signal permission or an error.
    ///
    /// Because this works in addition to standard UNIX permissions, we don't need to enforce anything
    /// extra here for now.
    ///
    /// This is a simplified implementation that grants access as long as the inode exists,
    /// without checking the specific `mask`. For non-existent inodes, it returns `ENOENT`.
    fn access(&mut self, _req: &Request<'_>, ino: u64, _mask: i32, reply: fuser::ReplyEmpty) {
        // Simple implementation: allow access if the inode exists
        match self.inodes.exists(ino) {
            true => reply.ok(),
            false => reply.error(ENOENT),
        }
    }

}
