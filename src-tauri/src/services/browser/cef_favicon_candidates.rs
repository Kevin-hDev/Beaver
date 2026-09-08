use super::{cef_text::validated_cef_url, favicon_policy::MAX_CANDIDATES};
use cef::{sys, CefString, CefStringList};

pub(super) fn read(list: &CefStringList) -> Vec<String> {
    let raw: *const sys::_cef_string_list_t = list.into();
    if raw.is_null() {
        return Vec::new();
    }
    // The high-level IntoIterator eagerly collects the entire untrusted list.
    // Borrow only during this callback and free each native temporary via RAII.
    unsafe {
        let raw = raw.cast_mut();
        let count = sys::cef_string_list_size(raw).min(MAX_CANDIDATES);
        let mut result = Vec::with_capacity(count);
        for index in 0..count {
            let mut value = NativeString(std::mem::zeroed());
            if sys::cef_string_list_value(raw, index, &mut value.0) != 1 {
                continue;
            }
            let borrowed = CefString::from(std::ptr::from_ref(&value.0));
            if let Some(url) = validated_cef_url(&borrowed) {
                result.push(url);
            }
        }
        result
    }
}

struct NativeString(sys::cef_string_t);
impl Drop for NativeString {
    fn drop(&mut self) {
        // CEF allocated this UTF-16 temporary, including on validation failure.
        unsafe { sys::cef_string_utf16_clear(&mut self.0) };
    }
}
