#[derive(Clone, Copy)]
pub enum LifecycleStage {
    SetupEntered,
    SetupCompleted,
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
