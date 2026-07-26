use std::process::Command;

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
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-Command", script]);
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines().find_map(|line| line.trim().parse::<u64>().ok())
}

fn nvidia_smi_field(field: &str) -> Option<u64> {
    let output = Command::new("nvidia-smi")
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
