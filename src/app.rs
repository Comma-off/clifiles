use crate::config::Config;
use crate::fs::entry::FileEntry;
use crate::fs::{self, ListOptions};
use crate::preview::{self, Preview};
use crate::prefetcher::Prefetcher;
use crate::preview_worker::PreviewWorker;
use anyhow::Result;
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingOp {
    Delete,
    Rename,
    Mkdir,
    Touch,
    Paste { cut: bool },
}

pub enum Mode {
    Normal,
    Prompt { op: PendingOp, input: String },
    Confirm { op: PendingOp },
}

pub struct Clipboard {
    pub path: PathBuf,
    pub cut: bool,
}

pub struct App {
    pub cwd: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub list_opts: ListOptions,
    pub mode: Mode,
    pub message: Option<String>,
    pub clipboard: Option<Clipboard>,
    pub preview: Preview,
    pub preview_scroll: u16,
    pub last_click: Option<(Instant, usize)>,
    pub config: Config,
    preview_worker: PreviewWorker,
    preview_generation: u64,
    prefetcher: Prefetcher,
    /// The preview pane's area as of the last frame. Used so the *next*
    /// preview request can pre-resize/encode images to the right size on
    /// the background thread rather than lazily on the UI thread. Starts
    /// zero-sized before the first frame, in which case the worker simply
    /// skips pre-encoding (matching how ratatui-image itself no-ops a
    /// zero-area resize) and the UI thread pays that one-time cost instead.
    pub last_preview_area: Rect,
}

impl App {
    pub fn new(start_dir: PathBuf, config: Config) -> Result<Self> {
        let list_opts = ListOptions {
            show_hidden: config.show_hidden,
            sort: config.sort.into(),
            dirs_first: true,
        };
        let mut app = Self {
            cwd: start_dir,
            entries: Vec::new(),
            selected: 0,
            list_opts,
            mode: Mode::Normal,
            message: None,
            clipboard: None,
            preview: Preview::Loading,
            preview_scroll: 0,
            last_click: None,
            config,
            preview_worker: PreviewWorker::new(),
            preview_generation: 0,
            prefetcher: Prefetcher::new(),
            last_preview_area: Rect::default(),
        };
        app.reload()?;
        Ok(app)
    }

    pub fn reload(&mut self) -> Result<()> {
        self.entries = fs::list_dir(&self.cwd, &self.list_opts)?;
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        self.prefetch_visible_images();
        Ok(())
    }

    /// Warms the thumbnail cache for images in the current listing, up
    /// front, the moment a directory is opened — not just the handful of
    /// neighbors around the current selection. Mirrors how yazi precaches:
    /// the decode work is unavoidable either way, but doing it across a
    /// thread pool ahead of time means it's usually already done by the
    /// time the user scrolls to any given file. Capped so opening a folder
    /// with thousands of photos doesn't queue unbounded background work.
    fn prefetch_visible_images(&self) {
        const MAX_PREFETCH: usize = 80;
        let paths = self
            .entries
            .iter()
            .enumerate()
            .filter(|(i, e)| *i != self.selected && preview::image::is_image(&e.path))
            .map(|(_, e)| e)
            .take(MAX_PREFETCH)
            .map(|e| e.path.clone());
        self.prefetcher.prefetch_many(paths);
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }

    /// Kicks off a background preview load for the current selection. Does
    /// not block: the result is picked up later by `poll_preview`. Scrolling
    /// through several entries before a slow decode (e.g. a large photo)
    /// finishes simply supersedes it, so the UI never backs up behind
    /// work the user has already scrolled past.
    pub fn request_preview(&mut self, picker: Picker) {
        self.preview_scroll = 0;
        match self.selected_entry() {
            Some(entry) => {
                self.preview_generation =
                    self.preview_worker
                        .request(entry.path.clone(), picker, self.last_preview_area);
                self.preview = Preview::Loading;
            }
            None => {
                self.preview_generation = 0;
                self.preview = Preview::Text(vec![]);
            }
        }
        self.prefetch_neighbors();
    }

    /// Warms the thumbnail cache for images near the current selection, so
    /// sequential `j`/`k` browsing through a photo folder doesn't pay full
    /// decode cost on every single step — see `prefetcher`.
    fn prefetch_neighbors(&self) {
        const AHEAD: usize = 2;
        const BEHIND: usize = 1;
        let indices = (self.selected.saturating_sub(BEHIND)..=self.selected.saturating_add(AHEAD))
            .filter(|&i| i != self.selected);
        for i in indices {
            if let Some(entry) = self.entries.get(i) {
                if preview::image::is_image(&entry.path) {
                    self.prefetcher.prefetch(entry.path.clone());
                }
            }
        }
    }

    /// Applies a background preview result if one matching the current
    /// selection has arrived. Cheap to call every frame.
    pub fn poll_preview(&mut self) {
        if let Some(preview) = self.preview_worker.poll(self.preview_generation) {
            self.preview = preview;
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.entries.is_empty() {
            return;
        }
        let len = self.entries.len() as i32;
        let mut idx = self.selected as i32 + delta;
        idx = idx.clamp(0, len - 1);
        self.selected = idx as usize;
    }

    pub fn enter_selected(&mut self) -> Result<()> {
        if let Some(entry) = self.selected_entry() {
            if entry.is_dir() {
                let path = entry.path.clone();
                self.cwd = path;
                self.selected = 0;
                self.reload()?;
            }
        }
        Ok(())
    }

    pub fn go_up(&mut self) -> Result<()> {
        if let Some(parent) = self.cwd.parent() {
            let prev = self.cwd.clone();
            self.cwd = parent.to_path_buf();
            self.reload()?;
            self.selected = self
                .entries
                .iter()
                .position(|e| e.path == prev)
                .unwrap_or(0);
        }
        Ok(())
    }

    pub fn toggle_hidden(&mut self) -> Result<()> {
        self.list_opts.show_hidden = !self.list_opts.show_hidden;
        self.reload()
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    pub fn start_prompt(&mut self, op: PendingOp, default: &str) {
        self.mode = Mode::Prompt {
            op,
            input: default.to_string(),
        };
    }

    pub fn start_confirm(&mut self, op: PendingOp) {
        self.mode = Mode::Confirm { op };
    }

    pub fn cancel_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn copy_to_clipboard(&mut self, cut: bool) {
        if let Some(entry) = self.selected_entry() {
            self.clipboard = Some(Clipboard {
                path: entry.path.clone(),
                cut,
            });
            self.set_message(if cut { "cut" } else { "copied" });
        }
    }

    pub fn run_prompt_submit(&mut self) -> Result<()> {
        let Mode::Prompt { op, input } = std::mem::replace(&mut self.mode, Mode::Normal) else {
            return Ok(());
        };
        let input = input.trim().to_string();
        if input.is_empty() {
            return Ok(());
        }
        match op {
            PendingOp::Rename => {
                if let Some(entry) = self.selected_entry() {
                    fs::ops::rename(&entry.path, &input)?;
                }
            }
            PendingOp::Mkdir => fs::ops::mkdir(&self.cwd, &input)?,
            PendingOp::Touch => fs::ops::touch(&self.cwd, &input)?,
            PendingOp::Delete | PendingOp::Paste { .. } => {}
        }
        self.reload()?;
        Ok(())
    }

    pub fn run_confirm_submit(&mut self) -> Result<()> {
        let Mode::Confirm { op } = std::mem::replace(&mut self.mode, Mode::Normal) else {
            return Ok(());
        };
        match op {
            PendingOp::Delete => {
                if let Some(entry) = self.selected_entry() {
                    fs::ops::delete(&entry.path)?;
                    self.set_message("deleted");
                }
            }
            PendingOp::Paste { cut } => {
                if let Some(clip) = self.clipboard.take() {
                    if cut {
                        fs::ops::move_to(&clip.path, &self.cwd)?;
                    } else {
                        fs::ops::copy(&clip.path, &self.cwd)?;
                    }
                    self.set_message(if cut { "moved" } else { "pasted" });
                }
            }
            PendingOp::Rename | PendingOp::Mkdir | PendingOp::Touch => {}
        }
        self.reload()?;
        Ok(())
    }

    pub fn request_paste(&mut self) {
        if let Some(clip) = &self.clipboard {
            let op = PendingOp::Paste { cut: clip.cut };
            self.start_confirm(op);
        } else {
            self.set_message("clipboard empty");
        }
    }

    pub fn request_delete(&mut self) {
        if self.selected_entry().is_some() {
            self.start_confirm(PendingOp::Delete);
        }
    }

    pub fn request_rename(&mut self) {
        if let Some(entry) = self.selected_entry() {
            let name = entry.name.clone();
            self.start_prompt(PendingOp::Rename, &name);
        }
    }

    pub fn cycle_sort(&mut self) -> Result<()> {
        self.list_opts.sort = match self.list_opts.sort {
            fs::SortMode::Name => fs::SortMode::Size,
            fs::SortMode::Size => fs::SortMode::Modified,
            fs::SortMode::Modified => fs::SortMode::Name,
        };
        let sort_label = match self.list_opts.sort {
            fs::SortMode::Name => "name",
            fs::SortMode::Size => "size",
            fs::SortMode::Modified => "modified",
        };
        self.reload()?;
        self.set_message(format!("sort: {sort_label}"));
        Ok(())
    }
}
