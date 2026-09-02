import {
  EXTENSION_BACKEND_ERROR_CODES,
  HOST_DIAGNOSTIC_CODES,
  HOST_LOAD_STAGES,
  LIMITS,
  RUNTIME_DIAGNOSTIC_CODES,
  type HostDiagnosticCode,
  type RuntimeDiagnosticCode,
} from "@/types/extension-contract.generated";
import type {
  ExtensionDiagnostic,
  ExtensionHostState,
  ExtensionHostStatus,
} from "@/types/extensions";
import { isExtensionIdentifier } from "./extension-records";

const HOST_STATES: readonly ExtensionHostState[] = [
  "stopped",
  "starting",
  "running",
  "error",
];
const MAX_VERSION_CHARS = 128;
const MAX_DIAGNOSTIC_FILE_CHARS = 128;
const MAX_DIAGNOSTIC_POSITION = 10_000_000;

export const EMPTY_EXTENSION_HOST: ExtensionHostStatus = {
  state: "stopped",
  jitiVersion: "",
  apiVersion: "1",
  activeExtensions: 0,
  diagnostics: [],
};

export function parseExtensionHostStatus(value: unknown): ExtensionHostStatus {
  const input = object(value);
  if (!Array.isArray(input.diagnostics)
    || input.diagnostics.length > LIMITS.maxExtensions) invalid();
  return {
    state: oneOf(input.state, HOST_STATES),
    nodeVersion: optionalText(input.nodeVersion, MAX_VERSION_CHARS),
    jitiVersion: text(input.jitiVersion, MAX_VERSION_CHARS, true),
    apiVersion: text(input.apiVersion, MAX_VERSION_CHARS),
    activeExtensions: integer(input.activeExtensions, LIMITS.maxExtensions),
    lastError: optionalOneOf(input.lastError, EXTENSION_BACKEND_ERROR_CODES),
    diagnostics: input.diagnostics.map(diagnostic),
  };
}

function diagnostic(value: unknown): ExtensionDiagnostic {
  const input = object(value);
  const extensionId = text(input.extensionId, LIMITS.maxIdentifierChars);
  const file = optionalText(input.file, MAX_DIAGNOSTIC_FILE_CHARS);
  if (!isExtensionIdentifier(extensionId) || file?.includes("/") || file?.includes("\\")) {
    invalid();
  }
  return {
    extensionId,
    stage: oneOf(input.stage, HOST_LOAD_STAGES),
    code: diagnosticCode(input.code),
    file,
    line: optionalInteger(input.line, MAX_DIAGNOSTIC_POSITION),
    column: optionalInteger(input.column, MAX_DIAGNOSTIC_POSITION),
  };
}

function diagnosticCode(value: unknown): HostDiagnosticCode | RuntimeDiagnosticCode {
  if (typeof value !== "string") invalid();
  if (HOST_DIAGNOSTIC_CODES.includes(value as HostDiagnosticCode)) {
    return value as HostDiagnosticCode;
  }
  if (RUNTIME_DIAGNOSTIC_CODES.includes(value as RuntimeDiagnosticCode)) {
    return value as RuntimeDiagnosticCode;
  }
  invalid();
}

function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) invalid();
  return value as Record<string, unknown>;
}

function text(value: unknown, maxChars: number, allowEmpty = false): string {
  if (typeof value !== "string" || value.length > maxChars * 2) invalid();
  const length = Array.from(value).length;
  if (length > maxChars || (!allowEmpty && length === 0)) invalid();
  return value;
}

function optionalText(value: unknown, maxChars: number): string | undefined {
  return value === null || value === undefined ? undefined : text(value, maxChars);
}

function oneOf<T extends string>(value: unknown, values: readonly T[]): T {
  if (typeof value !== "string" || !values.includes(value as T)) invalid();
  return value as T;
}

function optionalOneOf<T extends string>(value: unknown, values: readonly T[]): T | undefined {
  if (value === null || value === undefined) return undefined;
  return oneOf(value, values);
}

function integer(value: unknown, max: number): number {
  if (!Number.isInteger(value) || (value as number) < 0 || (value as number) > max) invalid();
  return value as number;
}

function optionalInteger(value: unknown, max: number): number | undefined {
  return value === null || value === undefined ? undefined : integer(value, max);
}

function invalid(): never {
  throw new Error("invalid_extension_host_response");
}
