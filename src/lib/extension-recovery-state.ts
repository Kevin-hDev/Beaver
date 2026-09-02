import {
  HOST_LOAD_STAGES,
  type ExtensionRecoveryState,
  type HostLoadStage,
} from "@/types/extensions";
import { isExtensionIdentifier } from "@/lib/extension-records";

export function parseExtensionRecoveryState(value: unknown): ExtensionRecoveryState {
  if (!value || typeof value !== "object" || Array.isArray(value)) throwInvalid();
  const input = value as Record<string, unknown>;
  const extensionId = optionalText(input.extensionId, 96);
  const stage = optionalStage(input.stage);
  const attempts = input.attempts === null
    ? null
    : boundedInteger(input.attempts, 255);
  if (extensionId && !isExtensionIdentifier(extensionId)) throwInvalid();
  for (const key of ["canRetry", "markerInvalid", "recoverySnapshotAvailable"]) {
    if (typeof input[key] !== "boolean") throwInvalid();
  }
  return {
    extensionId,
    stage,
    attempts,
    canRetry: input.canRetry as boolean,
    markerInvalid: input.markerInvalid as boolean,
    recoverySnapshotAvailable: input.recoverySnapshotAvailable as boolean,
  };
}

function optionalStage(value: unknown): HostLoadStage | null {
  if (value === null) return null;
  if (typeof value !== "string" || !HOST_LOAD_STAGES.includes(value as HostLoadStage)) {
    throwInvalid();
  }
  return value as HostLoadStage;
}

function optionalText(value: unknown, max: number): string | null {
  if (value === null) return null;
  if (typeof value !== "string" || value.length === 0 || value.length > max) throwInvalid();
  return value;
}

function boundedInteger(value: unknown, max: number): number {
  if (!Number.isInteger(value) || (value as number) < 0 || (value as number) > max) throwInvalid();
  return value as number;
}

function throwInvalid(): never {
  throw new Error("invalid_extension_recovery_response");
}
