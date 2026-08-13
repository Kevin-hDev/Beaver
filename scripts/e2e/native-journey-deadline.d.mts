export type NativeJourneyStage =
  | "page_server_start"
  | "onboarding"
  | "native_webviews"
  | "browser_capability"
  | "browser_session_open"
  | "browser_surface"
  | "cef_helper_start"
  | "page_load"
  | "exit_request"
  | "webdriver_release"
  | "owned_process_exit"
  | "native_webview_exit";

export interface NativeJourneyOperation {
  signal: AbortSignal;
  timeoutMs: number;
}

export interface NativeJourney {
  run<T>(
    stage: NativeJourneyStage,
    operation: (context: NativeJourneyOperation) => Promise<T> | T,
  ): Promise<T>;
  cleanup<T>(
    stage: string,
    operation: (context: NativeJourneyOperation) => Promise<T> | T,
  ): Promise<T>;
}

export const NATIVE_JOURNEY_TIMEOUT_MS: number;
export const NATIVE_JOURNEY_CLEANUP_TIMEOUT_MS: number;
export const NATIVE_JOURNEY_MOCHA_TIMEOUT_MS: number;
export const NATIVE_CEF_STAGE_CEILINGS_MS: Readonly<Record<NativeJourneyStage, number>>;
export function createNativeJourney(): NativeJourney;
