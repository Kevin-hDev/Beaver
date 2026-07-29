export interface AdvancedSettingsState {
  autostart: boolean;
  start_hidden: boolean;
  show_tray: boolean;
  default_model: string;
  keep_alive: string;
  allowed_paths: string[];
  session_outputs_directory: string;
  hardware_accel: string;
  multi_model: boolean;
  show_gpu_status: boolean;
  compression_enabled: boolean;
  compression_threshold: number;
  response_language: string;
  link_preview_enabled: boolean;
  ollama_setup_skipped: boolean;
}

export const ADVANCED_SETTINGS_DEFAULTS: AdvancedSettingsState = {
  autostart: false,
  start_hidden: false,
  show_tray: true,
  default_model: "",
  keep_alive: "5m",
  allowed_paths: ["/"],
  session_outputs_directory: "",
  hardware_accel: "gpu",
  multi_model: false,
  show_gpu_status: false,
  compression_enabled: true,
  compression_threshold: 85,
  response_language: "",
  link_preview_enabled: true,
  ollama_setup_skipped: false,
};
