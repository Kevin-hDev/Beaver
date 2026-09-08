use super::{favicon_png, favicon_policy::valid_dimensions};
use cef::{Image, ImplBinaryValue, ImplImage};

pub(super) fn png(image: &Image) -> Option<String> {
    if image.is_empty() == 1 {
        return None;
    }
    let (mut scale, mut width, mut height) = (0.0, 0, 0);
    if image.representation_info(1.0, Some(&mut scale), Some(&mut width), Some(&mut height)) != 1
        || !valid_dimensions(width, height)
    {
        return None;
    }
    // Output bounds do not bound CEF's earlier network transfer or decoder.
    let binary = image.as_png(1.0, 1, Some(&mut width), Some(&mut height))?;
    favicon_png::encode(width, height, binary.size(), |bytes| {
        binary.data(Some(bytes), 0)
    })
}
