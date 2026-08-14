import type { NativeProcessRecord } from "./native-cef-observer.mjs";

interface NativeCefLivenessOptions {
  platform?: string;
  root: string;
  timeoutMs?: number;
  pollMs?: number;
  listProcesses?: (platform: string) => NativeProcessRecord[] | Promise<NativeProcessRecord[]>;
}

export const CEF_HELPER_TURNOVER_POLL_MS: number;
export function waitForOwnedCefHelperSet(options: NativeCefLivenessOptions): Promise<number[]>;
export function waitForOwnedCefHelperTurnover(
  options: NativeCefLivenessOptions & { initialPids: number[] },
): Promise<{ exitedPid: number; initialPids: number[] }>;
