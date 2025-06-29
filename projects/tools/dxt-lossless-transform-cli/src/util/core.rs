use std::fs;
use std::path::*;

/// Recursively visits directories and collects entries.
///
/// This function traverses the directory tree rooted at `dir`, collecting all
/// directory entries into a vector. If an error occurs while reading a directory
/// or an individual entry, the error is handled gracefully and the function
/// continues with the remaining entries.
///
/// # Arguments
///
/// * `dir`: The directory to start the traversal from.
/// * `entries`: A mutable reference to the vector of entries to populate.
///
/// # Returns
///
/// A `Result` indicating whether the traversal was successful.
pub fn find_all_files(dir: &Path, entries: &mut Vec<fs::DirEntry>) -> std::io::Result<()> {
    // Gracefully handle cases where the directory cannot be read
    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()), // Silently return if directory can't be read
    };

    for entry in dir_entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue, // Skip problematic entries, e.g. those without access.
        };

        let path = entry.path();
        if path.is_dir() {
            // Recursively collect files.
            find_all_files(&path, entries)?;
        } else {
            entries.push(entry);
        }
    }
    Ok(())
}

/// Handles errors from process_dir_entry function by printing to stderr
/// (except for IgnoredByFilter which is silently ignored).
pub fn handle_process_entry_error(result: Result<(), crate::error::TransformError>) {
    if let Err(e) = result {
        match e {
            #[cfg(feature = "debug")]
            crate::error::TransformError::IgnoredByFilter => (),
            _ => eprintln!("{e}"),
        }
    }
}

/// Canonicalizes a CLI path argument, creating the directory if it doesn't exist.
///
/// # Arguments
///
/// * `value` - The path string to canonicalize
///
/// # Returns
///
/// A canonicalized PathBuf on success, or a String error message on failure.
pub fn canonicalize_cli_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);

    // If path doesn't exist, create it
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    // Now we can canonicalize it
    fs::canonicalize(path).map_err(|e| format!("Invalid path: {e}"))
}
