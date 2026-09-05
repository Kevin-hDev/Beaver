export const DIAGNOSTIC_POLL_MS: number;
export const MAX_DIAGNOSTIC_SAMPLES: number;
export function diagnosticProfile(value: unknown): Promise<string>;
export function sampleMarker(profile: string): Promise<{
  state: string;
  extensionId?: string;
  stage?: string;
}>;
export function safeOutcome(error: unknown): string;
export function verifiedScriptTimeout(driver: {
  getTimeouts(): Promise<{script?: number | null}>;
  setTimeout(value: {script: number}): Promise<unknown>;
}, requested: number): Promise<{previous?: number | null; effective: number}>;
