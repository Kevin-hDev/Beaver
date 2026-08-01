use super::tool_bash_session::{CompletionKind, ShellSession, ShellSessionSnapshot};
use super::types_tools::ShellOutput;

pub fn blocked(reason: String) -> ShellOutput {
    ShellOutput {
        stdout: String::new(),
        stderr: reason,
        exit_code: -1,
        running: false,
        stopped: false,
        timed_out: false,
        tracking_incomplete: false,
        output_incomplete: false,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
    }
}

pub fn from_snapshot(session: &ShellSession, snapshot: ShellSessionSnapshot) -> ShellOutput {
    let mut stdout = snapshot.stdout;
    let mut stderr = snapshot.stderr;
    let exit_code = match snapshot.completion {
        Some(CompletionKind::Exited(code)) => code,
        Some(CompletionKind::Stopped) => {
            append_note(&mut stdout, "Processus arrêté.");
            -1
        }
        Some(CompletionKind::Cancelled) => {
            stderr = "Commande annulee.".to_string();
            -1
        }
        Some(CompletionKind::TimedOut) => {
            stderr = "Timeout de la commande atteint.".to_string();
            -1
        }
        Some(CompletionKind::Failed) => {
            stderr = "Execution shell interrompue.".to_string();
            -1
        }
        None => -1,
    };

    if snapshot.running {
        append_note(
            &mut stdout,
            &format!(
                "[Processus actif: session_id={}, pid={}, {} ms]",
                session.id(),
                session.pid(),
                snapshot.elapsed_ms
            ),
        );
    } else if stdout.trim().is_empty() {
        if exit_code == 0 {
            stdout = format!("Commande terminee en {} ms (code 0).", snapshot.elapsed_ms);
        } else if stderr.is_empty() {
            stderr = format!(
                "Commande terminee en {} ms (code {}).",
                snapshot.elapsed_ms, exit_code
            );
        }
    }
    if snapshot.output_truncated {
        if let Some(path) = snapshot.output_path {
            append_note(
                &mut stdout,
                &format!("[Résultat complet disponible : {path}]"),
            );
        }
    }
    let affected_paths = snapshot
        .changes
        .iter()
        .map(|change| change.path.clone())
        .collect();
    ShellOutput {
        stdout,
        stderr,
        exit_code,
        running: snapshot.running,
        stopped: matches!(snapshot.completion, Some(CompletionKind::Stopped)),
        timed_out: matches!(snapshot.completion, Some(CompletionKind::TimedOut)),
        tracking_incomplete: snapshot.tracking_incomplete,
        output_incomplete: snapshot.output_incomplete,
        affected_paths,
        file_changes: snapshot.changes,
    }
}

fn append_note(output: &mut String, note: &str) {
    if !output.is_empty() {
        output.push_str("\n\n");
    }
    output.push_str(note);
}
