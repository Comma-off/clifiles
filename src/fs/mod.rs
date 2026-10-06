pub mod entry;
pub mod ops;

use anyhow::{Context, Result};
use entry::FileEntry;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Name,
    Size,
    Modified,
}

pub struct ListOptions {
    pub show_hidden: bool,
    pub sort: SortMode,
    pub dirs_first: bool,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort: SortMode::Name,
            dirs_first: true,
        }
    }
}

pub fn list_dir(path: &Path, opts: &ListOptions) -> Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path).with_context(|| format!("reading {}", path.display()))? {
        let entry = entry?;
        let fe = FileEntry::from_dir_entry(&entry)?;
        if fe.is_hidden && !opts.show_hidden {
            continue;
        }
        entries.push(fe);
    }

    entries.sort_by(|a, b| {
        if opts.dirs_first && a.is_dir() != b.is_dir() {
            return b.is_dir().cmp(&a.is_dir());
        }
        match opts.sort {
            SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortMode::Size => b.size.cmp(&a.size),
            SortMode::Modified => b.modified.cmp(&a.modified),
        }
    });

    Ok(entries)
}
