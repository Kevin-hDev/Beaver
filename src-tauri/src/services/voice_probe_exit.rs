use std::time::Duration;

use tauri::Manager;

#[derive(Clone, Copy)]
enum ExitProbeMode {
    Cooperative,
    Forced,
}

pub(crate) fn start_if_requested(app: &tauri::AppHandle) {
    let Some(mode) = requested_mode(std::env::args()) else {
        return;
    };
    let coordinator = app.state::<crate::app_exit::AppExitCoordinator>();
    let Ok(admission) = coordinator.work_supervisor().try_admit() else {
        return;
    };
    let cancellation = admission.cancellation_token();
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        match mode {
            ExitProbeMode::Cooperative => {
                admission.run(cancellation.cancelled()).await;
                write_cooperative_evidence();
            }
            ExitProbeMode::Forced => {
                let _admission = admission;
                std::future::pending::<()>().await;
            }
        }
    });
    let handle_for_exit = handle.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        crate::app_exit::request(&handle_for_exit, 0);
    });
}

fn write_cooperative_evidence() {
    let Ok(raw) = std::env::var("VOICE_PROBE_EXIT_EVIDENCE") else {
        return;
    };
    let requested = std::path::PathBuf::from(raw);
    if !requested.is_absolute()
        || requested.file_name().and_then(|name| name.to_str())
            != Some("voice-probe-cooperative.marker")
    {
        return;
    }
    let Some(path) = requested
        .parent()
        .and_then(|parent| parent.canonicalize().ok())
        .map(|parent| parent.join("voice-probe-cooperative.marker"))
    else {
        return;
    };
    let _ = crate::services::private_store::atomic_write(&path, b"cooperative-released\n");
}

fn requested_mode(args: impl IntoIterator<Item = String>) -> Option<ExitProbeMode> {
    let mut found = args.into_iter().filter_map(|argument| {
        argument
            .strip_prefix("--voice-probe-exit=")
            .map(str::to_owned)
    });
    let mode = match found.next()?.as_str() {
        "cooperative" => ExitProbeMode::Cooperative,
        "forced" => ExitProbeMode::Forced,
        _ => return None,
    };
    found.next().is_none().then_some(mode)
}

#[cfg(test)]
mod tests {
    #[test]
    fn accepts_only_one_known_probe_mode() {
        assert!(super::requested_mode(["app".into()]).is_none());
        assert!(super::requested_mode(["--voice-probe-exit=bad".into()]).is_none());
        assert!(super::requested_mode([
            "--voice-probe-exit=forced".into(),
            "--voice-probe-exit=cooperative".into(),
        ])
        .is_none());
        assert!(super::requested_mode(["--voice-probe-exit=forced".into()]).is_some());
    }
}
