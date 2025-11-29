use std::{collections::{HashMap, HashSet}, fs::{self, File}, path::{Path, PathBuf}};
use zip::ZipArchive;
use crate::archive::{ArchiveError, ArchiveInfo};

/// Helper function for inspecting zips
pub fn inspect_zip(path: &Path) -> Result<ArchiveInfo, ArchiveError> {
    let file = File::open(path).map_err(|source| ArchiveError::Open {
        path: path.to_path_buf(),
        source,
    })?;

    let mut zip = ZipArchive::new(file).map_err(|source| ArchiveError::CentralDirectory {
        path: path.to_path_buf(),
        source,
    })?;

    let mut files = Vec::new();
    let mut top_level_dirs: HashSet<PathBuf> = HashSet::new();
    let mut extension_counts: HashMap<String, usize> = HashMap::new();

    for i in 0..zip.len() {
        let entry =
            zip.by_index(i)
                .map_err(|source| ArchiveError::EntryAccess { index: i, source })?;

        let enclosed = entry
            .enclosed_name()
            .ok_or(ArchiveError::InvalidEntryName { index: i })?;

        if let Some(first) = enclosed.components().next() {
            top_level_dirs.insert(PathBuf::from(first.as_os_str()));
        }

        if !entry.is_dir() {
            if let Some(ext) = enclosed.extension().and_then(|e| e.to_str()) {
                *extension_counts
                    .entry(ext.to_ascii_lowercase())
                    .or_insert(0) += 1;
            }
            files.push(enclosed);
        }
    }

    Ok(ArchiveInfo {
        total_files: files.len(),
        files,
        top_level_dirs,
        file_counts_by_extension: extension_counts,
    })
}

/// Helper function for extracting files
pub fn extract_zip(path: &Path, dest: &Path) -> Result<ArchiveInfo, ArchiveError> {
    let file = File::open(path).map_err(|source| ArchiveError::Open {
        path: path.to_path_buf(),
        source,
    })?;
    let mut zip = ZipArchive::new(file).map_err(|source| ArchiveError::CentralDirectory {
        path: path.to_path_buf(),
        source,
    })?;

    ensure_dir(dest)?;
    let mut info = ArchiveInfo::default();

    for i in 0..zip.len() {
        let mut entry =
            zip.by_index(i)
                .map_err(|source| ArchiveError::EntryAccess { index: i, source })?;
        let enclosed = entry
            .enclosed_name()
            .ok_or(ArchiveError::InvalidEntryName { index: i })?;

        if let Some(first) = enclosed.components().next() {
            info.top_level_dirs
                .insert(PathBuf::from(first.as_os_str()));
        }

        let out_path = dest.join(enclosed);
        if entry.is_dir() {
            ensure_dir(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            ensure_dir(parent)?;
        }

        {
            let mut f = File::create(&out_path).map_err(|source| ArchiveError::FileCreate {
                path: out_path.clone(),
                source,
            })?;
            std::io::copy(&mut entry, &mut f).map_err(|source| ArchiveError::EntryCopy {
                path: out_path.clone(),
                source,
            })?;
        }

        if let Some(ext) = out_path.extension().and_then(|e| e.to_str()) {
            *info
                .file_counts_by_extension
                .entry(ext.to_ascii_lowercase())
                .or_insert(0) += 1;
        }

        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)).map_err(
                |source| ArchiveError::PermissionSet {
                    path: out_path.clone(),
                    source,
                },
            )?;
        }

        let rel = out_path.strip_prefix(dest).map_err(|source| ArchiveError::PathStripPrefix {
            path: out_path.clone(),
            base: dest.to_path_buf(),
            source,
        })?;
        info.files.push(rel.to_path_buf());
    }

    info.total_files = info.files.len();
    Ok(info)
}

/// Helper function to determine root directories of zips
pub fn determine_root_dir(info: &ArchiveInfo, extraction_root: &Path) -> PathBuf {
    if let Some(dir) = info.single_top_level_dir() {
        let candidate = extraction_root.join(&dir);
        if candidate.is_dir() {
            return candidate;
        }
    }
    extraction_root.to_path_buf()
}

/// Ensure's a directory exists, if it doesn't it creates it
pub fn ensure_dir(path: &Path) -> Result<(), ArchiveError> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|source| ArchiveError::DirectoryCreate {
            path: path.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

/// Replaces a symlink or creates it if it doesn't already exist
pub fn replace_symlink_dir(src: &Path, dest: &Path) -> Result<(), ArchiveError> {
    if dest.exists() {
        fs::remove_dir_all(dest).map_err(|source| ArchiveError::RemoveDir {
            path: dest.to_path_buf(),
            source,
        })?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dest).map_err(|source| ArchiveError::SymlinkCreate {
            src: src.to_path_buf(),
            dest: dest.to_path_buf(),
            source,
        })?;
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(src, dest).map_err(|source| ArchiveError::SymlinkCreate {
            src: src.to_path_buf(),
            dest: dest.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}
