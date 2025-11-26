use std::path::{Path, PathBuf};

/// Extension trait to add a `prefix` method to `Path` and `PathBuf`
/// that concatenates paths without the replacement behavior of `join`.
///
/// The standard library's `join` method treats absolute paths specially by
/// replacing the base path entirely. This trait provides an alternative that
/// always appends the path, regardless of whether it's absolute or relative.
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
/// # use path_prefix::PathPrefix;
///
/// let base = PathBuf::from("/home/user/files");
/// let file = Path::new("/document.txt");
///
/// // With join (replacement behavior):
/// let joined = base.join(file);
/// assert_eq!(joined, PathBuf::from("/document.txt"));
///
/// // With prefix (concatenation behavior):
/// let prefixed = file.prefix(&base);
/// assert_eq!(prefixed, PathBuf::from("/home/user/files/document.txt"));
/// ```
pub trait PathPrefix {
    /// Prefixes this path with another path, without treating absolute paths specially.
    ///
    /// Unlike [`Path::join`], this method will always append the path components,
    /// even if the path being appended starts with a root directory (`/` on Unix
    /// or a drive letter on Windows).
    ///
    /// Leading path separators and root components are stripped from `self` before
    /// appending to the base path.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::{Path, PathBuf};
    /// # use path_prefix::PathPrefix;
    ///
    /// let base = PathBuf::from("/var/data");
    ///
    /// // Absolute path becomes relative
    /// let abs_path = Path::new("/file.txt");
    /// assert_eq!(abs_path.prefix(&base), PathBuf::from("/var/data/file.txt"));
    ///
    /// // Relative paths work as expected
    /// let rel_path = Path::new("subdir/file.txt");
    /// assert_eq!(rel_path.prefix(&base), PathBuf::from("/var/data/subdir/file.txt"));
    /// ```
    fn prefix<P: AsRef<Path>>(&self, base: P) -> PathBuf;
}

impl PathPrefix for Path {
    fn prefix<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        let mut result = base.as_ref().to_path_buf();

        // Get the path as a string and strip leading separators
        if let Some(s) = self.to_str() {
            let stripped = s.trim_start_matches(std::path::MAIN_SEPARATOR);
            result.push(stripped);
        } else {
            // Fallback for non-UTF8 paths: iterate through components
            for component in self.components() {
                use std::path::Component;
                match component {
                    Component::RootDir => continue, // Skip root
                    Component::Prefix(_) => continue, // Skip Windows prefixes
                    _ => result.push(component),
                }
            }
        }

        result
    }
}

impl PathPrefix for PathBuf {
    /// Prefixes this `PathBuf` with a base path.
    ///
    /// This is a convenience implementation that delegates to the `Path` implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// # use path_prefix::PathPrefix;
    ///
    /// let base = PathBuf::from("/home/user");
    /// let file = PathBuf::from("/data/file.txt");
    ///
    /// assert_eq!(file.prefix(&base), PathBuf::from("/home/user/data/file.txt"));
    /// ```
    fn prefix<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        self.as_path().prefix(base)
    }
}