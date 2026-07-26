use super::DetectedEditor;
use core_foundation::array::{CFArrayGetCount, CFArrayGetValueAtIndex};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation::url::{kCFURLPOSIXPathStyle, CFURL};
use std::path::Path;

type LSRolesMask = u32;
const K_LS_ROLES_ALL: LSRolesMask = 0xFFFFFFFF;

extern "C" {
    fn LSCopyApplicationURLsForURL(
        url: core_foundation::url::CFURLRef,
        roles: LSRolesMask,
    ) -> core_foundation::array::CFArrayRef;

    fn LSCopyDefaultApplicationURLForURL(
        url: core_foundation::url::CFURLRef,
        roles: LSRolesMask,
        out_error: *mut core_foundation::error::CFErrorRef,
    ) -> core_foundation::url::CFURLRef;
}

fn app_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

fn cfurl_to_path(url_ref: core_foundation::url::CFURLRef) -> Option<String> {
    if url_ref.is_null() {
        return None;
    }
    // SAFETY: `url_ref` is checked non-null above and is borrowed from LaunchServices.
    // `wrap_under_get_rule` does not take ownership, so no release is performed here.
    let url: CFURL = unsafe { TCFType::wrap_under_get_rule(url_ref) };
    let cf_path = url.get_file_system_path(kCFURLPOSIXPathStyle);
    Some(cf_path.to_string())
}

pub fn detect(file_path: &Path) -> Vec<DetectedEditor> {
    let path_str = file_path.to_str().unwrap_or("");
    let cf_str = CFString::new(path_str);
    let file_url = CFURL::from_file_system_path(cf_str, kCFURLPOSIXPathStyle, false);

    // SAFETY: `file_url` is a valid CFURL created by core-foundation. LaunchServices
    // returns either null or a retained CFURL, which is wrapped with the create rule.
    let default_path = unsafe {
        let raw = LSCopyDefaultApplicationURLForURL(
            file_url.as_concrete_TypeRef(),
            K_LS_ROLES_ALL,
            std::ptr::null_mut(),
        );
        if raw.is_null() {
            None
        } else {
            let url = CFURL::wrap_under_create_rule(raw);
            let p = url.get_file_system_path(kCFURLPOSIXPathStyle);
            Some(p.to_string())
        }
    };

    // SAFETY: `file_url` is a valid CFURL. LaunchServices returns a retained array
    // or null; non-null arrays are released exactly once before returning.
    let raw_array =
        unsafe { LSCopyApplicationURLsForURL(file_url.as_concrete_TypeRef(), K_LS_ROLES_ALL) };
    if raw_array.is_null() {
        return vec![];
    }

    // SAFETY: `raw_array` was checked non-null and remains valid until CFRelease below.
    let count = unsafe { CFArrayGetCount(raw_array) };
    let mut editors = Vec::new();

    for i in 0..count {
        // SAFETY: `i` is in `0..count`, and CoreFoundation returns a borrowed item
        // from the still-live array. `cfurl_to_path` uses get-rule wrapping.
        let raw_url =
            unsafe { CFArrayGetValueAtIndex(raw_array, i) as core_foundation::url::CFURLRef };
        if let Some(path) = cfurl_to_path(raw_url) {
            let is_default = default_path.as_ref() == Some(&path);
            editors.push(DetectedEditor {
                name: app_name_from_path(&path),
                path,
                is_default,
            });
        }
    }

    // SAFETY: `raw_array` came from a Copy/Create API and has not been released yet.
    unsafe { core_foundation::base::CFRelease(raw_array as *const _) };

    editors.sort_by_key(|editor| std::cmp::Reverse(editor.is_default));
    editors
}
