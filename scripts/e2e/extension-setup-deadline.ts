import { TIMEOUTS } from "../../src/types/extension-contract.generated";

// WDIO wraps Mocha hooks and enforces the configured global timeout. Keep that
// outer guard above every bounded native operation performed during setup.
export const EXTENSION_HOST_SETUP_TIMEOUT_MS = TIMEOUTS.hostRequestTimeoutMs
  + TIMEOUTS.hostStopTimeoutMs;

export const EXTENSION_UI_SETUP_TIMEOUT_MS = EXTENSION_HOST_SETUP_TIMEOUT_MS
  + (2 * TIMEOUTS.uiActionTimeoutMs);
