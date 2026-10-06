use std::io::Read;
use std::path::Path;

const MAX_PREVIEW_BYTES: usize = 256 * 1024;

pub enum TextPreview {
    Lines(Vec<String>),
    Binary,
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
    let lines = text.lines().map(str::to_owned).collect();
    Ok(TextPreview::Lines(lines))
}

fn is_binary(buf: &[u8]) -> bool {
    let sample = &buf[..buf.len().min(4096)];
    let nul_or_control = sample
        .iter()
        .filter(|&&b| b == 0 || (b < 9 && b != b'\n' && b != b'\r' && b != b'\t'))
        .count();
    !sample.is_empty() && (nul_or_control as f64 / sample.len() as f64) > 0.01
}
