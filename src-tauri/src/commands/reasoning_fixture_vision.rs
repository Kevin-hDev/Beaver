use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};

pub(crate) const WIDTH: u32 = 64;
pub(crate) const HEIGHT: u32 = 64;
pub(crate) const MIME: &str = "image/png";
const MAX_BYTES: usize = 4 * 1024;

pub(crate) fn png_bytes() -> Result<Vec<u8>, String> {
    let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let rgba = match (x < WIDTH / 2, y < HEIGHT / 2) {
                (true, true) => [255, 0, 0, 255],
                (false, true) => [0, 255, 0, 255],
                (true, false) => [0, 0, 255, 255],
                (false, false) => [255, 255, 0, 255],
            };
            pixels.extend_from_slice(&rgba);
        }
    }
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(&pixels, WIDTH, HEIGHT, ExtendedColorType::Rgba8)
        .map_err(|_| "vision_fixture_invalid".to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("vision_fixture_invalid".into());
    }
    Ok(bytes)
}

pub(crate) fn inline_base64() -> Result<String, String> {
    png_bytes().map(|bytes| STANDARD.encode(bytes))
}

pub(crate) fn inline_attachment(
) -> Result<crate::models::agent_turn_contract::TurnAttachmentInput, String> {
    let bytes = png_bytes()?;
    let encoded = STANDARD.encode(&bytes);
    Ok(crate::models::agent_turn_contract::TurnAttachmentInput {
        name: "reasoning-fixture-quadrants.png".into(),
        path: String::new(),
        mime_type: MIME.into(),
        size: bytes.len() as u64,
        thumbnail: Some(format!("data:{MIME};base64,{encoded}")),
        access_grant: None,
    })
}
