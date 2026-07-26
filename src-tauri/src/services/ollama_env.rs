use crate::models::config::AdvancedSettings;
use crate::services::gpu_detect::GpuVendor;

const GPU_OVERHEAD_BYTES: &str = "1073741824";
const LOAD_TIMEOUT: &str = "10m";

pub fn build_env_vars(
    config: &AdvancedSettings,
    gpu: &GpuVendor,
    port: u16,
) -> Vec<(String, String)> {
    let mut vars = Vec::with_capacity(13);

    vars.push(("OLLAMA_HOST".into(), format!("127.0.0.1:{port}")));
    vars.push(("OLLAMA_FLASH_ATTENTION".into(), "1".into()));
    vars.push(("OLLAMA_KV_CACHE_TYPE".into(), "q8_0".into()));
    vars.push(("OLLAMA_NUM_PARALLEL".into(), "1".into()));
    vars.push(("OLLAMA_NO_CLOUD".into(), "1".into()));
    vars.push(("OLLAMA_LOAD_TIMEOUT".into(), LOAD_TIMEOUT.into()));

    let num_ctx = crate::services::gpu_detect::compute_default_num_ctx();
    vars.push(("OLLAMA_CONTEXT_LENGTH".into(), num_ctx.to_string()));

    let max_models = if config.multi_model { "0" } else { "1" };
    vars.push(("OLLAMA_MAX_LOADED_MODELS".into(), max_models.into()));

    let keep_alive = if config.keep_alive == "forever" {
        "-1".to_string()
    } else {
        config.keep_alive.clone()
    };
    vars.push(("OLLAMA_KEEP_ALIVE".into(), keep_alive));

    if config.hardware_accel == "cpu" {
        vars.push(("OLLAMA_LLM_LIBRARY".into(), "cpu".into()));
    } else {
        if !matches!(gpu, GpuVendor::Unknown) {
            vars.push(("OLLAMA_GPU_OVERHEAD".into(), GPU_OVERHEAD_BYTES.into()));
        }

        #[cfg(target_os = "windows")]
        {
            vars.push(("OLLAMA_VULKAN".into(), "1".into()));
        }
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> AdvancedSettings {
        AdvancedSettings::default()
    }

    fn find_var<'a>(vars: &'a [(String, String)], key: &str) -> Option<&'a str> {
        vars.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    #[test]
    fn always_sets_flash_attention() {
        let vars = build_env_vars(&default_config(), &GpuVendor::Nvidia, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_FLASH_ATTENTION"), Some("1"));
    }

    #[test]
    fn always_sets_kv_cache_q8() {
        let vars = build_env_vars(&default_config(), &GpuVendor::Nvidia, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_KV_CACHE_TYPE"), Some("q8_0"));
    }

    #[test]
    fn sets_host_with_port() {
        let vars = build_env_vars(&default_config(), &GpuVendor::Unknown, 11555);
        assert_eq!(find_var(&vars, "OLLAMA_HOST"), Some("127.0.0.1:11555"));
    }

    #[test]
    fn multi_model_off_sets_max_1() {
        let config = default_config();
        let vars = build_env_vars(&config, &GpuVendor::Unknown, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_MAX_LOADED_MODELS"), Some("1"));
    }

    #[test]
    fn multi_model_on_sets_max_0() {
        let mut config = default_config();
        config.multi_model = true;
        let vars = build_env_vars(&config, &GpuVendor::Unknown, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_MAX_LOADED_MODELS"), Some("0"));
    }

    #[test]
    fn cpu_mode_sets_llm_library() {
        let mut config = default_config();
        config.hardware_accel = "cpu".into();
        let vars = build_env_vars(&config, &GpuVendor::Nvidia, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_LLM_LIBRARY"), Some("cpu"));
    }

    #[test]
    fn gpu_mode_no_llm_library() {
        let vars = build_env_vars(&default_config(), &GpuVendor::Nvidia, 11500);
        assert!(find_var(&vars, "OLLAMA_LLM_LIBRARY").is_none());
    }

    #[test]
    fn cpu_mode_no_gpu_overhead() {
        let mut config = default_config();
        config.hardware_accel = "cpu".into();
        let vars = build_env_vars(&config, &GpuVendor::Nvidia, 11500);
        assert!(find_var(&vars, "OLLAMA_GPU_OVERHEAD").is_none());
    }

    #[test]
    fn gpu_overhead_set_when_gpu_detected() {
        let vars = build_env_vars(&default_config(), &GpuVendor::Nvidia, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_GPU_OVERHEAD"), Some("1073741824"));
    }

    #[test]
    fn no_gpu_overhead_when_unknown() {
        let vars = build_env_vars(&default_config(), &GpuVendor::Unknown, 11500);
        assert!(find_var(&vars, "OLLAMA_GPU_OVERHEAD").is_none());
    }

    #[test]
    fn keep_alive_forever_maps_to_minus_1() {
        let mut config = default_config();
        config.keep_alive = "forever".into();
        let vars = build_env_vars(&config, &GpuVendor::Unknown, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_KEEP_ALIVE"), Some("-1"));
    }

    #[test]
    fn no_cloud_always_set() {
        let vars = build_env_vars(&default_config(), &GpuVendor::Unknown, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_NO_CLOUD"), Some("1"));
    }

    #[test]
    fn model_load_timeout_allows_slow_hardware() {
        let vars = build_env_vars(&default_config(), &GpuVendor::Unknown, 11500);
        assert_eq!(find_var(&vars, "OLLAMA_LOAD_TIMEOUT"), Some("10m"));
    }
}
