use super::owned_probe::{self, ProbeSpec};
use crate::services::work_registry::ServiceWorkCancellation;

const OWNED_GPU_SCRIPT: &str = r#"
$registryTotal = 0
$cimTotal = 0
$cimUsed = 0
$counterTotal = 0
$counterUsed = 0
try {
  $registry = Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0*' -Name HardwareInformation.qwMemorySize -ErrorAction Stop
  $registryTotal = [UInt64](($registry | Measure-Object -Property 'HardwareInformation.qwMemorySize' -Maximum).Maximum)
} catch { $ignoredRegistryError = $_ }
try {
  $samples = Get-CimInstance -ClassName Win32_PerfFormattedData_GPUPerformanceCounters_GPUMemory -ErrorAction Stop
  $used = ($samples | Measure-Object -Property DedicatedUsage -Sum).Sum
  $limit = ($samples | Measure-Object -Property DedicatedLimit -Sum).Sum
  if ($null -ne $used) { $cimUsed = [UInt64]$used }
  if ($null -ne $limit) { $cimTotal = [UInt64]$limit }
} catch { $ignoredCounterError = $_ }
# Zero usage is a valid idle measurement. Only fall back when CIM has no total,
# so one refresh never combines counters describing different adapter sets.
if ($cimTotal -eq 0) {
  try {
    $counterTotal = [UInt64](((Get-Counter '\GPU Adapter Memory(*)\Dedicated Limit' -ErrorAction Stop).CounterSamples | Measure-Object -Property CookedValue -Sum).Sum)
    $counterUsed = [UInt64](((Get-Counter '\GPU Adapter Memory(*)\Dedicated Usage' -ErrorAction Stop).CounterSamples | Measure-Object -Property CookedValue -Sum).Sum)
  } catch { $ignoredLegacyCounterError = $_ }
}
Write-Output "$registryTotal,$cimTotal,$cimUsed,$counterTotal,$counterUsed"
"#;

pub(super) async fn detect_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, u64)> {
    let nvidia = nvidia_smi_info_owned(cancel);
    let system = powershell_info_owned(cancel);
    let (nvidia, system) = tokio::join!(nvidia, system);
    nvidia.or(system)
}

async fn nvidia_smi_info_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, u64)> {
    let output = owned_probe::run(
        ProbeSpec::new("nvidia-smi").args([
            "--query-gpu=memory.total,memory.used",
            "--format=csv,noheader,nounits",
        ]),
        cancel,
    )
    .await?;
    parse_pair_rows(&output.stdout, output.truncated, 1)
}

async fn powershell_info_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, u64)> {
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
    super::windows_snapshot::parse_sources(&output.stdout, output.truncated)
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
