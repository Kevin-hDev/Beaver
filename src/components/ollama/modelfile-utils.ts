export interface ModelParameter {
  key: string;
  value: string;
}

const MAX_EXTRACTED_PARAMETERS = 128;
const PARAMETER_LINE = /^\s*PARAMETER\s+(\S+)\s+(.+)$/i;
const DIRECTIVE_LINE = /^\s*(\S+)\s+(.+)$/;

type QuoteDelimiter = '"' | '"""';

export function extractParameters(modelfile: string): ModelParameter[] {
  if (!modelfile) return [];
  const result: ModelParameter[] = [];
  let multiline: QuoteDelimiter | null = null;

  for (const line of modelfile.split(/\r?\n/)) {
    if (multiline) {
      if (closesMultiline(line, multiline)) multiline = null;
      continue;
    }

    const parameter = PARAMETER_LINE.exec(line);
    if (parameter) {
      const rawValue = parameter[2].trim();
      const opening = openingDelimiter(rawValue);
      if (opening) {
        multiline = opening;
      } else if (result.length < MAX_EXTRACTED_PARAMETERS) {
        result.push({ key: parameter[1], value: decodeValue(rawValue) });
      }
      continue;
    }

    const value = directiveValue(line);
    multiline = value ? openingDelimiter(value) : null;
  }
  return result;
}

function directiveValue(line: string): string | null {
  const match = DIRECTIVE_LINE.exec(line);
  if (!match || match[1].startsWith("#")) return null;
  const rest = match[2].trim();
  if (match[1].toUpperCase() !== "MESSAGE") return rest;
  return /^\S+\s+(.+)$/.exec(rest)?.[1].trim() ?? null;
}

function openingDelimiter(value: string): QuoteDelimiter | null {
  if (value.startsWith('"""')) {
    return value.length >= 6 && value.endsWith('"""') ? null : '"""';
  }
  if (value.startsWith('"')) {
    return value.length >= 2 && value.endsWith('"') ? null : '"';
  }
  return null;
}

function closesMultiline(line: string, delimiter: QuoteDelimiter): boolean {
  return line.trimEnd().endsWith(delimiter);
}

function decodeValue(value: string): string {
  if (value.length >= 6 && value.startsWith('"""') && value.endsWith('"""')) {
    return value.slice(3, -3);
  }
  if (value.length >= 2 && value.startsWith('"') && value.endsWith('"')) {
    return value.slice(1, -1);
  }
  return value;
}
