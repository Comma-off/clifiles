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

[colors]
dir = "blue"        # named color or "#rrggbb" hex
symlink = "cyan"
file = "white"
# highlight_fg = "black"
# highlight_bg = "white"
```

## Keybindings

| Key                | Action                          |
|--------------------|----------------------------------|
| `j`/`k`, `↓`/`↑`    | Move selection                  |
| `g` / `G`           | Jump to top / bottom            |
| `l`, `→`, `Enter`   | Enter directory                 |
| `h`, `←`, Backspace | Go to parent directory          |
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
