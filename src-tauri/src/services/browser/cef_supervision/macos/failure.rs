use crate::services::browser::cef_unavailable::CefUnavailableCategory;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MacSupervisionFailure {
    PendingLayout,
    PendingMissing,
    ActiveSlotOccupied,
    MailboxSnapshot,
    GenerationMismatch,
    Publication,
    AuthorityClaim,
    Identity,
    AuthorityAdmission,
    EmergencyInstall,
    Liveness,
    ClosingClock,
    PendingCloseSignal,
    AdmittedCloseSignal,
    EmergencyUnavailable,
    TrackerPanic,
    EmergencyReaperPanic,
    ForcePass,
    TrackerJoinPanic,
    External(CefUnavailableCategory),
}

impl MacSupervisionFailure {
    pub(super) const fn category(self) -> CefUnavailableCategory {
        match self {
            Self::PendingLayout
            | Self::PendingMissing
            | Self::ActiveSlotOccupied
            | Self::MailboxSnapshot
            | Self::GenerationMismatch
            | Self::Publication
            | Self::AuthorityClaim
            | Self::Identity
            | Self::AuthorityAdmission
            | Self::EmergencyInstall => CefUnavailableCategory::Admission,
            Self::Liveness
            | Self::ClosingClock
            | Self::PendingCloseSignal
            | Self::AdmittedCloseSignal
            | Self::EmergencyUnavailable
            | Self::TrackerPanic
            | Self::EmergencyReaperPanic
            | Self::ForcePass
            | Self::TrackerJoinPanic => CefUnavailableCategory::Reaper,
            Self::External(category) => category,
        }
    }

    pub(super) const fn code(self) -> &'static str {
        match self {
            Self::PendingLayout => "admission-pending-layout",
            Self::PendingMissing => "admission-pending-missing",
            Self::ActiveSlotOccupied => "admission-active-slot-occupied",
            Self::MailboxSnapshot => "admission-mailbox-snapshot",
            Self::GenerationMismatch => "admission-generation-mismatch",
            Self::Publication => "admission-publication",
            Self::AuthorityClaim => "admission-authority-claim",
            Self::Identity => "admission-identity",
            Self::AuthorityAdmission => "admission-authority-commit",
            Self::EmergencyInstall => "admission-emergency-install",
            Self::Liveness => "reaper-liveness",
            Self::ClosingClock => "reaper-closing-clock",
            Self::PendingCloseSignal => "reaper-pending-close-signal",
            Self::AdmittedCloseSignal => "reaper-admitted-close-signal",
            Self::EmergencyUnavailable => "reaper-unavailable",
            Self::TrackerPanic => "reaper-tracker-panic",
            Self::EmergencyReaperPanic => "reaper-worker-panic",
            Self::ForcePass => "reaper-force-pass",
            Self::TrackerJoinPanic => "reaper-tracker-join-panic",
            Self::External(CefUnavailableCategory::Object) => "external-object",
            Self::External(CefUnavailableCategory::Permission) => "external-permission",
            Self::External(CefUnavailableCategory::Admission) => "external-admission",
            Self::External(CefUnavailableCategory::Reaper) => "external-reaper",
            Self::External(CefUnavailableCategory::Sandbox) => "external-sandbox",
        }
    }
}
