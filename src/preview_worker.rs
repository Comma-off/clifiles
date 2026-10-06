use crate::preview::{self, Preview};
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct Job {
    generation: u64,
    path: PathBuf,
    picker: Picker,
    area: Rect,
}

struct Shared {
    job: Mutex<Option<Job>>,
    cv: Condvar,
}

/// Runs preview decoding (image decode, syntax highlighting, archive listing)
/// on a background thread so scrolling through a directory full of large
/// photos never blocks the UI.
///
/// Requests are coalesced into a single-slot mailbox rather than a queue: if
/// the user moves the selection several times before the worker finishes the
/// current job, `request` simply overwrites the pending slot, so the worker
/// always decodes only the most recently selected file and never wastes time
/// on selections the user has already scrolled past.
pub struct PreviewWorker {
    shared: Arc<Shared>,
    result_rx: mpsc::Receiver<(u64, Preview)>,
    next_generation: u64,
}

impl PreviewWorker {
    pub fn new() -> Self {
        let shared = Arc::new(Shared {
            job: Mutex::new(None),
            cv: Condvar::new(),
        });
        let (result_tx, result_rx) = mpsc::channel();

        let worker_shared = Arc::clone(&shared);
        thread::spawn(move || loop {
            let job = {
                let mut guard = worker_shared.job.lock().unwrap();
                loop {
                    if let Some(job) = guard.take() {
                        break job;
                    }
                    guard = worker_shared.cv.wait(guard).unwrap();
                }
            };
            let preview = preview::load(&job.path, &job.picker, job.area);
            if result_tx.send((job.generation, preview)).is_err() {
                return;
            }
        });

        Self {
            shared,
            result_rx,
            next_generation: 0,
        }
    }

    /// Requests a preview for `path`, superseding any not-yet-started job.
    /// Returns the generation id; a result is only applied by the caller if
    /// it still matches the latest requested generation.
    pub fn request(&mut self, path: PathBuf, picker: Picker, area: Rect) -> u64 {
        self.next_generation += 1;
        let generation = self.next_generation;
        let mut guard = self.shared.job.lock().unwrap();
        *guard = Some(Job {
            generation,
            path,
            picker,
            area,
        });
        self.shared.cv.notify_one();
        generation
    }

    /// Drains any results ready so far; returns the last one that matches
    /// `current_generation`, if any.
    pub fn poll(&self, current_generation: u64) -> Option<Preview> {
        let mut latest = None;
        while let Ok((generation, preview)) = self.result_rx.try_recv() {
            if generation == current_generation {
                latest = Some(preview);
            }
        }
        latest
    }
}
