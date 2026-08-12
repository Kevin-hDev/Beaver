use super::owned_probe::{self, ProbeSpec};
use crate::services::work_registry::ServiceWorkCancellation;

const OWNED_GPU_SCRIPT: &str = r#"
$total = 0
$used = 0
try {
  $registry = Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0*' -Name HardwareInformation.qwMemorySize -ErrorAction Stop
  $total = [UInt64](($registry | Measure-Object -Property 'HardwareInformation.qwMemorySize' -Maximum).Maximum)
} catch { $ignoredRegistryError = $_ }
try {
  $samples = Get-CimInstance -ClassName Win32_PerfFormattedData_GPUPerformanceCounters_GPUMemory -ErrorAction Stop
  $dedicatedUsed = ($samples | Measure-Object -Property DedicatedUsage -Sum).Sum
  $sharedUsed = ($samples | Measure-Object -Property SharedUsage -Sum).Sum
  $dedicatedLimit = ($samples | Measure-Object -Property DedicatedLimit -Sum).Sum
  $sharedLimit = ($samples | Measure-Object -Property SharedLimit -Sum).Sum
  if ($null -ne $dedicatedUsed) { $used += $dedicatedUsed }
  if ($null -ne $sharedUsed) { $used += $sharedUsed }
  if ($total -eq 0 -and $null -ne $dedicatedLimit) { $total += $dedicatedLimit }
  if ($total -eq 0 -and $null -ne $sharedLimit) { $total += $sharedLimit }
} catch { $ignoredCounterError = $_ }
Write-Output "$total,$used"
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
    let output = owned_probe::run(
        ProbeSpec::new("powershell").args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            OWNED_GPU_SCRIPT,
        ]),
        cancel,
    )
    .await?;
    parse_pair_rows(&output.stdout, output.truncated, 1_048_576)
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

pub(super) fn detect_total() -> Option<u64> {
    if let Some(v) = nvidia_smi_vram() {
        return Some(v);
    }
    if let Some(v) = registry_vram() {
        return Some(v);
    }
    if let Some(v) = windows_gpu_counter_total() {
        return Some(v);
    }
    None
}

pub(super) fn detect_used() -> Option<u64> {
    if let Some(v) = nvidia_smi_field("memory.used") {
        return Some(v);
    }
    windows_gpu_counter_used()
}

fn registry_vram() -> Option<u64> {
    let script = "Get-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}\\0*' -Name HardwareInformation.qwMemorySize -ErrorAction SilentlyContinue | Select-Object -ExpandProperty 'HardwareInformation.qwMemorySize' | Measure-Object -Maximum | Select-Object -ExpandProperty Maximum";
    let bytes = run_powershell_u64(script)?;
    if bytes > 0 {
        Some(bytes / 1_048_576)
    } else {
        None
    }
}

fn windows_gpu_counter_used() -> Option<u64> {
    let script = r#"
$sum = 0
try {
  $samples = Get-CimInstance -ClassName Win32_PerfFormattedData_GPUPerformanceCounters_GPUMemory -ErrorAction Stop
  $dedicated = ($samples | Measure-Object -Property DedicatedUsage -Sum).Sum
  $shared = ($samples | Measure-Object -Property SharedUsage -Sum).Sum
  if ($null -ne $dedicated) { $sum += $dedicated }
  if ($null -ne $shared) { $sum += $shared }
} catch {
  $ignoredCimError = $_
}
if ($sum -eq 0) {
  foreach ($path in @('\GPU Adapter Memory(*)\Dedicated Usage', '\GPU Adapter Memory(*)\Shared Usage')) {
    try {
      $value = ((Get-Counter $path -ErrorAction Stop).CounterSamples | Measure-Object -Property CookedValue -Sum).Sum
      if ($null -ne $value) { $sum += $value }
    } catch {
      $ignoredCounterError = $_
    }
  }
}
[UInt64]$sum
"#;
    let bytes = run_powershell_u64(script)?;
    Some(bytes / 1_048_576)
}

fn windows_gpu_counter_total() -> Option<u64> {
    let script = r#"
$sum = 0
try {
  $samples = Get-CimInstance -ClassName Win32_PerfFormattedData_GPUPerformanceCounters_GPUMemory -ErrorAction Stop
  $dedicated = ($samples | Measure-Object -Property DedicatedLimit -Sum).Sum
  $shared = ($samples | Measure-Object -Property SharedLimit -Sum).Sum
  if ($null -ne $dedicated) { $sum += $dedicated }
  if ($null -ne $shared) { $sum += $shared }
} catch {
  $ignoredCimError = $_
}
if ($sum -eq 0) {
  foreach ($path in @('\GPU Adapter Memory(*)\Dedicated Limit', '\GPU Adapter Memory(*)\Shared Limit')) {
    try {
      $value = ((Get-Counter $path -ErrorAction Stop).CounterSamples | Measure-Object -Property CookedValue -Sum).Sum
      if ($null -ne $value) { $sum += $value }
    } catch {
      $ignoredCounterError = $_
    }
  }
}
[UInt64]$sum
"#;
    let bytes = run_powershell_u64(script)?;
    Some(bytes / 1_048_576)
}

fn run_powershell_u64(script: &str) -> Option<u64> {
    let mut cmd = crate::services::background_command::new("powershell");
    cmd.args(["-NoProfile", "-Command", script]);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines().find_map(|line| line.trim().parse::<u64>().ok())
}

fn nvidia_smi_field(field: &str) -> Option<u64> {
    let output = crate::services::background_command::new("nvidia-smi")
        .args([
            &format!("--query-gpu={field}"),
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines().next()?.trim().parse::<u64>().ok()
}

fn nvidia_smi_vram() -> Option<u64> {
    nvidia_smi_field("memory.total")
}
