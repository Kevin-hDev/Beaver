export interface ModelParameter {
  key: string;
  value: string;
}

const MAX_EXTRACTED_PARAMETERS = 128;

export function extractParameters(modelfile: string): ModelParameter[] {
  if (!modelfile) return [];
  const result: ModelParameter[] = [];
  const re = /^PARAMETER\s+(\S+)\s+(.+)$/gim;
  let m: RegExpExecArray | null;
  while (result.length < MAX_EXTRACTED_PARAMETERS && (m = re.exec(modelfile)) !== null) {
    result.push({ key: m[1], value: m[2].trim() });
  }
  return result;
}

export function hasSystemPrompt(modelfile: string): boolean {
  return /^[ \t]*SYSTEM(?:\s|$)/im.test(modelfile);
}
