// Generated from services/automations/types.rs. Do not edit manually.
export const AUTOMATION_ERROR_CODES = ["not_found","capacity_reached","invalid_input","immutable_field","invalid_schedule","provider_unavailable","model_unavailable","model_tools_unsupported","audit_unavailable","store_unavailable","cursor_expired","revision_conflict","globally_paused","consent_required","migration_unavailable","invalid_timezone","invalid_project"] as const;
export type AutomationErrorCode = typeof AUTOMATION_ERROR_CODES[number];
