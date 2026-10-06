use anyhow::{Context, Result};
use image::DynamicImage;
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::Resize;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Images are downscaled to at most this on their longest side before being
/// handed to the terminal-graphics layer. Decoding a full-resolution photo
/// (a 24MP JPEG, a 10+ MiB PNG) is the real cost of showing a preview — the
/// resize down to the pane is already cheap and already fits the pane, so
/// capping here is what actually saves time, in two ways: it bounds the
/// *one-time* decode-adjacent work (resize, encode), and it means a later
/// terminal resize re-fits this capped image rather than the giant original.
/// It also gives the on-disk cache below a small, fast-to-redecode file.
const MAX_DIMENSION: u32 = 1600;

pub fn load(path: &Path, picker: &Picker, preview_area: Rect) -> Result<StatefulProtocol> {
    let img = load_capped(path)?;
    let mut proto = picker.new_resize_protocol(img);

    // `new_resize_protocol` only wraps the decoded image; the actual
    // resize-to-pane-size + protocol encode (base64 for Kitty/iTerm2, the
    // sixel palette pass, ...) happens lazily on first render otherwise —
    // which runs on the UI thread. Pre-run it here, on this background
    // thread, so the UI thread's render is just a cheap buffer write. The
    // crate's own docs call this out as the intended use of these methods:
    // "This can be done in a background thread, and the result is stored in
    // this StatefulProtocol." A zero-size area (e.g. the very first preview
    // before any frame has been drawn) is skipped — ratatui-image no-ops it
    // the same way — and that one case falls back to the lazy UI-thread path.
    if preview_area.width > 0 && preview_area.height > 0 {
        let resize = Resize::default();
        let background_color = proto.background_color();
        if let Some(rect) = proto.needs_resize(&resize, preview_area) {
            proto.resize_encode(&resize, background_color, rect);
        }
    }

    Ok(proto)
}

/// Decodes, downscales, and caches `path` if it isn't cached already, without
/// building a `StatefulProtocol` (no `Picker` needed). Used by the prefetcher
/// to warm the on-disk cache for files the user hasn't selected yet, so that
/// by the time they do, the expensive part (the one unavoidable full decode)
/// has already happened off the critical path.
pub fn warm_cache(path: &Path) -> Result<()> {
    if cache_path_for(path).is_some_and(|p| p.exists()) {
        return Ok(());
    }
    load_capped(path).map(|_| ())
}

fn load_capped(path: &Path) -> Result<DynamicImage> {
    let cache_file = cache_path_for(path);

    if let Some(cache_file) = &cache_file {
        if let Ok(cached) = image::open(cache_file) {
            return Ok(cached);
        }
    }

    let img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?;
    let capped = cap_dimensions(img);

    if let Some(cache_file) = &cache_file {
        // Best-effort: a failed cache write (e.g. disk full, unwritable cache
        // dir) should never block showing the preview we already decoded.
        let _ = write_cache(cache_file, &capped);
    }

    Ok(capped)
}

fn cap_dimensions(img: DynamicImage) -> DynamicImage {
    if img.width() <= MAX_DIMENSION && img.height() <= MAX_DIMENSION {
        return img;
    }
    img.resize(
        MAX_DIMENSION,
        MAX_DIMENSION,
        image::imageops::FilterType::Triangle,
    )
}

/// Cache key covers path + mtime + size (so an edited file misses the cache
/// instead of showing a stale thumbnail) and the cap itself (so changing
/// `MAX_DIMENSION` naturally invalidates old entries rather than mixing
/// differently-sized cached thumbnails).
fn cache_path_for(path: &Path) -> Option<PathBuf> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    canon.hash(&mut hasher);
    mtime.hash(&mut hasher);
    meta.len().hash(&mut hasher);
    MAX_DIMENSION.hash(&mut hasher);
    let key = hasher.finish();

    let cache_dir = directories::ProjectDirs::from("", "", "clifiles")?
        .cache_dir()
        .join("thumbnails");
    Some(cache_dir.join(format!("{key:016x}.jpg")))
}

fn write_cache(cache_file: &Path, img: &DynamicImage) -> Result<()> {
    if let Some(parent) = cache_file.parent() {
        std::fs::create_dir_all(parent).context("creating thumbnail cache dir")?;
    }
    // JPEG: small and fast to redecode. The cache is a display-only
    // thumbnail, so JPEG's lossy compression and dropped alpha are fine.
    img.to_rgb8()
        .save_with_format(cache_file, image::ImageFormat::Jpeg)
        .context("writing thumbnail cache")?;
    Ok(())
}

pub fn is_image(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()),
        Some(ref ext) if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "tiff" | "tga" | "pnm" | "qoi")
    )
}
