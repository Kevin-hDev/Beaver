const LANGUAGE_BY_EXT: Record<string, string> = {
  c: "c",
  cpp: "cpp",
  css: "css",
  go: "go",
  h: "c",
  html: "html",
  java: "java",
  js: "javascript",
  jsx: "jsx",
  json: "json",
  lua: "lua",
  md: "markdown",
  py: "python",
  rs: "rust",
  sh: "bash",
  sql: "sql",
  toml: "toml",
  ts: "typescript",
  tsx: "tsx",
  yaml: "yaml",
  yml: "yaml",
};

export function languageFromPath(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  return LANGUAGE_BY_EXT[ext] ?? "text";
}

const WRAP_EXTENSIONS = new Set(["md", "txt", "json", "yaml", "yml", "toml", "csv", "log"]);

export function shouldWrapFile(path: string): boolean {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  return WRAP_EXTENSIONS.has(ext);
}
