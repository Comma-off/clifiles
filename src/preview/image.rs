use anyhow::Result;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::path::Path;

pub fn load(path: &Path, picker: &mut Picker) -> Result<StatefulProtocol> {
    let dyn_img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?;
    Ok(picker.new_resize_protocol(dyn_img))
}

pub fn is_image(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()),
        Some(ref ext) if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" | "tiff" | "tga" | "pnm" | "qoi")
    )
}
