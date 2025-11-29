use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

// TODO - Move to archive/info
#[derive(Debug, Default)]
pub struct ArchiveInfo {
    pub files: Vec<PathBuf>,
    pub top_level_dirs: HashSet<PathBuf>,
    pub file_counts_by_extension: HashMap<String, usize>,
    pub total_files: usize, // count of non-directory entries
}

impl ArchiveInfo {
    pub fn count_ext(&self, ext: &str) -> usize {
        self.file_counts_by_extension
            .get(&ext.to_ascii_lowercase())
            .copied()
            .unwrap_or(0)
    }

    pub fn single_top_level_dir(&self) -> Option<PathBuf> {
        if self.top_level_dirs.len() == 1 {
            self.top_level_dirs.iter().next().cloned()
        } else {
            None
        }
    }
}
