use std::path::Path;

use larptrix_protocol::MAX_ATTACHMENT_BYTES;

pub struct UploadKind {
    pub mime: &'static str,
    pub ext: &'static str,
}

pub fn detect_image(bytes: &[u8]) -> Option<UploadKind> {
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Some(UploadKind {
            mime: "image/jpeg",
            ext: "jpg",
        });
    }
    if bytes.len() >= 8 && bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(UploadKind {
            mime: "image/png",
            ext: "png",
        });
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(UploadKind {
            mime: "image/gif",
            ext: "gif",
        });
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some(UploadKind {
            mime: "image/webp",
            ext: "webp",
        });
    }
    None
}

pub fn detect_audio(bytes: &[u8]) -> Option<UploadKind> {
    if bytes.starts_with(b"OggS") {
        return Some(UploadKind {
            mime: "audio/ogg",
            ext: "ogg",
        });
    }
    if bytes.starts_with(b"ID3")
        || (bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] & 0xE0 == 0xE0)
    {
        return Some(UploadKind {
            mime: "audio/mpeg",
            ext: "mp3",
        });
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WAVE" {
        return Some(UploadKind {
            mime: "audio/wav",
            ext: "wav",
        });
    }
    if bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        return Some(UploadKind {
            mime: "audio/webm",
            ext: "webm",
        });
    }
    if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        return Some(UploadKind {
            mime: "audio/mp4",
            ext: "m4a",
        });
    }
    None
}

pub fn safe_filename(raw: Option<&str>) -> String {
    let name = raw
        .and_then(|name| Path::new(name).file_name())
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "attachment".into());
    let cleaned: String = name
        .chars()
        .filter(|ch| !ch.is_control())
        .map(|ch| {
            if matches!(ch, '/' | '\\' | '"') {
                '_'
            } else {
                ch
            }
        })
        .take(120)
        .collect();
    if cleaned.trim().is_empty() {
        "attachment".into()
    } else {
        cleaned
    }
}

pub fn safe_extension(filename: &str) -> String {
    filename
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| {
            !ext.is_empty() && ext.len() <= 10 && ext.chars().all(|c| c.is_ascii_alphanumeric())
        })
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "bin".into())
}

pub fn write_upload(dir: &Path, id: &str, ext: &str, bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() > MAX_ATTACHMENT_BYTES {
        return Err("file is too large");
    }
    std::fs::create_dir_all(dir).map_err(|_| "could not create upload dir")?;
    let path = dir.join(format!("{id}.{ext}"));
    std::fs::write(path, bytes).map_err(|_| "could not save image")?;
    Ok(())
}

pub fn upload_path(dir: &Path, id: &str, ext: &str) -> std::path::PathBuf {
    dir.join(format!("{id}.{ext}"))
}

#[cfg(test)]
mod tests {
    use super::{detect_audio, safe_extension, safe_filename};

    #[test]
    fn detects_browser_recorded_webm_audio() {
        let kind = detect_audio(&[0x1A, 0x45, 0xDF, 0xA3]).unwrap();
        assert_eq!(kind.mime, "audio/webm");
        assert_eq!(kind.ext, "webm");
    }

    #[test]
    fn sanitizes_uploaded_names_and_extensions() {
        assert_eq!(safe_filename(Some("../../report\n.txt")), "report.txt");
        assert_eq!(
            safe_filename(Some("unsafe\\name\".PDF")),
            "unsafe_name_.PDF"
        );
        assert_eq!(safe_extension("report.PDF"), "pdf");
        assert_eq!(safe_extension("no extension"), "bin");
    }
}
