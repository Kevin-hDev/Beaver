export interface ModelParameter {
  key: string;
  value: string;
}

const MAX_EXTRACTED_PARAMETERS = 128;
const PARAMETER_LINE = /^\s*PARAMETER\s+(\S+)\s+(.+)$/i;

export function extractParameters(modelfile: string): ModelParameter[] {
  if (!modelfile) return [];
  const result: ModelParameter[] = [];
  let inMultiline = false;

  for (const line of modelfile.split(/\r?\n/)) {
    if (!inMultiline && result.length < MAX_EXTRACTED_PARAMETERS) {
      const match = PARAMETER_LINE.exec(line);
      if (match) result.push({ key: match[1], value: match[2].trim() });
    }
    if (tripleQuoteCount(line) % 2 === 1) inMultiline = !inMultiline;
  }
  return result;
}

function tripleQuoteCount(line: string): number {
  return line.match(/"""/g)?.length ?? 0;
}
