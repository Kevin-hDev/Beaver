import { OFFICE_LIMITS } from "./constants.mjs";
import { structuredResultBytes } from "./result-budget.mjs";

const SAFE_CODES = new Set([
  "file_not_found",
  "file_too_large",
  "invalid_input",
  "invalid_path",
  "operation_failed",
  "output_too_large",
  "unsupported_format",
  "unsafe_archive",
  "unsupported_character",
]);

export class OfficePluginError extends Error {
  constructor(code) {
    super(SAFE_CODES.has(code) ? code : "operation_failed");
    this.name = "OfficePluginError";
    this.code = SAFE_CODES.has(code) ? code : "operation_failed";
  }
}

export function rejectOffice(code) {
  throw new OfficePluginError(code);
}

export function safeTool(handler) {
  return async (arguments_, context) => {
    try {
      return await handler(arguments_, context);
    } catch (error) {
      const code = error instanceof OfficePluginError
        ? error.code
        : "operation_failed";
      return { content: code, isError: true };
    }
  };
}

export function success(payload, displaySummary) {
  if (structuredResultBytes(payload) > OFFICE_LIMITS.maxStructuredResultBytes) {
    rejectOffice("output_too_large");
  }
  return {
    content: JSON.stringify(payload),
    displaySummary,
  };
}
