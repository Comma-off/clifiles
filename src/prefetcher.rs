use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// Warms the on-disk image thumbnail cache (see `preview::image`) for files
/// the user hasn't selected yet but is likely to visit next: every image
/// visible when a directory is opened, plus the current selection's
/// immediate neighbors as the user scrolls further.
///
/// Decoding a large photo is real, unavoidable work (no partial-decode
/// shortcut exists for PNG/JPEG in this stack — confirmed by reading how
/// yazi, a Rust file manager known for being fast at this, does it: same
/// `image` crate, same decode-then-downscale approach, no special fast
/// path). What yazi does differently is run that unavoidable work across a
/// real thread pool instead of one thread at a time, so a whole directory's
/// worth of thumbnails finishes in roughly (work / core count) instead of
/// (work × file count). This does the same.
///
/// Runs on its own pool, entirely separate from `PreviewWorker`'s
/// single-slot "current selection" mailbox, so prefetch work can never
/// delay the preview the user is actually looking at.
pub struct Prefetcher {
    tx: mpsc::Sender<PathBuf>,
}

impl Prefetcher {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<PathBuf>();
        let rx = Arc::new(Mutex::new(rx));
        // Per-run de-dup only (not persisted, not mtime-aware): skipping a
        // path here never affects correctness, since the real selection
        // always re-checks the cache key itself.
        let attempted = Arc::new(Mutex::new(HashSet::new()));

        let worker_count = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(4);
        for _ in 0..worker_count {
            let rx = Arc::clone(&rx);
            let attempted = Arc::clone(&attempted);
            thread::spawn(move || loop {
                let path = {
                    let guard = rx.lock().unwrap();
                    guard.recv()
                };
                let Ok(path) = path else { return };
                if !attempted.lock().unwrap().insert(path.clone()) {
                    continue;
                }
                let _ = crate::preview::image::warm_cache(&path);
            });
        }

        Self { tx }
    }

    pub fn prefetch(&self, path: PathBuf) {
        let _ = self.tx.send(path);
    }

    pub fn prefetch_many(&self, paths: impl IntoIterator<Item = PathBuf>) {
        for path in paths {
            self.prefetch(path);
        }
    }
}
