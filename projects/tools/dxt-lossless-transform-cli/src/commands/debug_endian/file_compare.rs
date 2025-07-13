use super::endian_test::EndianTestError;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a unique random directory name to prevent parallel test conflicts
pub fn generate_random_dir_name(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Add process ID for additional uniqueness
    let pid = std::process::id();

    format!("{}_{}_{}_{}", prefix, timestamp, pid, rand_suffix())
}

/// Generate a random suffix using timestamp and thread ID
fn rand_suffix() -> u32 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();

    // Combine subsecond nanos with thread ID for better uniqueness
    let thread_id = std::thread::current().id();
    let thread_num = format!("{:?}", thread_id)
        .chars()
        .filter(|c| c.is_numeric())
        .collect::<String>()
        .parse::<u32>()
        .unwrap_or(0);

    time.wrapping_add(thread_num)
}

/// Compare two files for byte-for-byte equality
pub fn compare_files(path1: &Path, path2: &Path) -> Result<bool, EndianTestError> {
    // Check if both files exist
    if !path1.exists() {
        return Err(EndianTestError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path1.display()),
        )));
    }

    if !path2.exists() {
        return Err(EndianTestError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path2.display()),
        )));
    }

    // Check file sizes first for early exit optimization
    let metadata1 = fs::metadata(path1)?;
    let metadata2 = fs::metadata(path2)?;

    if metadata1.len() != metadata2.len() {
        return Ok(false);
    }

    // Use efficient buffered reading for large file comparison
    let mut file1 = BufReader::new(File::open(path1)?);
    let mut file2 = BufReader::new(File::open(path2)?);

    const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer for efficient I/O
    let mut buffer1 = vec![0u8; BUFFER_SIZE];
    let mut buffer2 = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes1 = file1.read(&mut buffer1)?;
        let bytes2 = file2.read(&mut buffer2)?;

        // Check if read amounts differ
        if bytes1 != bytes2 {
            return Ok(false);
        }

        // Check if buffer contents differ
        if buffer1[..bytes1] != buffer2[..bytes2] {
            return Ok(false);
        }

        // If we read 0 bytes, we've reached EOF
        if bytes1 == 0 {
            break;
        }
    }

    Ok(true)
}

/// Compare all files in two directories recursively
pub fn compare_directories(dir1: &Path, dir2: &Path) -> Result<bool, EndianTestError> {
    let entries1 = collect_directory_files(dir1)?;
    let entries2 = collect_directory_files(dir2)?;

    // Check if directory structure matches
    if entries1.len() != entries2.len() {
        return Ok(false);
    }

    // Compare each file pair
    for (rel_path, file1) in &entries1 {
        if let Some((_, file2)) = entries2.iter().find(|(path, _)| path == rel_path) {
            if !compare_files(file1, file2)? {
                return Ok(false);
            }
        } else {
            // File exists in dir1 but not in dir2
            return Ok(false);
        }
    }

    Ok(true)
}

/// Recursively collect all files in a directory with relative paths
fn collect_directory_files(dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, EndianTestError> {
    let mut files = Vec::new();
    collect_files_recursive(dir, dir, &mut files)?;
    Ok(files)
}

/// Helper function for recursive file collection
fn collect_files_recursive(
    base_dir: &Path,
    current_dir: &Path,
    files: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), EndianTestError> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let relative_path = path.strip_prefix(base_dir).unwrap().to_path_buf();
            files.push((relative_path, path));
        } else if path.is_dir() {
            collect_files_recursive(base_dir, &path, files)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_comparison_identical() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("test1.bin");
        let file2 = temp_dir.path().join("test2.bin");

        let test_data = b"identical content for testing";
        fs::write(&file1, test_data).unwrap();
        fs::write(&file2, test_data).unwrap();

        assert!(compare_files(&file1, &file2).unwrap());
    }

    #[test]
    fn test_file_comparison_different() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("test1.bin");
        let file2 = temp_dir.path().join("test2.bin");

        fs::write(&file1, b"content one").unwrap();
        fs::write(&file2, b"content two").unwrap();

        assert!(!compare_files(&file1, &file2).unwrap());
    }

    #[test]
    fn test_file_comparison_different_sizes() {
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("test1.bin");
        let file2 = temp_dir.path().join("test2.bin");

        fs::write(&file1, b"short").unwrap();
        fs::write(&file2, b"much longer content").unwrap();

        assert!(!compare_files(&file1, &file2).unwrap());
    }

    #[test]
    fn test_random_dir_name_generation() {
        let name1 = generate_random_dir_name("test");
        let name2 = generate_random_dir_name("test");

        // Names should be different (extremely high probability)
        assert_ne!(name1, name2);

        // Names should start with prefix
        assert!(name1.starts_with("test_"));
        assert!(name2.starts_with("test_"));
    }

    #[test]
    fn test_directory_comparison() {
        let temp_dir = tempdir().unwrap();

        // Create two identical directory structures
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");
        fs::create_dir_all(&dir1).unwrap();
        fs::create_dir_all(&dir2).unwrap();

        // Add identical files
        fs::write(dir1.join("file1.txt"), b"content").unwrap();
        fs::write(dir2.join("file1.txt"), b"content").unwrap();

        assert!(compare_directories(&dir1, &dir2).unwrap());

        // Modify one file
        fs::write(dir2.join("file1.txt"), b"different").unwrap();

        assert!(!compare_directories(&dir1, &dir2).unwrap());
    }
}
