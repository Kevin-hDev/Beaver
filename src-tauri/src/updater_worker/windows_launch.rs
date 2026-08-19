use std::process::Command;

use windows_sys::Win32::System::Threading::CREATE_BREAKAWAY_FROM_JOB;

/// Le helper de mise a jour relance Beaver puis disparait aussitot. Sans cet
/// affranchissement, l'application relancee reste dans le Job `kill on close`
/// herite du helper et meurt a sa sortie ; le lancement echoue plutot que de
/// retomber sur un lancement confine, qui rendrait une reussite trompeuse.
pub(super) fn configure_relaunched_application(command: &mut Command) {
    crate::services::background_command::configure_with_extra_flags(
        command,
        CREATE_BREAKAWAY_FROM_JOB,
    );
}
