use crate::virtfs::virtfs::VirtualFs;
use fuser::{FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request};
use libc::{EIO, EISDIR, ENOENT, ENOTDIR};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

const TTL: Duration = Duration::from_secs(1);


/// Implementation of the FUSE `Filesystem` trait for `SuisaiMount`.
///
/// This implementation defines the behavior of the virtual filesystem from the perspective
/// of the FUSE kernel driver. It handles requests from the kernel to perform filesystem
/// operations like looking up files, reading directories, and getting file attributes.
///
/// The filesystem presented is a simple, read-only filesystem with a root directory
/// containing a single file named "hello.txt".
impl Filesystem for VirtualFs {
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
    /// Returns a FileAttr for a file or directory given a parent inode + a filename
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let child_ino = self.inodes.get_child(&parent, name);
        let child_inode = child_ino.and_then(|f| self.inodes.get(&f));

        // Make sure child inode exists (`name: &OsStr` isn't invalid)
        if let Some(inodeRef) = child_inode {
            let inode = inodeRef.borrow();
            let attr = self.attributes.get(child_ino.unwrap(), inode.real_path.as_ref());
            // Make sure metadata is readable (should never be unreadable if the inode exists)
            if let Ok(attr) = attr {
                reply.entry(&TTL, attr, inode.generation);
                return;
            }
        }
        
        reply.error(EIO);
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
        // Make sure inode exists
        match self.inodes.get(&ino) {
            Some(ir) => {
                let inode = ir.borrow();
                let path = inode.real_path.as_ref();

                // Make sure metadata is readable (should never be unreadable if the inode exists)
                match self.attributes.get(ino, path) {
                    Ok(attr) => reply.attr(&TTL, attr),
                    Err(_) => reply.error(ENOENT),
                }
            }
            None => {reply.error(ENOENT)}
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
    fn read(&mut self, _req: &Request<'_>, ino: u64, _fh: u64, offset: i64, size: u32, _flags: i32, _lock_owner: Option<u64>, reply: ReplyData) {
        let inode_ref = self.inodes.get(&ino);

        // Make sure inode exists
        match inode_ref {
            Some(ir) => {
                let inode = ir.borrow();

                // Skip for non-directories
                if inode.kind != FileType::RegularFile {
                    reply.error(EISDIR);
                    return;
                }

                // Open file
                let real_path = inode.real_path.as_ref().unwrap();
                let mut file = match File::open(real_path) {
                    Ok(f) => f,
                    Err(_) => {
                        reply.error(EIO);
                        return;
                    }
                };

                // Seek to offset
                if let Err(_) = file.seek(SeekFrom::Start(offset as u64)) {
                    reply.error(EIO);
                    return;
                }

                // Read up to `size` bytes
                let mut buffer = vec![0u8; size as usize];
                let bytes_read = match file.read(&mut buffer) {
                    Ok(n) => n,
                    Err(_) => {
                        reply.error(EIO);
                        return;
                    }
                };

                // Send data back to kernel
                reply.data(&buffer[..bytes_read]);
            }
            None => {
                reply.error(ENOENT);
                return;
            }
        }
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
    /// It's only meant to be called with a directory, so non-directories (file, symlink, etc)
    /// yields ENOTDIR
    fn readdir(&mut self, _req: &Request<'_>, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        let inodes_ref = self.inodes.get(&ino);

        // Make sure inode exists
        match inodes_ref {
            Some(ir) => {
                let inode = ir.borrow();

                // Skip for non-directories
                if inode.kind != FileType::Directory {
                    reply.error(ENOTDIR);
                    return;
                }

                // Return `.` and `..`
                if offset <= 0 { let _ = reply.add(ino, 1, FileType::Directory, "."); }
                if offset <= 1 {let _ = reply.add(inode.parent, 2, FileType::Directory, ".."); }

                // Return children
                // TODO: Verify children are hydrated + populated before querying
                let children = &inode.children;
                for (i, entry) in children.iter().enumerate().skip(offset as usize) {
                    let (_name, ino) = entry;

                    let child_ref = self.inodes.get(&ino).unwrap();
                    let child = child_ref.borrow();

                    if reply.add(*ino, (i + 3) as i64, child.kind, &child.name) {
                        break;
                    }
                }
                reply.ok();
            }
            None => {
                reply.error(ENOENT);
                return;
            }
        }
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
    fn listxattr(&mut self, _req: &Request<'_>, _ino: u64, size: u32, reply: fuser::ReplyXattr) {
        if size == 0 {
            // Caller is asking how much space they need
            reply.size(0);
        } else {
            // Caller provided a buffer, return empty list
            reply.data(&[]);
        }
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
        match self.inodes.contains_key(&ino) {
            true => reply.ok(),
            false => reply.error(ENOENT),
        }
    }

}
