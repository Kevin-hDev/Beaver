use crate::services::agent_local::tool_hooks::run_post_hooks;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use futures_util::future::join_all;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use super::tool_executor_helpers::post_record_read;

const MAX_PARALLEL: usize = 10;
const READ_BATCH_TIMEOUT: Duration = Duration::from_secs(600);

pub(super) struct BatchEntry<'a> {
    pub global_idx: usize,
    pub name: &'a str,
    pub effective_args: &'a Value,
}

pub(super) async fn flush_read_batch<'a>(
    batch: &[BatchEntry<'a>],
    indexed_results: &mut [Option<(&'a str, ToolResult)>],
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
    write_guard: &mut WriteGuard,
    eager_results: &mut Option<&mut HashMap<usize, ToolResult>>,
    session_id: &str,
    request_id: &str,
) {
    for chunk in batch.chunks(MAX_PARALLEL) {
        if cancel.is_cancelled() {
            for entry in chunk {
                indexed_results[entry.global_idx] = Some((entry.name, ToolResult::err("Annulé.")));
            }
            continue;
        }

        let mut pending_indices: Vec<usize> = Vec::new();
        let mut chunk_results: Vec<Option<ToolResult>> = vec![None; chunk.len()];

        for (pos, entry) in chunk.iter().enumerate() {
            if let Some(ref mut eager) = eager_results.as_deref_mut() {
                if let Some(tr) = eager.remove(&entry.global_idx) {
                    post_record_read(
                        entry.name,
                        entry.effective_args,
                        working_dir,
                        &tr,
                        write_guard,
                    );
                    let tr = run_post_hooks(entry.name, entry.effective_args, tr);
                    chunk_results[pos] = Some(tr);
                    continue;
                }
            }
            pending_indices.push(pos);
        }

        if !pending_indices.is_empty() {
            let futs: Vec<_> = pending_indices
                .iter()
                .map(|&pos| dispatch_pending(&chunk[pos], working_dir, session_id, request_id))
                .collect();
            let dispatched = tokio::select! {
                results = join_all(futs) => Some(results),
                _ = cancel.cancelled() => None,
                _ = tokio::time::sleep(READ_BATCH_TIMEOUT) => None,
            };
            if let Some(dispatched) = dispatched {
                for (pos, tr) in pending_indices.iter().zip(dispatched) {
                    let entry = &chunk[*pos];
                    post_record_read(
                        entry.name,
                        entry.effective_args,
                        working_dir,
                        &tr,
                        write_guard,
                    );
                    let tr = run_post_hooks(entry.name, entry.effective_args, tr);
                    chunk_results[*pos] = Some(tr);
                }
            } else {
                let msg = if cancel.is_cancelled() {
                    "Annulé."
                } else {
                    "Timeout du batch de lectures."
                };
                for pos in &pending_indices {
                    chunk_results[*pos] = Some(ToolResult::err(msg));
                }
            }
        }

        for (pos, entry) in chunk.iter().enumerate() {
            let tr = chunk_results[pos]
                .take()
                .unwrap_or_else(|| ToolResult::err("Annulé."));
            indexed_results[entry.global_idx] = Some((entry.name, tr));
        }
    }
}

async fn dispatch_pending(
    entry: &BatchEntry<'_>,
    working_dir: &std::path::Path,
    session_id: &str,
    request_id: &str,
) -> ToolResult {
    super::tool_executor_parallel_dispatch::dispatch_read(
        entry.name,
        entry.effective_args,
        working_dir,
        session_id,
        request_id,
    )
    .await
}
