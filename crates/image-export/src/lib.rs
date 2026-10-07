use image::{DynamicImage, ImageFormat, RgbaImage};
use std::{io::Cursor, path::{Path, PathBuf}};

/// A filename without an extension uses PNG. Other extensions must name a real encoder.
pub fn normalize_target(path: &Path) -> Result<(PathBuf, ImageFormat), String> {
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
    let format = match extension.as_str() {
        "" | "png" => ImageFormat::Png,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "gif" => ImageFormat::Gif,
        "bmp" => ImageFormat::Bmp,
        _ => return Err(format!("unsupported image format for {}", path.display())),
    };
    let target = if extension.is_empty() { path.with_extension("png") } else { path.to_path_buf() };
    Ok((target, format))
}

pub fn encode_rgba(width: u32, height: u32, rgba: Vec<u8>, format: ImageFormat) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 { return Err("image surface has no rendered pixels".into()); }
    let image = RgbaImage::from_raw(width, height, rgba).ok_or("rendered image dimensions do not match its pixel bytes")?;
    let image = if format == ImageFormat::Jpeg {
        DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(image).into_rgb8())
    } else { DynamicImage::ImageRgba8(image) };
    let mut encoded = Cursor::new(Vec::new());
    image.write_to(&mut encoded, format).map_err(|error| format!("failed to encode {format:?} image: {error}"))?;
    Ok(encoded.into_inner())
}

/// The dialog owner supplies the normalized and overwrite-confirmed final target.
pub fn write_rgba_atomic(target: &Path, width: u32, height: u32, rgba: Vec<u8>, format: ImageFormat) -> Result<(), String> {
    let bytes = encode_rgba(width, height, rgba, format)?;
    sgf::write_file_atomic(target, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_source_formats_are_real_decodable_rectangular_images() {
        for suffix in ["png", "jpg", "jpeg", "gif", "bmp", "PNG"] {
            let (_, format) = normalize_target(Path::new(&format!("board.{suffix}"))).unwrap();
            let rgba = [225, 193, 122, 255, 0, 0, 0, 255, 255, 255, 255, 255].repeat(2);
            let bytes = encode_rgba(3, 2, rgba, format).unwrap();
            assert_eq!(image::guess_format(&bytes).unwrap(), format);
            let decoded = image::load_from_memory(&bytes).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (3, 2));
        }
        assert_eq!(normalize_target(Path::new("board")).unwrap().0, Path::new("board.png"));
        assert!(normalize_target(Path::new("board.webp")).is_err());
        assert!(encode_rgba(3, 2, vec![0; 4], ImageFormat::Png).is_err());
    }

    #[test]
    fn atomic_image_write_preserves_target_on_encode_and_replace_failure() {
        let directory = std::env::temp_dir().join(format!("image-export-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&directory).unwrap();
        let target = directory.join("protected.png");
        std::fs::write(&target, b"prior target bytes").unwrap();
        assert!(write_rgba_atomic(&target, 3, 2, vec![0; 4], ImageFormat::Png).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"prior target bytes");
        let occupied = directory.join("occupied.png");
        std::fs::create_dir(&occupied).unwrap();
        std::fs::write(occupied.join("prior"), b"protected").unwrap();
        assert!(write_rgba_atomic(&occupied, 1, 1, vec![0, 0, 0, 255], ImageFormat::Png).is_err());
        assert_eq!(std::fs::read(occupied.join("prior")).unwrap(), b"protected");
        assert!(write_rgba_atomic(&directory.join("missing/board.png"), 1, 1, vec![0, 0, 0, 255], ImageFormat::Png).is_err());
        write_rgba_atomic(&target, 1, 1, vec![0, 0, 0, 255], ImageFormat::Png).unwrap();
        assert_eq!(image::load_from_memory(&std::fs::read(&target).unwrap()).unwrap().width(), 1);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 2);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
