import { TIMEOUTS } from "../../src/types/extension-contract.generated";

// WDIO wraps Mocha hooks and enforces the configured global timeout. Keep that
// outer guard above every bounded native operation performed during setup.
export const EXTENSION_HOST_SETUP_TIMEOUT_MS = TIMEOUTS.hostRequestTimeoutMs
  + TIMEOUTS.hostStopTimeoutMs;

export const EXTENSION_UI_SETUP_TIMEOUT_MS = EXTENSION_HOST_SETUP_TIMEOUT_MS
  + (2 * TIMEOUTS.uiActionTimeoutMs);

// Diagnostic only: initialization, refresh, installation and activation are measured separately.
export const EXTENSION_DIAGNOSTIC_TIMEOUT_MS = EXTENSION_UI_SETUP_TIMEOUT_MS * 4;

export const WEBDRIVER_IMPLICIT_TIMEOUT_MS = 0;
export const WEBDRIVER_PAGE_LOAD_TIMEOUT_MS = 300_000;
