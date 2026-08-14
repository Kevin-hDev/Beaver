interface CaptureOptions {
  logDirectory: string;
  root: string;
  timeoutMs: number;
  observeTurnover?: (options: {
    platform: string;
    root: string;
    timeoutMs: number;
  }) => Promise<{ exitedPid: number; initialPids: number[] }>;
}

interface WaitOptions {
  logDirectory: string;
  timeoutMs: number;
  pollMs?: number;
}

export function captureMacCefTurnoverProof(options: CaptureOptions): Promise<void>;
export function waitForMacCefTurnoverProof(options: WaitOptions): Promise<{
  exitedPid: number;
  initialPids: number[];
  observedAt: string;
}>;
