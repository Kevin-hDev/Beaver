use std::ffi::{OsStr, OsString};
#[cfg(all(test, unix))]
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use super::python_runtime_path::{command_for, gui_path, locate};
use super::runtime_error::RuntimeError;
use super::runtime_manifest::RuntimeManifest;

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PROBE_CODE: &str =
    "import sys; print(sys.implementation.name); print(sys.version_info.major); print(sys.version_info.minor)";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PythonRuntime {
    pub(super) program: PathBuf,
    pub(super) prefix_args: Vec<String>,
    label: String,
    path: OsString,
}

impl PythonRuntime {
    pub(super) async fn resolve(manifest: &RuntimeManifest) -> Result<Self, RuntimeError> {
        resolve_in_path(manifest, gui_path()).await
    }

    #[cfg(all(test, unix))]
    pub(super) async fn resolve_with_path(
        manifest: &RuntimeManifest,
        path: &Path,
    ) -> Result<Self, RuntimeError> {
        let path =
            std::env::join_paths([path]).map_err(|_| RuntimeError::EnvironmentUnavailable)?;
        resolve_in_path(manifest, path).await
    }

    pub(super) fn command(&self) -> tokio::process::Command {
        let mut command = command_for(&self.program, &self.path);
        command.args(&self.prefix_args);
        command
    }

    pub(super) fn with_program(&self, program: PathBuf) -> Self {
        Self {
            label: program.to_string_lossy().into_owned(),
            program,
            prefix_args: Vec::new(),
            path: self.path.clone(),
        }
    }

    #[cfg(all(test, unix))]
    pub(super) fn label(&self) -> &str {
        &self.label
    }
}

async fn resolve_in_path(
    manifest: &RuntimeManifest,
    path: OsString,
) -> Result<PythonRuntime, RuntimeError> {
    for mut candidate in candidates(manifest, &path) {
        let Some(program) = locate(&candidate.program, &path) else {
            continue;
        };
        candidate.program = program;
        if probe(&candidate, manifest).await {
            return Ok(candidate);
        }
    }
    Err(RuntimeError::PythonUnavailable)
}

fn candidates(manifest: &RuntimeManifest, path: &OsStr) -> Vec<PythonRuntime> {
    let exact = format!("python{}.{}", manifest.python_major, manifest.python_minor);
    #[cfg(windows)]
    let entries = vec![
        (
            "py".to_string(),
            vec![format!(
                "-{}.{}",
                manifest.python_major, manifest.python_minor
            )],
        ),
        (exact, Vec::new()),
        ("python3".to_string(), Vec::new()),
        ("python".to_string(), Vec::new()),
    ];
    #[cfg(not(windows))]
    let entries = vec![
        (exact, Vec::new()),
        ("python3".to_string(), Vec::new()),
        ("python".to_string(), Vec::new()),
    ];
    entries
        .into_iter()
        .map(|(program, prefix_args)| {
            let label = if prefix_args.is_empty() {
                program.clone()
            } else {
                format!("{program} {}", prefix_args.join(" "))
            };
            PythonRuntime {
                program: PathBuf::from(program),
                prefix_args,
                label,
                path: path.to_os_string(),
            }
        })
        .collect()
}

async fn probe(candidate: &PythonRuntime, manifest: &RuntimeManifest) -> bool {
    let mut command = candidate.command();
    command.args(["-c", PROBE_CODE]);
    super::python_probe::run(&mut command, PROBE_TIMEOUT, |_| {})
        .await
        .is_some_and(|output| probe_matches(&output, manifest))
}

#[cfg(all(test, windows))]
pub(super) async fn run_probe_for_test(
    program: PathBuf,
    args: Vec<OsString>,
    timeout: Duration,
    started: tokio::sync::oneshot::Sender<u32>,
) -> Option<Vec<u8>> {
    let mut command = crate::services::background_command::new_tokio(program);
    command.args(args);
    super::python_probe::run(&mut command, timeout, move |pid| {
        let _ = started.send(pid);
    })
    .await
}

pub(super) fn probe_matches(output: &[u8], manifest: &RuntimeManifest) -> bool {
    let Ok(output) = std::str::from_utf8(output) else {
        return false;
    };
    // `lines` normalise uniquement LF et CRLF : la même sonde reste stricte
    // tout en acceptant la sortie native de CPython sous Windows.
    let mut values = output.lines();
    let implementation = values.next();
    let major = values.next().and_then(|value| value.parse::<u8>().ok());
    let minor = values.next().and_then(|value| value.parse::<u8>().ok());
    implementation == Some("cpython")
        && major == Some(manifest.python_major)
        && minor == Some(manifest.python_minor)
        && values.next().is_none()
        && output.ends_with('\n')
}
