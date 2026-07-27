import { OFFICE_LIMITS } from "./constants.mjs";
import { structuredResultBytes } from "./result-budget.mjs";

const SAFE_CODES = new Set([
  "file_not_found",
  "file_too_large",
  "invalid_input",
  "invalid_path",
  "operation_failed",
  "output_too_large",
  "too_many_requests",
  "unsupported_format",
  "unsafe_archive",
  "unsupported_character",
]);

export class OfficePluginError extends Error {
  constructor(code, details) {
    super(SAFE_CODES.has(code) ? code : "operation_failed");
    this.name = "OfficePluginError";
    this.code = SAFE_CODES.has(code) ? code : "operation_failed";
    this.details = safeDetails(this.code, details);
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
      const pluginError = error instanceof OfficePluginError
        ? error
        : new OfficePluginError("operation_failed");
      return { content: errorContent(pluginError), isError: true };
    }
  };
}

function safeDetails(code, details) {
  if (code !== "unsupported_character" || !details) return undefined;
  const codePoint = details.codePoint;
  const paragraph = details.paragraph;
  const title = details.title === true;
  if (
    !Number.isInteger(codePoint)
    || codePoint < 0
    || codePoint > 0x10ffff
    || (!title && (
      !Number.isInteger(paragraph)
      || paragraph < 1
      || paragraph > OFFICE_LIMITS.maxBlocks
    ))
  ) {
    return undefined;
  }
  return Object.freeze({ codePoint, paragraph, title });
}

function errorContent(error) {
  if (error.code !== "unsupported_character" || !error.details) {
    return error.code;
  }
  const point = `U+${error.details.codePoint.toString(16).toUpperCase().padStart(4, "0")}`;
  return error.details.title
    ? `unsupported_character ${point} title`
    : `unsupported_character ${point} paragraph ${error.details.paragraph}`;
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
