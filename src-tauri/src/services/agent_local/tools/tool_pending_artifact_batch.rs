use super::tool_pending_artifact_errors::{invalid_result, too_large_result};
use super::tool_pending_artifact_inspect::inspect_result;
use super::tool_pending_artifact_read::read_result;
use super::tool_pending_artifact_revalidate::revalidate_resources;
use super::types_tools::ToolResult;
use tokio_util::sync::CancellationToken;

const MAX_BATCH_ARTIFACT_BYTES: u64 =
    crate::services::extensions::types::MAX_PARALLEL_EPHEMERAL_ARTIFACT_BYTES as u64;

pub(crate) struct BatchArtifactBudget {
    used_bytes: u64,
}

impl BatchArtifactBudget {
    pub(crate) fn new() -> Self {
        Self { used_bytes: 0 }
    }

    fn admit(&mut self, bytes: u64) -> Result<(), ()> {
        let used = self.used_bytes.checked_add(bytes).ok_or(())?;
        if used > MAX_BATCH_ARTIFACT_BYTES {
            return Err(());
        }
        self.used_bytes = used;
        Ok(())
    }
}

pub(crate) async fn resolve_batch(
    results: &mut [Option<ToolResult>],
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
) {
    if cancel.is_cancelled() {
        cancel_results(results);
        return;
    }
    revalidate_resources(results).await;
    let resolving: Vec<bool> = results
        .iter()
        .map(|result| {
            result.as_ref().is_some_and(|result| {
                !result.is_error
                    && (!result.pending_artifacts().is_empty()
                        || result.has_pending_extension_resource())
            })
        })
        .collect();
    if !resolving.iter().any(|resolving| *resolving) {
        return;
    }
    let len = results.len();
    let input = results
        .iter_mut()
        .zip(&resolving)
        .map(|(result, resolving)| resolving.then(|| result.take()).flatten())
        .collect();
    let root = working_dir.to_path_buf();
    let worker_cancel = cancel.clone();
    let worker = tokio::task::spawn_blocking(move || {
        let mut budget = BatchArtifactBudget::new();
        resolve_with_budget(input, &root, &worker_cancel, &mut budget, None)
    });
    let output = tokio::select! {
        output = worker => Some(output.unwrap_or_else(|_| unavailable_results(len))),
        // Blocking I/O cannot be killed mid-syscall. The shared token stops the
        // reader at its next chunk boundary; a cancelled result is never published.
        _ = cancel.cancelled() => None,
    };
    if let Some(output) = output {
        for ((slot, resolving), result) in results.iter_mut().zip(&resolving).zip(output) {
            if *resolving {
                *slot = result;
            }
        }
    } else {
        cancel_resolving(results, &resolving);
    }
}

pub(super) fn resolve_with_budget(
    results: Vec<Option<ToolResult>>,
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
    budget: &mut BatchArtifactBudget,
    key: Option<&[u8]>,
) -> Vec<Option<ToolResult>> {
    resolve_with_budget_inner(results, working_dir, cancel, budget, key, true)
}

#[cfg(test)]
pub(super) fn resolve_with_unavailable_workspace_key(
    results: Vec<Option<ToolResult>>,
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
    budget: &mut BatchArtifactBudget,
) -> Vec<Option<ToolResult>> {
    resolve_with_budget_inner(results, working_dir, cancel, budget, None, false)
}

fn resolve_with_budget_inner(
    mut results: Vec<Option<ToolResult>>,
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
    budget: &mut BatchArtifactBudget,
    key: Option<&[u8]>,
    load_workspace_key: bool,
) -> Vec<Option<ToolResult>> {
    let mut prepared = Vec::new();
    for (index, slot) in results.iter_mut().enumerate() {
        let Some(result) = slot.take() else { continue };
        if result.is_error
            || (result.pending_artifacts().is_empty() && !result.has_pending_extension_resource())
        {
            *slot = Some(result);
            continue;
        }
        if cancel.is_cancelled() {
            *slot = Some(ToolResult::cancelled("Annulé."));
            continue;
        }
        match inspect_result(result, working_dir, cancel) {
            Ok(prepared_result) => prepared.push((index, prepared_result)),
            Err(result) => *slot = Some(result),
        }
    }

    let mut admitted = Vec::new();
    for (index, prepared_result) in prepared {
        if budget.admit(prepared_result.bytes).is_err() {
            results[index] = Some(too_large_result());
        } else {
            admitted.push((index, prepared_result));
        }
    }
    if admitted.is_empty() {
        return results;
    }
    let needs_key = admitted
        .iter()
        .any(|(_, prepared_result)| !prepared_result.files.is_empty());
    let key = if needs_key {
        match key {
            Some(key) => Some(zeroize::Zeroizing::new(key.to_vec())),
            None if load_workspace_key => crate::services::attachment_access::attachment_key().ok(),
            None => None,
        }
    } else {
        None
    };
    for (index, prepared_result) in admitted {
        if !prepared_result.files.is_empty() && key.is_none() {
            results[index] = Some(invalid_result());
        } else {
            results[index] = Some(read_result(
                prepared_result,
                cancel,
                key.as_ref().map(|key| key.as_slice()),
            ));
        }
    }
    results
}

fn cancel_results(results: &mut [Option<ToolResult>]) {
    for result in results.iter_mut().filter_map(Option::as_mut) {
        if !result.pending_artifacts().is_empty() || result.has_pending_extension_resource() {
            *result = ToolResult::cancelled("Annulé.");
        }
    }
}

fn cancel_resolving(results: &mut [Option<ToolResult>], resolving: &[bool]) {
    for (slot, resolving) in results.iter_mut().zip(resolving) {
        if *resolving {
            *slot = Some(ToolResult::cancelled("Annulé."));
        }
    }
}

fn unavailable_results(len: usize) -> Vec<Option<ToolResult>> {
    (0..len).map(|_| Some(invalid_result())).collect()
}

#[cfg(test)]
#[path = "tool_pending_artifact_batch_tests.rs"]
mod tests;
