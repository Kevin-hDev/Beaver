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

    pub(super) const fn id(self) -> u8 {
        match self {
            Self::Object => 1,
            Self::Permission => 2,
            Self::Admission => 3,
            Self::Reaper => 4,
            Self::Sandbox => 5,
        }
    }

    pub(super) const fn from_id(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Object),
            2 => Some(Self::Permission),
            3 => Some(Self::Admission),
            4 => Some(Self::Reaper),
            5 => Some(Self::Sandbox),
            _ => None,
        }
    }
}
