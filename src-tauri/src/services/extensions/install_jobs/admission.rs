use super::{limits::*, store::Job, types::*, InstallJobStore};
use tokio_util::sync::CancellationToken;

impl InstallJobStore {
    pub(crate) fn start(&self, request: InstallRequest) -> Result<InstallJobView, String> {
        self.start_with(request, || {})
    }
    pub(super) fn start_with(
        &self,
        request: InstallRequest,
        before_spawn: impl FnOnce(),
    ) -> Result<InstallJobView, String> {
        let (request, key, kind) = super::request::normalize(request)?;
        let mut state = self.lock()?;
        if !self.work.is_open() {
            return Err(UNAVAILABLE.into());
        }
        if let Some(job) = state
            .jobs
            .iter()
            .find(|job| job.key == key && !job.view.status.terminal())
        {
            return state
                .snapshot()
                .jobs
                .into_iter()
                .find(|view| view.id == job.view.id)
                .ok_or_else(|| INVALID.into());
        }
        let executor = self.executor.clone().ok_or(UNAVAILABLE)?;
        if state
            .jobs
            .iter()
            .filter(|job| !job.view.status.terminal())
            .count()
            >= MAX_ACTIVE
        {
            return Err(BUSY.into());
        }
        let admission = if state.worker {
            None
        } else {
            Some(
                self.work
                    .try_admit_operation()
                    .map_err(|error| error.public_code().to_string())?,
            )
        };
        if state
            .jobs
            .iter()
            .any(|job| job.view.status.terminal() && !job.clean)
        {
            return Err(BUSY.into());
        }
        state.evict(MAX_RECENT - 1)?;
        let view = InstallJobView {
            id: uuid::Uuid::new_v4().to_string(),
            revision: 0,
            kind,
            // A source can contain a private path or credential: it is never a public label.
            display_name: match kind {
                InstallKind::Local => "Local",
                InstallKind::Git => "Git",
                InstallKind::Npm => "npm",
                InstallKind::Update => "Extension",
            }
            .into(),
            status: InstallStatus::Queued,
            phase: InstallPhase::Resolving,
            downloaded_bytes: None,
            download_total_bytes: None,
            occupied_bytes: 0,
            free_bytes: None,
            confirmation_id: None,
            error_code: None,
            extension_id: None,
            can_cancel: true,
            can_resume: false,
            queue_blocker: None,
        };
        let id = view.id.clone();
        state.jobs.push(Job {
            view,
            request,
            key,
            cancel: CancellationToken::new(),
            clean: true,
            finished_revision: None,
        });
        if let Some(admission) = admission {
            state.worker = true;
            before_spawn();
            let store = self.clone();
            if admission
                .spawn(move |shutdown| super::worker::run(store, executor, shutdown))
                .is_err()
            {
                state.worker = false;
                let index = state.index(&id)?;
                let revision = state.revision + 1;
                let job = &mut state.jobs[index];
                job.finished_revision = Some(revision);
                job.view.status = InstallStatus::Failed;
                job.view.can_cancel = false;
                job.view.error_code = Some(FAILED.into());
                self.changed(&mut state);
                return Err(UNAVAILABLE.into());
            }
        }
        self.changed(&mut state);
        state
            .snapshot()
            .jobs
            .into_iter()
            .find(|job| job.id == id)
            .ok_or_else(|| INVALID.into())
    }
}
