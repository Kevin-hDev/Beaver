//! Compiled only in the isolated E2E build; never available in production.
use super::InstallJobStore;
use std::sync::Mutex;

static OVERRIDE: Mutex<Option<InstallJobStore>> = Mutex::new(None);
const GIT_LOCATOR: &str = "https://beaver-e2e.invalid/install.git";
const FIXTURE_DIRECTORY: &str = "extension-install-e2e";

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Fixture {
    npm: String,
    git: String,
    local: String,
}

pub(super) fn current() -> Result<Option<InstallJobStore>, String> {
    OVERRIDE
        .lock()
        .map(|value| value.clone())
        .map_err(|_| super::limits::UNAVAILABLE.into())
}

pub(crate) fn configure(app: &tauri::AppHandle, enabled: bool) -> Result<Option<Fixture>, String> {
    let mut store = super::super::runtime::global()?.install_jobs.clone();
    if store
        .snapshot()?
        .jobs
        .iter()
        .any(|job| !job.status.terminal())
    {
        return Err(super::limits::BUSY.into());
    }
    let mut current = OVERRIDE.lock().map_err(|_| super::limits::UNAVAILABLE)?;
    if !enabled {
        *current = None;
        return Ok(None);
    }
    let directory = crate::services::paths::data_dir().join(FIXTURE_DIRECTORY);
    crate::services::private_store::ensure_private_dir(&directory)?;
    let cli = directory.join("npm.mjs");
    crate::services::private_store::atomic_write(&cli, include_bytes!("volume_fixture.mjs"))?;
    let ui = super::super::ui_builder::UiBuildRuntime::resolve(app)
        .map_err(|error| error.code().to_string())?;
    let npm = super::super::npm_runner::NpmRunner::for_test(ui.node.clone(), cli);
    store.executor = Some(super::executor::ProductionExecutor::for_test(npm, ui));
    store.disk_policy = super::disk_policy::DiskPolicy {
        warning_bytes: 1024,
        reserve_bytes: 1024,
        poll_interval: std::time::Duration::from_millis(10),
    };
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let local = prepare_sources(&directory)?;
    *current = Some(store);
    Ok(Some(Fixture {
        npm: format!("beaver-e2e-volume-{nonce}"),
        git: GIT_LOCATOR.into(),
        local,
    }))
}

pub(super) fn git_source(
    source: super::super::source_validation::GitSource,
) -> Result<super::super::source_validation::GitSource, String> {
    if source.locator != GIT_LOCATOR || current()?.is_none() {
        return Ok(source);
    }
    let root = crate::services::paths::data_dir()
        .join(FIXTURE_DIRECTORY)
        .join("git");
    let canonical = root
        .canonicalize()
        .map_err(|_| super::limits::UNAVAILABLE)?;
    let url = url::Url::from_directory_path(canonical)
        .map_err(|_| super::limits::UNAVAILABLE)?
        .to_string();
    Ok(super::super::source_validation::GitSource {
        locator: source.locator,
        clone_url: url,
        reference: None,
    })
}

fn prepare_sources(directory: &std::path::Path) -> Result<String, String> {
    let git = directory.join("git");
    crate::services::private_store::ensure_private_dir(&git)?;
    let manifest = serde_json::json!({"id":"beaver.e2e.git-volume", "name":"Git volume fixture", "version":"1.0.0", "beaverApi":"1", "runtime":"node", "main":"index.mjs", "access":"full"});
    crate::services::private_store::atomic_write(
        &git.join("beaver-extension.json"),
        manifest.to_string().as_bytes(),
    )?;
    crate::services::private_store::atomic_write(&git.join("index.mjs"), b"export default {};")?;
    crate::services::private_store::atomic_write(&git.join("payload"), &[0_u8; 4096])?;
    let repository = git2::Repository::init(&git).map_err(|_| super::limits::UNAVAILABLE)?;
    let mut index = repository.index().map_err(|_| super::limits::UNAVAILABLE)?;
    index
        .add_all(
            ["beaver-extension.json", "index.mjs", "payload"],
            git2::IndexAddOption::DEFAULT,
            None,
        )
        .map_err(|_| super::limits::UNAVAILABLE)?;
    let tree_id = index.write_tree().map_err(|_| super::limits::UNAVAILABLE)?;
    let tree = repository
        .find_tree(tree_id)
        .map_err(|_| super::limits::UNAVAILABLE)?;
    let signature = git2::Signature::now("Fixture", "fixture@example.invalid")
        .map_err(|_| super::limits::UNAVAILABLE)?;
    let parent = repository
        .head()
        .ok()
        .and_then(|head| head.peel_to_commit().ok());
    repository
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            "fixture",
            &tree,
            &parent.iter().collect::<Vec<_>>(),
        )
        .map_err(|_| super::limits::UNAVAILABLE)?;
    // Local installs must leave this directory unchanged; no npm dependencies here.
    git.to_str()
        .map(str::to_owned)
        .ok_or_else(|| super::limits::UNAVAILABLE.into())
}
