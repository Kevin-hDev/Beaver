import { basename } from "node:path";

const MAX_POSITION = 10_000_000;
const KNOWN_NODE_CODES = new Set([
  "ERR_MODULE_NOT_FOUND",
  "ERR_UNKNOWN_FILE_EXTENSION",
  "MODULE_NOT_FOUND",
]);

export function createDiagnostic(error, stage, mainPath) {
  const file = basename(String(mainPath ?? "")).slice(0, 128);
  const position = sourcePosition(error, file);
  return {
    stage,
    code: diagnosticCode(error, stage),
    file: file || undefined,
    line: position?.line,
    column: position?.column,
  };
}

function diagnosticCode(error, stage) {
  const nodeCode = typeof error?.code === "string" ? error.code : "";
  if (KNOWN_NODE_CODES.has(nodeCode)) return "module_not_found";
  const messagePrefix =
    typeof error?.message === "string" ? error.message.slice(0, 64) : "";
  if (error?.name === "SyntaxError" || messagePrefix.startsWith("ParseError:")) {
    return "syntax_error";
  }
  if (stage === "activate") return "activation_failed";
  if (stage === "register") return "registration_failed";
  return "import_failed";
}

function sourcePosition(error, file) {
  if (!file || typeof error?.stack !== "string") return null;
  const escaped = file.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = error.stack.match(new RegExp(`${escaped}:(\\d+):(\\d+)`));
  if (!match) return null;
  const line = Number(match[1]);
  const column = Number(match[2]);
  if (
    !Number.isSafeInteger(line)
    || !Number.isSafeInteger(column)
    || line < 1
    || column < 1
    || line > MAX_POSITION
    || column > MAX_POSITION
  ) {
    return null;
  }
  return { line, column };
}
