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

export function extractSystemPrompt(modelfile: string): string {
  if (!modelfile) return "";

  const tripleMatch = modelfile.match(/^SYSTEM\s+"""([\s\S]*?)"""/m);
  if (tripleMatch) return tripleMatch[1].trim();

  const singleMatch = modelfile.match(/^SYSTEM\s+"([^"\n]*)"/m);
  if (singleMatch) return singleMatch[1];

  const bareMatch = modelfile.match(/^SYSTEM\s+(.+)$/m);
  if (bareMatch) return bareMatch[1].trim();

  return "";
}
