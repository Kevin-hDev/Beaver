export const MAX_ALLOWED_PATHS = 70;
export const MAX_PATH_CHARS = 4_096;

export interface DirectoryAccessDecision {
  allowed: boolean;
  allowed_paths: string[];
}

export function parseAllowedPaths(value: unknown): string[] {
  if (!Array.isArray(value)
    || value.length < 1
    || value.length > MAX_ALLOWED_PATHS
    || value.some((path) => typeof path !== "string"
      || path.length < 1
      || path.length > MAX_PATH_CHARS)) {
    throw new Error("Invalid access paths");
  }
  return value as string[];
}

export function parseDirectoryAccessDecision(value: unknown): DirectoryAccessDecision {
  if (!value || typeof value !== "object") {
    throw new Error("Invalid access decision");
  }
  const decision = value as Record<string, unknown>;
  if (typeof decision.allowed !== "boolean") {
    throw new Error("Invalid access decision");
  }
  return {
    allowed: decision.allowed,
    allowed_paths: parseAllowedPaths(decision.allowed_paths),
  };
}
