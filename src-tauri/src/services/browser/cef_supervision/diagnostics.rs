#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::browser) enum CefUnavailableCategory {
    Object,
    Permission,
    Admission,
    Reaper,
    Sandbox,
}

impl CefUnavailableCategory {
    pub(super) const ALL: [Self; 5] = [
        Self::Object,
        Self::Permission,
        Self::Admission,
        Self::Reaper,
        Self::Sandbox,
    ];

    pub(super) const fn code(self) -> &'static str {
        match self {
            Self::Object => "cef-supervision-object",
            Self::Permission => "cef-supervision-permission",
            Self::Admission => "cef-supervision-admission",
            Self::Reaper => "cef-supervision-reaper",
            Self::Sandbox => "cef-supervision-sandbox",
        }
    }
}
