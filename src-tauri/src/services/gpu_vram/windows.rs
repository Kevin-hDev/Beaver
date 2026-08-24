use super::owned_probe::{self, ProbeSpec};
use crate::services::work_registry::ServiceWorkCancellation;

const OWNED_GPU_SCRIPT: &str = r#"
$samples = Get-CimInstance -ClassName Win32_PerfFormattedData_GPUPerformanceCounters_GPUAdapterMemory -ErrorAction Stop
foreach ($sample in $samples) {
  $used = [UInt64]$sample.DedicatedUsage
  Write-Output "$($sample.Name),$used"
}
"#;

pub(super) async fn detect_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, Option<u64>)> {
    let nvidia = nvidia_smi_info_owned(cancel);
    let system = powershell_info_owned(cancel);
    let (nvidia, system) = tokio::join!(nvidia, system);
    nvidia.or(system)
}

async fn nvidia_smi_info_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, Option<u64>)> {
    let output = owned_probe::run(
        ProbeSpec::new("nvidia-smi").args([
            "--query-gpu=memory.total,memory.used",
            "--format=csv,noheader,nounits",
        ]),
        cancel,
    )
    .await?;
    parse_pair_rows(&output.stdout, output.truncated, 1)
        .map(|(total_mb, used_mb)| (total_mb, Some(used_mb)))
}

async fn powershell_info_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, Option<u64>)> {
    let capacities = tokio::task::spawn_blocking(super::windows_dxgi::capacities)
        .await
        .ok()
        .flatten()?;
    if cancel.is_cancelled() {
        return None;
    }
    let powershell = crate::services::system_executable::powershell().ok()?;
    let output = owned_probe::run(
        ProbeSpec::new(powershell).args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            OWNED_GPU_SCRIPT,
        ]),
        cancel,
    )
    .await?;
    let usages = super::windows_snapshot::parse_usage_rows(&output.stdout, output.truncated)
        .unwrap_or_default();
    // DXGI and the Windows counter expose the same LUID. Joining on it prevents
    // capacity from one GPU being paired with usage from another adapter.
    super::windows_snapshot::select_snapshot(&capacities, &usages)
}

fn parse_pair_rows(bytes: &[u8], truncated: bool, divisor: u64) -> Option<(u64, u64)> {
    if truncated || divisor == 0 {
        return None;
    }
    let text = String::from_utf8_lossy(bytes);
    let mut total = 0_u64;
    let mut used = 0_u64;
    let mut found = false;
    for line in text.lines() {
        let mut fields = line.split(',').map(str::trim);
        total = total.saturating_add(fields.next()?.parse::<u64>().ok()? / divisor);
        used = used.saturating_add(fields.next()?.parse::<u64>().ok()? / divisor);
        found = true;
    }
    found.then_some((total, used))
}

#[cfg(test)]
mod tests {
    use crate::app_exit::AppExitCoordinator;
    use crate::services::work_registry::ServiceWorkSupervisor;

    #[tokio::test]
    #[ignore = "requires a physical Windows GPU and native performance counters"]
    async fn local_probe_joins_real_adapter_capacity_and_usage() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
        let admission = supervisor.try_admit().expect("probe admission");

        let (total_mb, used_mb) = super::detect_owned(&admission.cancellation())
            .await
            .expect("Windows GPU snapshot");

        assert!(total_mb >= 256);
        assert!(used_mb.is_some());
        drop(admission);
    }
}
