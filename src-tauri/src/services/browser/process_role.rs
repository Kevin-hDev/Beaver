#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CefProcessRole {
    Helper = 1,
}

impl TryFrom<u8> for CefProcessRole {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Helper),
            _ => Err(()),
        }
    }
}

impl From<CefProcessRole> for u8 {
    fn from(value: CefProcessRole) -> Self {
        value as Self
    }
}

#[cfg(any(test, target_os = "macos"))]
pub(super) fn validate_browser_process_result(result: i32) -> Result<(), ()> {
    (result == -1).then_some(()).ok_or(())
}
