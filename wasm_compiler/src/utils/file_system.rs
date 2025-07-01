use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn ensure_dir_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory {:?}: {}", path, e))?;
        log::info!("Created directory: {:?}", path);
    }
    Ok(())
}

pub fn copy_dir_contents(src: &Path, dest: &Path) -> Result<(), String> {
    ensure_dir_exists(dest)?;
    for entry in WalkDir::new(src).min_depth(1).max_depth(1) { // Iterate over top-level entries
        let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry_path.file_name().ok_or_else(|| "Failed to get file name".to_string())?);

        if entry_path.is_dir() {
            fs::create_dir_all(&dest_path).map_err(|e| format!("Failed to create subdir {:?}: {}", dest_path, e))?;
            copy_dir_recursive(entry_path, &dest_path)?; // If it's a directory, copy recursively
        } else {
            fs::copy(entry_path, &dest_path)
                .map_err(|e| format!("Failed to copy file {:?} to {:?}: {}", entry_path, dest_path, e))?;
        }
    }
    Ok(())
}


pub fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    ensure_dir_exists(dest)?;
    for entry in WalkDir::new(src).min_depth(1) { // Iterate over all entries recursively
        let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
        let src_path = entry.path();
        let relative_path = src_path.strip_prefix(src)
            .map_err(|e| format!("Failed to strip prefix: {}", e))?;
        let dest_path = dest.join(relative_path);

        if src_path.is_dir() {
            ensure_dir_exists(&dest_path)?;
        } else if src_path.is_file() {
            // Ensure parent directory of the destination file exists
            if let Some(parent) = dest_path.parent() {
                ensure_dir_exists(parent)?;
            }
            fs::copy(src_path, &dest_path)
                .map_err(|e| format!("Failed to copy file {:?} to {:?}: {}", src_path, dest_path, e))?;
            log::trace!("Copied {:?} to {:?}", src_path, dest_path);
        }
    }
    Ok(())
}

// Example of a function that might be needed later
#[allow(dead_code)]
pub fn find_file_by_extension(dir: &Path, extension: &str) -> Option<walkdir::DirEntry> {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        if entry.path().extension().map_or(false, |ext| ext == extension) {
            return Some(entry);
        }
    }
    None
}
