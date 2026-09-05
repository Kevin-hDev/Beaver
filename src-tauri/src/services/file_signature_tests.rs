use super::file_signature::{classify, FileSignature};

#[test]
fn classifies_every_supported_signature_and_keeps_zip_ambiguous() {
    for (bytes, expected) in [
        (b"\xff\xd8\xff".as_slice(), FileSignature::Jpeg),
        (b"\x89PNG\r\n\x1a\n".as_slice(), FileSignature::Png),
        (b"GIF89a".as_slice(), FileSignature::Gif),
        (b"RIFF....WEBP".as_slice(), FileSignature::Webp),
        (b"%PDF-1.7".as_slice(), FileSignature::Pdf),
        (b"PK\x03\x04".as_slice(), FileSignature::Zip),
        ("texte UTF-8 🦫".as_bytes(), FileSignature::Utf8),
        (b"\x00\x01\x02".as_slice(), FileSignature::Binary),
    ] {
        assert_eq!(classify(bytes), expected);
    }
}

#[test]
fn attachment_and_vision_delegate_their_signature_tables() {
    let attachment = include_str!("agent_local/conversation_attachment_format.rs");
    let vision = include_str!("llm/vision.rs");

    for marker in ["\\x89PNG", "GIF87a", "RIFF", "iVBOR", "R0lGO", "UklGR"] {
        assert!(
            !attachment.contains(marker),
            "attachment duplicates {marker}"
        );
        assert!(!vision.contains(marker), "vision duplicates {marker}");
    }
}
