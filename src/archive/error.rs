use std::path::PathBuf;

use thiserror::Error;


#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("failed to open archive {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid zip central directory in {path}: {source}")]
    CentralDirectory {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },

    #[error("failed to access zip entry at index {index}: {source}")]
    EntryAccess {
        index: usize,
        #[source]
        source: zip::result::ZipError,
    },

    #[error("zip entry at index {index} had invalid (non-enclosed) name")]
    InvalidEntryName { index: usize },

    #[error("failed to create directory {path}: {source}")]
    DirectoryCreate {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to remove existing directory {path}: {source}")]
    RemoveDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create file {path}: {source}")]
    FileCreate {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed copying data into {path}: {source}")]
    EntryCopy {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed setting permissions on {path}: {source}")]
    PermissionSet {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to create symlink {src} -> {dest}: {source}")]
    SymlinkCreate {
        src: PathBuf,
        dest: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to strip prefix {base} from {path}: {source}")]
    PathStripPrefix {
        path: PathBuf,
        base: PathBuf,
        #[source]
        source: std::path::StripPrefixError,
    },
}
