use anyhow::Result;
use std::path::{Path, PathBuf};

/// Writes the final working directory to the path given via --cwd-file (if any),
/// so the shell wrapper function can `cd` into it after the process exits.
pub fn write_cwd(cwd_file: &Option<PathBuf>, cwd: &Path) -> Result<()> {
    if let Some(file) = cwd_file {
        std::fs::write(file, cwd.to_string_lossy().as_bytes())?;
    }
    Ok(())
}
