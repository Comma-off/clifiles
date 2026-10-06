use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn list_entries(path: &Path) -> Result<Vec<String>> {
    let name = path.to_string_lossy().to_lowercase();
    if name.ends_with(".zip") {
        list_zip(path)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        list_tar_gz(path)
    } else if name.ends_with(".tar") {
        list_tar(path)
    } else if name.ends_with(".7z") {
        list_7z(path)
    } else {
        Ok(vec!["(unsupported archive format)".into()])
    }
}

fn list_zip(path: &Path) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut out = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        out.push(format_entry(entry.name(), entry.size(), entry.is_dir()));
    }
    Ok(out)
}

fn list_tar(path: &Path) -> Result<Vec<String>> {
    let file = File::open(path)?;
    list_tar_reader(file)
}

fn list_tar_gz(path: &Path) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let gz = flate2::read::GzDecoder::new(file);
    list_tar_reader(gz)
}

fn list_tar_reader<R: Read>(reader: R) -> Result<Vec<String>> {
    let mut archive = tar::Archive::new(reader);
    let mut out = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        let header = entry.header();
        let path = entry.path()?.to_string_lossy().into_owned();
        out.push(format_entry(&path, header.size().unwrap_or(0), header.entry_type().is_dir()));
    }
    Ok(out)
}

fn list_7z(path: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    sevenz_rust2::decompress_file_with_extract_fn(path, "", |entry, _reader, _dest| {
        out.push(format_entry(&entry.name, entry.size, entry.is_directory));
        Ok(true)
    })
    .ok();
    if out.is_empty() {
        out.push("(unable to read 7z contents)".into());
    }
    Ok(out)
}

fn format_entry(name: &str, size: u64, is_dir: bool) -> String {
    if is_dir {
        format!("{name}/")
    } else {
        format!("{name}  ({})", crate::fs::entry::human_size(size))
    }
}
