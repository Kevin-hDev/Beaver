use crate::services::agent_local::types_tools::ShellOutput;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};

const DEFAULT_STARTUP_TIMEOUT: u64 = 30;
const MAX_STARTUP_TIMEOUT: u64 = 60;
const POLL_INTERVAL_MS: u64 = 100;
const BACKGROUND_LINE_BUFFER: usize = 256;
const MAX_CAPTURE_BYTES: usize = 64 * 1024;
const TRUNCATION_MARKER: &str = "[sortie tronquée pendant le démarrage]";
const BACKGROUND_PATTERNS: &str = "npm run dev|pnpm dev|yarn dev|bun dev|npm start|vite|next dev|next start|nuxt dev|astro dev|svelte-kit dev|tauri dev|cargo tauri dev|python -m http.server|http-server|uvicorn |flask run|rails server|cargo watch|--watch|tail -f|docker logs -f|kubectl logs -f|kubectl port-forward|ngrok |while true";
const READY_MARKERS: &str = "local:|localhost|127.0.0.1|listening|ready|server running|running at|compiled successfully|application running|press h + enter|http://|https://";

#[derive(Clone, Copy)]
enum StreamKind {
    Stdout,
    Stderr,
}

struct LineEvent {
    kind: StreamKind,
    line: String,
}

pub fn should_run_in_background(command: &str) -> bool {
    let cmd = command.to_ascii_lowercase();
    BACKGROUND_PATTERNS
        .split('|')
        .any(|pattern| cmd.contains(pattern))
}

pub async fn execute_background_shell(
    command: &str,
    working_dir: &Path,
    timeout_secs: Option<u64>,
) -> Result<ShellOutput, String> {
    super::security::check_destructive_command(command)?;
    let (shell, flag) = super::tool_bash::detect_shell();
    let mut child = Command::new(&shell)
        .args([&flag, command])
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Erreur lancement shell: {e}"))?;

    let (tx, mut rx) = mpsc::channel(BACKGROUND_LINE_BUFFER);
    spawn_child_readers(&mut child, tx);

    let deadline = Instant::now() + startup_timeout(timeout_secs);
    let mut stdout = String::new();
    let mut stderr = String::new();

    loop {
        drain_pending_lines(&mut rx, &mut stdout, &mut stderr);
        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("Erreur exécution: {e}"))?
        {
            drain_pending_lines(&mut rx, &mut stdout, &mut stderr);
            return Ok(ShellOutput {
                stdout: super::tool_bash::truncate_output(&stdout),
                stderr: super::tool_bash::truncate_output(&stderr),
                exit_code: status.code().unwrap_or(-1),
                timed_out: false,
                new_cwd: None,
                affected_paths: Vec::new(),
                file_changes: Vec::new(),
            });
        }

        if output_is_ready(&stdout) || output_is_ready(&stderr) || Instant::now() >= deadline {
            return Ok(background_started_output(child, stdout, stderr));
        }

        tokio::select! {
            event = rx.recv() => {
                if let Some(event) = event {
                    append_event(event, &mut stdout, &mut stderr);
                }
            }
            _ = sleep(Duration::from_millis(POLL_INTERVAL_MS)) => {}
        }
    }
}

fn spawn_child_readers(child: &mut Child, tx: mpsc::Sender<LineEvent>) {
    if let Some(stdout) = child.stdout.take() {
        spawn_reader(StreamKind::Stdout, stdout, tx.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_reader(StreamKind::Stderr, stderr, tx);
    }
}

fn spawn_reader<R>(kind: StreamKind, stream: R, tx: mpsc::Sender<LineEvent>)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(stream).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx.send(LineEvent { kind, line }).await.is_err() {
                break;
            }
        }
    });
}

fn drain_pending_lines(
    rx: &mut mpsc::Receiver<LineEvent>,
    stdout: &mut String,
    stderr: &mut String,
) {
    while let Ok(event) = rx.try_recv() {
        append_event(event, stdout, stderr);
    }
}

fn append_event(event: LineEvent, stdout: &mut String, stderr: &mut String) {
    let target = match event.kind {
        StreamKind::Stdout => stdout,
        StreamKind::Stderr => stderr,
    };
    if !target.is_empty() {
        target.push('\n');
    }
    target.push_str(&event.line);
    trim_capture(target);
}

fn trim_capture(output: &mut String) {
    if output.len() <= MAX_CAPTURE_BYTES {
        return;
    }

    let marker_len = TRUNCATION_MARKER.len() + 1;
    let budget = MAX_CAPTURE_BYTES.saturating_sub(marker_len);
    let start = output
        .char_indices()
        .rev()
        .find(|(idx, _)| output.len().saturating_sub(*idx) <= budget)
        .map(|(idx, _)| idx)
        .unwrap_or(output.len());
    let tail = output[start..].to_string();
    output.clear();
    output.push_str(TRUNCATION_MARKER);
    output.push('\n');
    output.push_str(&tail);
}

fn output_is_ready(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    READY_MARKERS
        .split('|')
        .any(|needle| lower.contains(needle))
}

fn background_started_output(child: Child, mut stdout: String, stderr: String) -> ShellOutput {
    let (_id, pid) = super::tool_bash_background::register_background_process(child);
    let pid_label = pid
        .map(|p| p.to_string())
        .unwrap_or_else(|| "inconnu".to_string());
    let note = format!("[Commande longue active en arrière-plan: pid {pid_label}]");
    if stdout.is_empty() {
        stdout = note;
    } else {
        stdout.push_str("\n\n");
        stdout.push_str(&note);
    }
    eprintln!("[bash-background] commande longue suivie: pid={pid_label}");
    ShellOutput {
        stdout: super::tool_bash::truncate_output(&stdout),
        stderr: super::tool_bash::truncate_output(&stderr),
        exit_code: 0,
        timed_out: false,
        new_cwd: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
    }
}

fn startup_timeout(timeout_secs: Option<u64>) -> Duration {
    let secs = timeout_secs
        .unwrap_or(DEFAULT_STARTUP_TIMEOUT)
        .clamp(1, MAX_STARTUP_TIMEOUT);
    Duration::from_secs(secs)
}
