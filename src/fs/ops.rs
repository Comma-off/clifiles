use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn delete(path: &Path) -> Result<()> {
    if path.is_dir() && !path.is_symlink() {
        fs::remove_dir_all(path).with_context(|| format!("deleting directory {}", path.display()))
    } else {
        fs::remove_file(path).with_context(|| format!("deleting file {}", path.display()))
    }
}

pub fn rename(path: &Path, new_name: &str) -> Result<()> {
    let dest = path
        .parent()
        .map(|p| p.join(new_name))
        .context("path has no parent")?;
    fs::rename(path, &dest).with_context(|| format!("renaming {} to {}", path.display(), dest.display()))
}

pub fn mkdir(parent: &Path, name: &str) -> Result<()> {
    fs::create_dir(parent.join(name)).context("creating directory")
}

pub fn touch(parent: &Path, name: &str) -> Result<()> {
    fs::File::create(parent.join(name)).context("creating file")?;
    Ok(())
}

pub fn copy(src: &Path, dest_dir: &Path) -> Result<()> {
    let name = src.file_name().context("source has no filename")?;
    let dest = dest_dir.join(name);
    if src.is_dir() {
        copy_dir_recursive(src, &dest)
    } else {
        fs::copy(src, &dest).with_context(|| format!("copying {} to {}", src.display(), dest.display()))?;
        Ok(())
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dest.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

pub fn move_to(src: &Path, dest_dir: &Path) -> Result<()> {
    let name = src.file_name().context("source has no filename")?;
    let dest = dest_dir.join(name);
    match fs::rename(src, &dest) {
        Ok(()) => Ok(()),
        Err(_) => {
            // cross-device move: copy then delete
            copy(src, dest_dir)?;
            delete(src)
        }
    }
}
