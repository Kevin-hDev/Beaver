use serde::Serialize;

const SAFETY_MARGIN_PERCENT: u64 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceFit {
    Comfortable,
    Constrained,
    Insufficient,
    Unknown,
    Cloud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuMemoryKind {
    Dedicated,
    Unified,
    Unknown,
}

impl ResourceFit {
    pub fn code(self) -> &'static str {
        match self {
            Self::Comfortable => "comfortable",
            Self::Constrained => "constrained",
            Self::Insufficient => "insufficient",
            Self::Unknown => "unknown",
            Self::Cloud => "cloud",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct HardwareProfile {
    pub gpu_memory_kind: GpuMemoryKind,
    pub vram_total_mb: Option<u64>,
    pub vram_available_mb: Option<u64>,
    pub ram_available_mb: Option<u64>,
}

pub fn detect() -> HardwareProfile {
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    let available = system.available_memory() / 1_048_576;
    let ram_available_mb = (available > 0).then_some(available);
    let vram_total_mb = crate::services::gpu_vram::detect_vram_mb();
    let gpu_memory_kind = if is_apple_silicon() && vram_total_mb.is_some() {
        GpuMemoryKind::Unified
    } else if vram_total_mb.is_some() {
        GpuMemoryKind::Dedicated
    } else {
        GpuMemoryKind::Unknown
    };
    let vram_available_mb = if gpu_memory_kind == GpuMemoryKind::Unified {
        ram_available_mb
    } else {
        vram_total_mb
            .zip(crate::services::gpu_vram::detect_vram_used_mb())
            .map(|(total, used)| total.saturating_sub(used))
    };
    HardwareProfile {
        gpu_memory_kind,
        vram_total_mb,
        vram_available_mb,
        ram_available_mb,
    }
}

pub fn resource_fit(
    model: &super::catalog::ForecastModelSpec,
    profile: HardwareProfile,
) -> ResourceFit {
    if model.is_cloud {
        return ResourceFit::Cloud;
    }
    let gpu_fit = (model.gpu_supported && model.vram_mb.is_some()).then(|| {
        profile
            .vram_available_mb
            .map_or(ResourceFit::Unknown, |available| {
                fit(model.vram_mb.unwrap_or_default() as u64, available)
            })
    });
    let cpu_fit = model.cpu_supported.then(|| {
        profile
            .ram_available_mb
            .map_or(ResourceFit::Unknown, |available| {
                fit(model.ram_mb as u64, available)
            })
    });
    best_fit(gpu_fit, cpu_fit)
}

pub fn validate_model_resources(model: &super::catalog::ForecastModelSpec) -> Result<(), String> {
    validate_model_resources_with_profile(model, detect())
}

pub(crate) fn validate_model_resources_with_profile(
    model: &super::catalog::ForecastModelSpec,
    profile: HardwareProfile,
) -> Result<(), String> {
    match resource_fit(model, profile) {
        ResourceFit::Insufficient => Err("Ressources insuffisantes pour ce modèle".into()),
        ResourceFit::Unknown if model.ram_mb > super::limits::MAX_UNKNOWN_RESOURCE_RAM_MB => {
            Err("Ressources indisponibles pour ce modèle".into())
        }
        _ => Ok(()),
    }
}

const fn is_apple_silicon() -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
}

fn fit(required_mb: u64, available_mb: u64) -> ResourceFit {
    if required_mb == 0 {
        return ResourceFit::Comfortable;
    }
    let required_with_margin = required_mb
        .saturating_mul(100 + SAFETY_MARGIN_PERCENT)
        .div_ceil(100);
    if available_mb < required_with_margin {
        ResourceFit::Insufficient
    } else if available_mb >= required_mb.saturating_mul(2) {
        ResourceFit::Comfortable
    } else {
        ResourceFit::Constrained
    }
}

fn best_fit(gpu: Option<ResourceFit>, cpu: Option<ResourceFit>) -> ResourceFit {
    let rank = |value| match value {
        ResourceFit::Comfortable => 3,
        ResourceFit::Constrained => 2,
        ResourceFit::Unknown => 1,
        ResourceFit::Insufficient => 0,
        ResourceFit::Cloud => 4,
    };
    gpu.into_iter()
        .chain(cpu)
        .max_by_key(|value| rank(*value))
        .unwrap_or(ResourceFit::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_margin_blocks_tight_resources() {
        assert_eq!(fit(1_000, 1_199), ResourceFit::Insufficient);
        assert_eq!(fit(1_000, 1_200), ResourceFit::Constrained);
        assert_eq!(fit(1_000, 2_000), ResourceFit::Comfortable);
    }

    #[test]
    fn unknown_cpu_memory_does_not_reject_a_cpu_capable_model() {
        let model = super::super::catalog::find_model("chronos-bolt-tiny").unwrap();
        let profile = HardwareProfile {
            gpu_memory_kind: GpuMemoryKind::Unknown,
            vram_total_mb: None,
            vram_available_mb: None,
            ram_available_mb: None,
        };

        assert_eq!(resource_fit(model, profile), ResourceFit::Unknown);
    }

    #[test]
    fn unknown_resources_reject_large_models_at_execution_time() {
        let model = super::super::catalog::find_model("chronos-2").unwrap();
        let profile = HardwareProfile {
            gpu_memory_kind: GpuMemoryKind::Unknown,
            vram_total_mb: None,
            vram_available_mb: None,
            ram_available_mb: None,
        };

        assert!(validate_model_resources_with_profile(model, profile).is_err());
    }

    #[test]
    fn unknown_resources_keep_lightweight_models_available() {
        let model = super::super::catalog::find_model("chronos-bolt-tiny").unwrap();
        let profile = HardwareProfile {
            gpu_memory_kind: GpuMemoryKind::Unknown,
            vram_total_mb: None,
            vram_available_mb: None,
            ram_available_mb: None,
        };

        assert!(validate_model_resources_with_profile(model, profile).is_ok());
    }
}
