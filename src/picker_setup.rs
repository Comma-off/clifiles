use crate::config::{ImageConfig, ImageProtocol};
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
///
/// The cost of skipping the terminal round-trip is that both protocol
/// detection and font-cell-size detection are best-effort guesses: protocol
/// detection can't see through an SSH hop (the env vars it keys off are
/// local-terminal vars that generally aren't forwarded to a remote shell),
/// and without a real cell size in pixels we fall back to a generic guess
/// that's frequently smaller than the terminal's actual font, capping image
/// detail well below what the terminal could show. `ImageConfig` lets a user
/// override either when auto-detection gets it wrong.
pub fn build_picker(config: &ImageConfig) -> Picker {
    let font_size = config
        .cell_width
        .zip(config.cell_height)
        .or_else(cell_size_from_ioctl)
        .unwrap_or((10, 20));

    let mut picker = Picker::from_fontsize(font_size);

    // `Picker::from_fontsize` already detects iTerm2-family terminals (and kitty/iTerm2
    // when nested in tmux) purely from env vars, with no stdin involved. The gaps are
    // terminals that speak the kitty graphics protocol but that ratatui-image's own
    // env-var list doesn't know about, patched in below.
    if picker.protocol_type() == ProtocolType::Halfblocks && is_kitty_protocol_terminal() {
        picker.set_protocol_type(ProtocolType::Kitty);
    }

    if let Some(protocol) = config.protocol {
        picker.set_protocol_type(match protocol {
            ImageProtocol::Kitty => ProtocolType::Kitty,
            ImageProtocol::Iterm2 => ProtocolType::Iterm2,
            ImageProtocol::Sixel => ProtocolType::Sixel,
            ImageProtocol::Halfblocks => ProtocolType::Halfblocks,
        });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ImageConfig;

    #[test]
    fn ghostty_is_detected_as_kitty_protocol() {
        // SAFETY: test runs with --test-threads=1, no concurrent env access.
        unsafe {
            std::env::remove_var("KITTY_WINDOW_ID");
            std::env::remove_var("TERM_PROGRAM");
            std::env::set_var("TERM", "xterm-ghostty");
            std::env::set_var("GHOSTTY_RESOURCES_DIR", "/usr/share/ghostty");
        }
        let picker = build_picker(&ImageConfig::default());
        assert_eq!(picker.protocol_type(), ProtocolType::Kitty);
        unsafe {
            std::env::remove_var("GHOSTTY_RESOURCES_DIR");
            std::env::remove_var("TERM");
        }
    }
}

fn is_kitty_protocol_terminal() -> bool {
    // Plain (non-tmux) kitty.
    std::env::var("KITTY_WINDOW_ID").is_ok_and(|v| !v.is_empty())
        || std::env::var("TERM").is_ok_and(|t| t.contains("kitty"))
        // Ghostty implements the kitty graphics protocol (not iTerm2's), but isn't
        // in ratatui-image's built-in iTerm2-family env-var list.
        || std::env::var("TERM_PROGRAM").is_ok_and(|t| t.eq_ignore_ascii_case("ghostty"))
        || std::env::var("GHOSTTY_RESOURCES_DIR").is_ok()
}
