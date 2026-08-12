export interface NativeProcessRecord {
  pid: number;
  parentPid: number;
  executable: string;
  command: string;
}

export function runtimeRootForBinary(platform: string, binaryPath: string): string;
export function waitForOwnedCefHelper(options: {
  platform?: string;
  root: string;
  timeoutMs?: number;
  pollMs?: number;
  listProcesses?: (platform: string) => NativeProcessRecord[] | Promise<NativeProcessRecord[]>;
}): Promise<void>;
export function waitForOwnedProcessesToExit(options: {
  platform?: string;
  root: string;
  timeoutMs?: number;
  pollMs?: number;
  listProcesses?: (platform: string) => NativeProcessRecord[] | Promise<NativeProcessRecord[]>;
}): Promise<void>;
export function waitForProcessIdsToExit(options: {
  platform?: string;
  pids: number[];
  timeoutMs?: number;
  pollMs?: number;
  listProcesses?: (platform: string) => NativeProcessRecord[] | Promise<NativeProcessRecord[]>;
}): Promise<void>;
