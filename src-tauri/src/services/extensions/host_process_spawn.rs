use super::HostProcess;
use crate::services::extensions::error_codes;
use crate::services::extensions::host_channel::PendingRequests;
use crate::services::extensions::host_identity::HostIdentity;
use crate::services::extensions::host_load_tracker::HostLoadTracker;
use crate::services::extensions::host_paths::HostPaths;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

pub(in crate::services::extensions) struct HostSpawnBinding {
    identity: HostIdentity,
    generation: Arc<crate::services::extensions::runtime_hosts::HostGeneration>,
    revoked: tokio_util::sync::CancellationToken,
    exit_sender:
        tokio::sync::mpsc::Sender<crate::services::extensions::runtime_hosts::HostExitNotice>,
}

impl HostSpawnBinding {
    pub(in crate::services::extensions) fn new(
        identity: HostIdentity,
        generation: Arc<crate::services::extensions::runtime_hosts::HostGeneration>,
        revoked: tokio_util::sync::CancellationToken,
        exit_sender: tokio::sync::mpsc::Sender<
            crate::services::extensions::runtime_hosts::HostExitNotice,
        >,
    ) -> Self {
        Self {
            identity,
            generation,
            revoked,
            exit_sender,
        }
    }
}

impl HostProcess {
    pub async fn spawn_bound(
        paths: &HostPaths,
        work: &crate::services::extensions::work_supervision::ExtensionWorkServices,
        binding: HostSpawnBinding,
        deadline: std::time::Instant,
        temporary_directory: &std::path::Path,
    ) -> Result<Self, String> {
        Self::spawn_inner(paths, work, binding, deadline, temporary_directory, None).await
    }

    #[cfg(test)]
    pub async fn spawn(
        paths: &HostPaths,
        work: &crate::services::extensions::work_supervision::ExtensionWorkServices,
    ) -> Result<Self, String> {
        let temporary =
            tempfile::tempdir().map_err(|_| error_codes::HOST_UNAVAILABLE.to_string())?;
        let path = temporary.path().to_path_buf();
        Self::spawn_inner(
            paths,
            work,
            HostSpawnBinding::new(
                HostIdentity::Official,
                Arc::new(crate::services::extensions::runtime_hosts::HostGeneration::new(1)),
                tokio_util::sync::CancellationToken::new(),
                tokio::sync::mpsc::channel(1).0,
            ),
            crate::services::extensions::runtime_lifecycle::new_stop_deadline(),
            &path,
            Some(temporary),
        )
        .await
    }

    async fn spawn_inner(
        paths: &HostPaths,
        work: &crate::services::extensions::work_supervision::ExtensionWorkServices,
        binding: HostSpawnBinding,
        deadline: std::time::Instant,
        temporary_directory: &std::path::Path,
        owned_temporary_directory: Option<tempfile::TempDir>,
    ) -> Result<Self, String> {
        let HostSpawnBinding {
            identity,
            generation,
            revoked,
            exit_sender,
        } = binding;
        let generation_number = generation.number;
        let reader_admission = work
            .try_admit_reader()
            .map_err(|error| error.public_code().to_string())?;
        let mut command = Command::new(&paths.node);
        command
            .arg(&paths.script)
            .current_dir(&paths.directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let path = crate::services::extensions::process_environment::inherited_path()
            .map_err(|_| error_codes::HOST_UNAVAILABLE.to_string())?;
        crate::services::extensions::process_environment::configure_host(
            &mut command,
            path,
            temporary_directory,
        )
        .map_err(|_| error_codes::HOST_UNAVAILABLE.to_string())?;
        let (mut child, process_scope) =
            crate::services::owned_process::OwnedProcess::spawn_tokio_scoped(
                &mut command,
                crate::services::process_tree::ProcessKind::ExtensionHost,
            )
            .await
            .map_err(|_| error_codes::HOST_UNAVAILABLE.to_string())?;
        let pid = child
            .id()
            .ok_or_else(|| error_codes::HOST_UNAVAILABLE.to_string())?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| error_codes::HOST_UNAVAILABLE.to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| error_codes::HOST_UNAVAILABLE.to_string())?;
        let writer = Arc::new(Mutex::new(stdin));
        let pending: PendingRequests = Arc::new(std::sync::Mutex::new(HashMap::new()));
        let alive = Arc::new(AtomicBool::new(true));
        let reader_cancel = tokio_util::sync::CancellationToken::new();
        let load_tracker = Arc::new(HostLoadTracker::default());
        let log_identity = identity.clone();
        let reader_finished = match reader_admission.spawn_with_completion({
            let work = work.clone();
            let writer = writer.clone();
            let pending = pending.clone();
            let alive = alive.clone();
            let revoked = revoked.clone();
            let reader_cancel = reader_cancel.clone();
            let load_tracker = load_tracker.clone();
            move |cancel| async move {
                crate::services::extensions::host_reader::run(
                    stdout,
                    crate::services::extensions::host_reader::HostReaderChannel {
                        writer,
                        pending,
                        alive,
                        revoked,
                        reader_cancel,
                        load_tracker,
                    },
                    work,
                    cancel,
                    crate::services::extensions::host_reader::HostAuthority {
                        identity,
                        generation,
                        exit_sender,
                    },
                )
                .await;
            }
        }) {
            Ok(completion) => completion,
            Err(_) => {
                crate::services::process_tree::terminate_tokio_scoped(
                    &mut child,
                    crate::services::process_tree::ProcessKind::ExtensionHost,
                    &process_scope,
                    pid,
                    deadline,
                )
                .await;
                return Err(error_codes::HOST_UNAVAILABLE.to_string());
            }
        };
        crate::services::extensions::access_log::write_host_started(
            log_identity,
            generation_number,
            pid,
        );
        Ok(Self {
            child: Mutex::new(child),
            writer,
            pending,
            alive,
            reader_cancel,
            load_tracker,
            reader_done: Mutex::new(Some(reader_finished)),
            process_scope,
            root_pid: pid,
            _test_temporary_directory: owned_temporary_directory,
        })
    }
}
