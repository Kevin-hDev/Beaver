#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

const VRAM_TIER_HIGH_MB: u64 = 24_000;
const VRAM_TIER_MID_MB: u64 = 12_000;
const CTX_HIGH: u32 = 32768;
const CTX_MID: u32 = 16384;
const CTX_LOW: u32 = 8192;

#[allow(clippy::needless_return)] // pattern multi-cfg cross-plateforme
pub fn detect_vram_mb() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        macos::detect_total()
    }
    #[cfg(target_os = "linux")]
    {
        return linux::detect_total();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::detect_total();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    None
}

#[allow(clippy::needless_return)] // pattern multi-cfg cross-plateforme
pub fn detect_vram_used_mb() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        macos::detect_used()
    }
    #[cfg(target_os = "linux")]
    {
        return linux::detect_used();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::detect_used();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    None
}

pub fn compute_default_num_ctx() -> u32 {
    match detect_vram_mb() {
        Some(mb) if mb >= VRAM_TIER_HIGH_MB => CTX_HIGH,
        Some(mb) if mb >= VRAM_TIER_MID_MB => CTX_MID,
        _ => CTX_LOW,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_vram_returns_something() {
        let vram = detect_vram_mb();
        if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            assert!(
                vram.is_some(),
                "Apple Silicon devrait retourner la mémoire unifiée"
            );
        }
    }

    #[test]
    fn default_num_ctx_is_reasonable() {
        let ctx = compute_default_num_ctx();
        assert!((CTX_LOW..=CTX_HIGH).contains(&ctx));
    }
}
