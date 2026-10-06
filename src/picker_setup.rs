use ratatui_image::picker::{Picker, ProtocolType};

/// Builds a `Picker` without touching stdin.
///
/// `Picker::from_query_stdio()` writes an escape-sequence query and spawns a
/// thread that blocks on a raw `read` of stdin waiting for the terminal's
/// reply. When a terminal never answers (notably under tmux/screen, or any
/// non-responding emulator), that thread never exits and is left permanently
/// racing crossterm's own stdin reader for every later keystroke, silently
/// dropping input. We avoid that class of bug entirely by sizing from the
/// ioctl-based `window_size()` (no terminal round-trip) and selecting the
/// graphics protocol from environment variables alone.
pub fn build_picker() -> Picker {
    let font_size = cell_size_from_ioctl().unwrap_or((8, 16));
    // `Picker::from_fontsize` already detects iTerm2-family terminals (and kitty/iTerm2
    // when nested in tmux) purely from env vars, with no stdin involved. The one gap is
    // a plain (non-tmux) kitty terminal, which we patch in below.
    let mut picker = Picker::from_fontsize(font_size);
    if picker.protocol_type() == ProtocolType::Halfblocks && is_plain_kitty() {
        picker.set_protocol_type(ProtocolType::Kitty);
    }
    picker
}

fn cell_size_from_ioctl() -> Option<(u16, u16)> {
    let size = crossterm::terminal::window_size().ok()?;
    if size.width == 0 || size.height == 0 || size.columns == 0 || size.rows == 0 {
        return None;
    }
    Some((size.width / size.columns, size.height / size.rows))
}

fn is_plain_kitty() -> bool {
    std::env::var("KITTY_WINDOW_ID").is_ok_and(|v| !v.is_empty())
        || std::env::var("TERM").is_ok_and(|t| t.contains("kitty"))
}
