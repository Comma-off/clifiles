use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const MAX_PREVIEW_BYTES: usize = 256 * 1024;
const MAX_HIGHLIGHT_LINES: usize = 2000;

#[derive(Clone)]
pub struct StyledSpan {
    pub text: String,
    pub color: Option<(u8, u8, u8)>,
    pub bold: bool,
    pub italic: bool,
}

pub type StyledLine = Vec<StyledSpan>;

pub enum TextPreview {
    Lines(Vec<StyledLine>),
    Binary,
}

fn syntax_set() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let mut set = ThemeSet::load_defaults();
        set.themes
            .remove("base16-ocean.dark")
            .unwrap_or_else(|| set.themes.remove("base16-eighties.dark").unwrap())
    })
}

pub fn load(path: &Path) -> std::io::Result<TextPreview> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = vec![0u8; MAX_PREVIEW_BYTES];
    let n = file.read(&mut buf)?;
    buf.truncate(n);

    if is_binary(&buf) {
        return Ok(TextPreview::Binary);
    }

    let text = String::from_utf8_lossy(&buf);
    Ok(TextPreview::Lines(highlight(path, &text)))
}

fn highlight(path: &Path, text: &str) -> Vec<StyledLine> {
    let syntax_set = syntax_set();
    let syntax = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(|ext| syntax_set.find_syntax_by_extension(ext))
        .or_else(|| {
            let first_line = text.lines().next().unwrap_or("");
            syntax_set.find_syntax_by_first_line(first_line)
        })
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme());
    LinesWithEndings::from(text)
        .take(MAX_HIGHLIGHT_LINES)
        .map(|line| {
            let ranges = highlighter
                .highlight_line(line, syntax_set)
                .unwrap_or_else(|_| vec![(syntect::highlighting::Style::default(), line)]);
            ranges
                .into_iter()
                .map(|(style, piece)| StyledSpan {
                    text: piece.trim_end_matches(['\n', '\r']).to_string(),
                    color: Some((
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    )),
                    bold: style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::BOLD),
                    italic: style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::ITALIC),
                })
                .collect()
        })
        .collect()
}

fn is_binary(buf: &[u8]) -> bool {
    let sample = &buf[..buf.len().min(4096)];
    let nul_or_control = sample
        .iter()
        .filter(|&&b| b == 0 || (b < 9 && b != b'\n' && b != b'\r' && b != b'\t'))
        .count();
    !sample.is_empty() && (nul_or_control as f64 / sample.len() as f64) > 0.01
}
