use std::fs::Metadata;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Dir,
    File,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub is_hidden: bool,
}

impl FileEntry {
    pub fn from_dir_entry(entry: &std::fs::DirEntry) -> std::io::Result<Self> {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let meta = entry.metadata()?;
        Ok(Self::build(path, name, &meta))
    }

    fn build(path: PathBuf, name: String, meta: &Metadata) -> Self {
        let kind = if meta.is_symlink() {
            EntryKind::Symlink
        } else if meta.is_dir() {
            EntryKind::Dir
        } else {
            EntryKind::File
        };
        let is_hidden = name.starts_with('.');
        Self {
            name,
            path,
            kind,
            size: meta.len(),
            modified: meta.modified().ok(),
            is_hidden,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.kind, EntryKind::Dir)
    }
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes}B")
    } else {
        format!("{size:.1}{}", UNITS[unit])
    }
}
