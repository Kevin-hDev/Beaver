#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileSignature {
    Jpeg,
    Png,
    Gif,
    Webp,
    Pdf,
    Zip,
    Utf8,
    Binary,
}

impl FileSignature {
    pub(crate) fn mime(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
            Self::Pdf => "application/pdf",
            Self::Zip => "application/zip",
            Self::Utf8 => "text/plain",
            Self::Binary => "application/octet-stream",
        }
    }

    pub(crate) fn image(self) -> bool {
        matches!(self, Self::Jpeg | Self::Png | Self::Gif | Self::Webp)
    }

    pub(crate) fn matches_declared_image(self, declared: &str) -> bool {
        declared.eq_ignore_ascii_case(self.mime())
            || declared.eq_ignore_ascii_case(self.extension())
            || (self == Self::Jpeg && declared.eq_ignore_ascii_case("jpg"))
    }

    pub(crate) fn matches_image_extension(self, extension: &str) -> bool {
        self.image()
            && (extension.eq_ignore_ascii_case(self.extension())
                || (self == Self::Jpeg && extension.eq_ignore_ascii_case("jpg")))
    }

    fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::Webp => "webp",
            Self::Pdf => "pdf",
            Self::Zip => "zip",
            Self::Utf8 => "txt",
            Self::Binary => "bin",
        }
    }
}

pub(crate) fn classify(bytes: &[u8]) -> FileSignature {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        FileSignature::Jpeg
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        FileSignature::Png
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        FileSignature::Gif
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        FileSignature::Webp
    } else if bytes.starts_with(b"%PDF-") {
        FileSignature::Pdf
    } else if bytes.starts_with(b"PK\x03\x04") {
        FileSignature::Zip
    } else if std::str::from_utf8(bytes).is_ok() && !bytes.contains(&0) {
        FileSignature::Utf8
    } else {
        FileSignature::Binary
    }
}

pub(crate) fn classify_base64(payload: &str) -> FileSignature {
    let prefix = payload.as_bytes().get(..16).unwrap_or(payload.as_bytes());
    if prefix.starts_with(b"/9j/") {
        FileSignature::Jpeg
    } else if prefix.starts_with(b"iVBOR") {
        FileSignature::Png
    } else if prefix.starts_with(b"R0lGO") {
        FileSignature::Gif
    } else if prefix.starts_with(b"UklGR") {
        FileSignature::Webp
    } else {
        FileSignature::Binary
    }
}
