# clifiles

A terminal file manager with mouse support and file previews (text, images,
archives), built in Rust with [ratatui](https://ratatui.rs).

## Build

```
cargo build --release
```

The binary is at `target/release/clifiles`.

## Shell integration (cd on exit)

A child process can't change its parent shell's working directory, so
`clifiles` writes the directory it was in when you quit to a file (passed via
`--cwd-file`), and a small shell function does the `cd`.

**bash**

```bash
echo 'source /path/to/clifiles/shell/cf.bash' >> ~/.bashrc
```

**zsh**

```zsh
echo 'source /path/to/clifiles/shell/cf.zsh' >> ~/.zshrc
```

**fish**

```fish
echo 'source /path/to/clifiles/shell/cf.fish' >> ~/.config/fish/config.fish
```

**PowerShell**

```powershell
Add-Content $PROFILE '. /path/to/clifiles/shell/cf.ps1'
```

Then run `cf` (instead of `clifiles` directly) to browse and land in the
directory you ended up in.

## Configuration

`clifiles` reads an optional TOML config from the OS config directory
(`~/.config/clifiles/config.toml` on Linux, `~/Library/Application
Support/clifiles/config.toml` on macOS, `%APPDATA%\clifiles\config.toml` on
Windows). A missing or invalid file just falls back to defaults — it never
blocks startup.

```toml
show_hidden = false
sort = "name"  # name | size | modified
start_dir = "home"  # home | current (home is the default; "current" starts
                    # in the shell's working directory instead)

[colors]
dir = "blue"        # named color or "#rrggbb" hex
symlink = "cyan"
file = "white"
# highlight_fg = "black"
# highlight_bg = "white"

[image]
# protocol = "kitty"   # kitty | iterm2 | sixel | halfblocks
# cell_width = 10      # your terminal's font cell width in pixels
# cell_height = 20     # your terminal's font cell height in pixels
```

### Fixing low-resolution / blurry image previews

Image previews are rendered through Kitty/iTerm2/Sixel graphics protocols
when detected, falling back to Unicode half-blocks (inherently coarse —
there's no way to improve resolution there). Detection is intentionally
done without querying the terminal (an earlier version did, and a
non-responding terminal left a thread stuck reading stdin forever, silently
eating keystrokes afterward — see `src/picker_setup.rs`). The tradeoff is
that detection is a best-effort guess from environment variables and can be
wrong in two ways:

- **Over SSH**, the env vars it keys off (`TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  ...) belong to your local terminal and usually aren't forwarded to the
  remote shell, so detection has nothing to go on and falls back to
  half-blocks even if your terminal actually supports real graphics.
- **The font cell size in pixels** (needed to know how much detail an
  image can show) often can't be determined at all, and falls back to a
  generic guess that's frequently smaller than your terminal's real font.

If previews look blocky/blurry or your terminal clearly supports better,
set `[image] protocol` and `cell_width`/`cell_height` explicitly in the
config above. If you don't know your terminal's cell size in pixels, err on
the larger side — an overestimate just means the terminal receives more
detail than it needs, not a worse image.

### Image loading speed

Decoding a full-resolution photo is the expensive part of showing a preview
— not resizing it down to fit the pane, which is already cheap. Image
previews are decoded on a background thread (so the UI never blocks), and
downscaled, cached copies are kept in the OS cache directory
(`~/.cache/clifiles/thumbnails` on Linux, `~/Library/Caches/clifiles` on
macOS, `%LOCALAPPDATA%\clifiles\cache` on Windows) so revisiting a file —
including just scrolling back past it — is near-instant afterward. A file's
cache entry is keyed by its path, size, and modified time, so editing a
file invalidates its cached thumbnail automatically. The cache only grows;
it's safe to delete the directory any time to reclaim space.

## Keybindings

| Key                | Action                          |
|--------------------|----------------------------------|
| `j`/`k`, `↓`/`↑`    | Move selection                  |
| `g` / `G`           | Jump to top / bottom            |
| `Enter`, `→`        | Enter directory                 |
| `o`, `←`, Backspace | Go to parent directory ("out")  |
| `.`                 | Toggle hidden files             |
| `s`                 | Cycle sort (name/size/modified) |
| `d`                 | Delete selected (confirm)       |
| `r`                 | Rename selected                 |
| `m`                 | Create directory                |
| `t`                 | Create empty file                |
| `y` / `x`           | Copy / cut selected              |
| `p`                 | Paste (confirm)                  |
| `J` / `K`           | Scroll preview down / up         |
| `q`, `Esc`          | Quit                              |

Mouse: click to select, double-click to enter a directory, scroll wheel to
move the selection.

## Previews

- **Text** — plain text, up to 256 KiB.
- **Images** — rendered via Kitty/iTerm2 graphics protocols when the
  terminal supports them, falling back to Unicode half-blocks otherwise.
- **Archives** — `.zip`, `.tar`, `.tar.gz`/`.tgz`, `.7z` contents are listed
  without extracting.
- **Directories** — listed in the preview pane.
