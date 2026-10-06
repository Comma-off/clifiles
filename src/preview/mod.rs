pub mod archive;
pub mod image;
pub mod text;

use crate::fs::entry::{human_size, FileEntry};
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::path::Path;
use text::StyledLine;

pub enum Preview {
    Loading,
    Dir(Vec<FileEntry>),
    Text(Vec<StyledLine>),
    Image(Box<StatefulProtocol>),
    Archive(Vec<String>),
    Binary { size: u64 },
    Error(String),
}

const ARCHIVE_EXTS: &[&str] = &["zip", "tar", "gz", "tgz", "7z"];

pub fn load(path: &Path, picker: &Picker, preview_area: Rect) -> Preview {
    if path.is_dir() {
        return match crate::fs::list_dir(path, &crate::fs::ListOptions::default()) {
            Ok(entries) => Preview::Dir(entries),
            Err(e) => Preview::Error(e.to_string()),
        };
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    if image::is_image(path) {
        return match image::load(path, picker, preview_area) {
            Ok(proto) => Preview::Image(Box::new(proto)),
            Err(e) => Preview::Error(format!("image decode failed: {e}")),
        };
    }

    let name = path.to_string_lossy().to_lowercase();
    let is_archive = ext.as_deref().is_some_and(|e| ARCHIVE_EXTS.contains(&e)) || name.ends_with(".tar.gz");
    if is_archive {
        return match archive::list_entries(path) {
            Ok(entries) => Preview::Archive(entries),
            Err(e) => Preview::Error(format!("archive read failed: {e}")),
        };
    }

    match text::load(path) {
        Ok(text::TextPreview::Lines(lines)) => Preview::Text(lines),
        Ok(text::TextPreview::Binary) => {
            let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            Preview::Binary { size }
        }
        Err(e) => Preview::Error(e.to_string()),
    }
}

pub fn binary_label(size: u64) -> String {
    format!("binary file ({})", human_size(size))
}
