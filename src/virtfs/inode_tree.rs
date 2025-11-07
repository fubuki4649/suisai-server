use crate::virtfs::inode::Inode;
use fuser::FileType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::mem::swap;
use std::ops::Index;
use std::rc::Rc;

type InodeRef = Rc<RefCell<Inode>>;
type InodeMap = HashMap<u64, InodeRef>;


/// A weird HashMap-based tree that also allows for O(1) lookup of inodes by ino
pub(super) struct InodeTree {
    map: InodeMap,
    next_inode: u64,
    next_inode_generation: u64,
}

impl Index<&u64> for InodeTree {
    type Output = InodeRef;
    fn index(&self, ino: &u64) -> &Self::Output {
        &self.map[ino]
    }
}

impl InodeTree {
    fn increment_ino(&mut self) {
        if self.next_inode == u64::MAX {
            self.next_inode_generation += 1;
            self.next_inode = 2;
        } else {
            self.next_inode += 1;
        }
    }

    /// Helper function for `remove_inode`. Finds and removes the inode with `ino`, but does not
    /// remove the inode from the parent's children list. Requires manual clearing or deletion of
    /// parent's children list
    ///
    /// Does not borrow parent inode
    fn remove_inode_recurse(&mut self, ino: u64) {
        if let Some(inode_ref) = self.map.remove(&ino) {
            let inode = inode_ref.borrow();

            // Recurse over children
            inode.children.iter().for_each(|(_name, child_ino)| {
                self.remove_inode_recurse(*child_ino)
            });
        }
    }
}

impl InodeTree {
    pub(super) fn new() -> Self {
        // The parent of the root node is supposed to be itself; this is not a typo
        let root: InodeRef = Rc::new(RefCell::new(
            Inode::new(1, OsString::from(""), FileType::Directory, 0, None)
        ));

        let mut map: InodeMap = HashMap::new();
        map.insert(1, Rc::clone(&root));

        InodeTree {
            map,
            next_inode: 2,
            next_inode_generation: 0,
        }
    }

    /// Gets an inode by its `ino`. It's O(1) (avg.) because it's hashmap based
    pub(super) fn get(&self, ino: &u64) -> Option<InodeRef> {
        self.map.get(ino).map(|inode_ref| Rc::clone(inode_ref))
    }

    /// Returns `u64` inode number if found; `None` if either `ino` or `name` are not found
    pub(super) fn get_child(&mut self, ino: &u64, name: &OsStr) -> Option<u64> {
        // TODO : Make sure inode children are hydrated + populated before querying
        match self.map.get(&ino) {
            Some(inode) => inode.borrow().children.get(name).copied(),
            None => None,
        }
    }

    /// Check if an `ino` exists
    pub(super) fn contains_key(&self, ino: &u64) -> bool {
        self.map.contains_key(ino)
    }

    /// Create an inode with a free inode number and attach it to the inode tree
    pub(super) fn add(&mut self, parent: u64, name: OsString, filetype: FileType, real_path: Option<OsString>) {
        let parent_ref = self[&parent].clone();
        let new_inode = Inode::new(parent, name.clone(), filetype, self.next_inode_generation, real_path);
        let new_inode_ref = Rc::new(RefCell::new(new_inode));

        // find a free inode number
        while self.map.contains_key(&self.next_inode) {
            self.increment_ino();
        }

        // Insert inode
        self.map.insert(self.next_inode, new_inode_ref.clone());

        // Attach to parent
        let mut parent = parent_ref.borrow_mut();
        parent.children.insert(name, self.next_inode);

        // Update ino / inode generation tracking
        self.increment_ino();
    }

    /// Safely removes an inode from the `InodeTree`
    pub(super) fn remove(&mut self, ino: &u64) {
        // Root node is self-referencing and cannot be removed
        if *ino == 1 { return; }

        if let Some(inode_ref) = self.map.remove(ino) {
            let inode = inode_ref.borrow();

            // Remove from parent
            let parent_ref = self[&inode.parent].clone();
            let mut parent = parent_ref.borrow_mut();
            parent.children.remove(&inode.name);
            drop(parent);

            // Recurse over children
            inode.children.iter().for_each(|(_name, child_ino)| {
                self.remove_inode_recurse(*child_ino)
            });
        }
    }

    /// Checks if an inode is still hydrated; if not, return `false` and clear the children
    pub(super) fn validate_inode(&mut self, ino: u64) -> bool {
        match self.map.get(&ino) {
            Some(inode_ref) => {
                let mut inode = inode_ref.borrow_mut();

                // Clear children if not hydrated
                if !inode.children_hydrated() {

                    // Clear children
                    let mut children: HashMap<OsString, u64> = HashMap::new();
                    swap(&mut inode.children, &mut children);

                    // Drop borrow so we can recurse
                    drop(inode);

                    // Delete children
                    children.iter().for_each(|(_, child_ino)| {
                        self.remove_inode_recurse(*child_ino);
                    });

                    false
                } else { true }
            }
            None => false
        }
    }
}