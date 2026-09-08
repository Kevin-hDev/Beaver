#[cfg(all(test, native_browser))]
mod browser_slot_tests;
#[cfg(test)]
mod build_policy_tests;
#[cfg(all(test, target_os = "macos"))]
mod bundle_layout_tests;
#[cfg(test)]
mod cef_cookie_gate_policy_tests;
#[cfg(all(test, native_browser))]
mod cef_diagnostics_tests;
#[cfg(test)]
mod cef_preflight_tests;
#[cfg(test)]
mod cookie_store_probe_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod ffi_guard_tests;
#[cfg(test)]
mod lifecycle_tests;
#[cfg(test)]
mod live_session_registry_tests;
#[cfg(test)]
mod local_site_candidates_tests;
#[cfg(test)]
mod local_site_policy_tests;
#[cfg(test)]
mod local_site_probe_tests;
#[cfg(test)]
mod local_site_scan_state_tests;
#[cfg(test)]
mod local_site_scan_throttle_tests;
#[cfg(test)]
mod native_paths_tests;
#[cfg(all(test, target_os = "macos"))]
mod native_pump_policy_tests;
#[cfg(test)]
mod navigation_target_tests;
#[cfg(test)]
mod process_role_tests;
#[cfg(all(test, target_os = "macos"))]
mod pump_gate_tests;
#[cfg(test)]
mod runtime_handle_tests;
#[cfg(test)]
mod runtime_revision_tests;
#[cfg(test)]
mod session_model_tests;
#[cfg(test)]
mod session_order_tests;
#[cfg(test)]
mod session_store_tests;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod settings_tests;
#[cfg(test)]
mod surface_bounds_tests;
#[cfg(test)]
mod url_policy_tests;
#[cfg(test)]
mod view_recency_tests;
#[cfg(test)]
mod view_state_tests;
#[cfg(test)]
mod windows_bundle_layout_tests;
#[cfg(all(test, target_os = "windows"))]
mod windows_surface_order_tests;

#[cfg(test)]
mod favicon_png_tests;
