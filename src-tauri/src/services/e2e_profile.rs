#[derive(Clone, Copy)]
pub enum LifecycleStage {
    SetupEntered,
    SetupCompleted,
}

#[derive(Clone, Copy)]
pub enum BrowserExitSource {
    Initialization,
    LaunchCallback,
    ChildAdmission,
    Supervision,
}

pub fn report_lifecycle(stage: LifecycleStage) {
    #[cfg(feature = "e2e")]
    eprintln!(
        "[e2e-lifecycle] {}",
        match stage {
            LifecycleStage::SetupEntered => "setup-entered",
            LifecycleStage::SetupCompleted => "setup-completed",
        }
    );
    #[cfg(not(feature = "e2e"))]
    let _ = stage;
}

pub fn report_browser_exit_source(source: BrowserExitSource) {
    #[cfg(feature = "e2e")]
    eprintln!("[e2e-exit-source] {}", exit_source_name(source));
    #[cfg(not(feature = "e2e"))]
    let _ = source;
}

#[cfg(feature = "e2e")]
const fn exit_source_name(source: BrowserExitSource) -> &'static str {
    match source {
        BrowserExitSource::Initialization => "browser-initialization",
        BrowserExitSource::LaunchCallback => "browser-launch-callback",
        BrowserExitSource::ChildAdmission => "browser-child-admission",
        BrowserExitSource::Supervision => "browser-supervision",
    }
}

pub fn load_dotenv<Action>(action: Action)
where
    Action: FnOnce(),
{
    #[cfg(not(feature = "e2e"))]
    action();
    #[cfg(feature = "e2e")]
    drop(action);
}

pub fn run_host_mutation<Action>(action: Action)
where
    Action: FnOnce(),
{
    #[cfg(not(feature = "e2e"))]
    action();
    #[cfg(feature = "e2e")]
    drop(action);
}

pub fn external_home_dir() -> Result<std::path::PathBuf, String> {
    #[cfg(not(feature = "e2e"))]
    return dirs::home_dir().ok_or_else(|| "Dossier utilisateur indisponible".to_string());

    #[cfg(feature = "e2e")]
    {
        let profile = crate::services::paths::data_dir();
        let home = profile.join("e2e-home");
        std::fs::create_dir_all(&home).map_err(|_| "Analyse indisponible".to_string())?;
        let canonical_profile =
            dunce::canonicalize(profile).map_err(|_| "Analyse indisponible".to_string())?;
        let canonical_home =
            dunce::canonicalize(home).map_err(|_| "Analyse indisponible".to_string())?;
        canonical_home
            .starts_with(canonical_profile)
            .then_some(canonical_home)
            .ok_or_else(|| "Analyse indisponible".to_string())
    }
}

#[cfg(all(test, feature = "e2e"))]
mod tests {
    use super::{exit_source_name, BrowserExitSource};

    #[test]
    fn browser_exit_sources_are_fixed_categories() {
        assert_eq!(
            exit_source_name(BrowserExitSource::Initialization),
            "browser-initialization"
        );
        assert_eq!(
            exit_source_name(BrowserExitSource::LaunchCallback),
            "browser-launch-callback"
        );
        assert_eq!(
            exit_source_name(BrowserExitSource::ChildAdmission),
            "browser-child-admission"
        );
        assert_eq!(
            exit_source_name(BrowserExitSource::Supervision),
            "browser-supervision"
        );
    }
}
