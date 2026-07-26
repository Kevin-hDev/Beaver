const CONTEXT_SIZE = 3;

export interface DiffContext {
  before: string[];
  after: string[];
  beforeStartLine: number;
  afterStartLine: number;
}

export function extractDiffContext(
  fileContent: string,
  startLine: number,
  newLineCount: number,
): DiffContext {
  const allLines = fileContent.split("\n");
  const total = allLines.length;

  const bStart = Math.max(0, startLine - 1 - CONTEXT_SIZE);
  const bEnd = Math.max(0, startLine - 1);

  const editEnd = startLine - 1 + newLineCount;
  const aEnd = Math.min(total, editEnd + CONTEXT_SIZE);

  return {
    before: allLines.slice(bStart, bEnd),
    after: allLines.slice(editEnd, aEnd),
    beforeStartLine: bStart + 1,
    afterStartLine: editEnd + 1,
  };
}
