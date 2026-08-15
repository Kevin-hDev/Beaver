import { readFile, readdir, stat } from "node:fs/promises";
import { join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_ROOTS = ["src", "src-tauri/src"];
const SOURCE_EXTENSIONS = new Set([".cjs", ".js", ".mjs", ".rs", ".ts", ".tsx"]);
const IGNORED_SEGMENTS = new Set([".git", "graphify-out", "node_modules", "target"]);
const IGNORED_FILE_SUFFIXES = [
  ".spec.ts",
  ".test.js",
  ".test.mjs",
  ".test.ts",
  "_tests.rs",
];
const ALLOWED_RELATIVE_PATHS = new Set([
  "src-tauri/src/services/ollama_manager.rs",
  "src-tauri/src/services/paths/ollama.rs",
]);
const ALLOWED_DIRECTORY = "src-tauri/src/services/ollama_manager";
const MAX_FILES = 4096;
const MAX_FILE_BYTES = 512 * 1024;
const MAX_VIOLATIONS = 128;

const RULES = [
  {
    rule: "transactional-path",
    pattern: /["'`]ollama-(?:bundle(?:-(?:staging|old|failed|install-staging(?:-archives(?:-failed)?)?|update-staging|backup(?:-delete)?|failed-delete|receipt(?:\.tmp)?))?|update-state(?:\.(?:json|tmp))?|layout-migration(?:\.(?:json|tmp))?|process-receipt)(?=["'`),/\\])/u,
  },
  {
    rule: "binary-spawn",
    pattern: /(?:Command|windows_spawn::Command)::new\s*\([^\n)]*ollama/iu,
  },
  {
    rule: "direct-runtime-control",
    pattern: /\b(?:OLLAMA_(?:INSTALL_LOCK|SETUP_CANCEL|BASE_URL|PORT|ENDPOINT)|ollama_(?:lifecycle|kill|port|env|ps)|OllamaSidecar)\b/u,
  },
  {
    rule: "direct-port-selection",
    pattern: /\b(?:OllamaEndpoint::loopback|DefaultOllamaPortAllocator::new|allocate_loopback)\b/u,
  },
  {
    rule: "global-endpoint",
    pattern: /\b(?:static|const)\s+[A-Z0-9_]*(?:OLLAMA_(?:PORT|BASE_URL|ENDPOINT)|OLLAMA_ENDPOINT)[A-Z0-9_]*\b|\b(?:static|const)\s+[A-Z0-9_]*:\s*OnceLock\s*<[^>]*Ollama(?:Endpoint|Port|Client)/u,
  },
  {
    rule: "duplicate-retry-calendar",
    pattern: /\[\s*2\s*,\s*4\s*,\s*8\s*,\s*16\s*\]|\[\s*5\s*,\s*15\s*,\s*60\s*,\s*300\s*\]/u,
  },
  {
    rule: "removed-module",
    pattern: /\b(?:mod|use|pub\s+mod)\s+(?:ollama_(?:lifecycle|kill|port|env|ps|polling)|ollama_(?:bundle_utils|download|checksum|extract|setup_install|setup_start|setup_cancel))\b/u,
  },
];

function fail(message) {
  throw new Error(`Ollama manager contract scan failed: ${message}`);
}

function extensionOf(path) {
  const dot = path.lastIndexOf(".");
  return dot === -1 ? "" : path.slice(dot).toLowerCase();
}

function isIgnoredFile(path) {
  return IGNORED_FILE_SUFFIXES.some((suffix) => path.endsWith(suffix));
}

function isAllowed(path) {
  return ALLOWED_RELATIVE_PATHS.has(path) || path.startsWith(`${ALLOWED_DIRECTORY}/`);
}

async function collectFiles(root, directory, files) {
  const entries = await readdir(directory, { withFileTypes: true });
  for (const entry of entries) {
    if (IGNORED_SEGMENTS.has(entry.name)) continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      await collectFiles(root, path, files);
      continue;
    }
    if (!entry.isFile() || !SOURCE_EXTENSIONS.has(extensionOf(entry.name))) continue;
    const relativePath = relative(root, path).split(sep).join("/");
    if (relativePath.startsWith("scripts/ollama/fixtures/")) continue;
    if (isIgnoredFile(relativePath)) continue;
    files.push({ path, relativePath });
    if (files.length > MAX_FILES) fail("source file limit exceeded");
  }
}

function lineNumber(source, index) {
  return source.slice(0, index).split(/\r?\n/u).length;
}

function violationsFor(source, relativePath) {
  if (isAllowed(relativePath)) return [];
  const violations = [];
  for (const { rule, pattern } of RULES) {
    const match = pattern.exec(source);
    if (!match) continue;
    violations.push({
      path: relativePath,
      line: lineNumber(source, match.index),
      rule,
    });
  }
  return violations;
}

export async function verifyOllamaManagerContracts({ repoRoot }) {
  if (typeof repoRoot !== "string" || repoRoot.length === 0) fail("invalid repository root");
  const files = [];
  for (const sourceRoot of SOURCE_ROOTS) {
    const directory = join(repoRoot, sourceRoot);
    try {
      const information = await stat(directory);
      if (!information.isDirectory()) continue;
    } catch (error) {
      if (error?.code === "ENOENT") continue;
      throw error;
    }
    await collectFiles(repoRoot, directory, files);
  }
  if (files.length === 0) fail("no source files found");

  const violations = [];
  for (const file of files) {
    const source = await readFile(file.path, "utf8");
    if (Buffer.byteLength(source, "utf8") > MAX_FILE_BYTES) {
      fail(`file too large: ${file.relativePath}`);
    }
    violations.push(...violationsFor(source, file.relativePath));
    if (violations.length > MAX_VIOLATIONS) fail("violation limit exceeded");
  }
  return { scannedFiles: files.length, violations };
}

export function formatViolations(violations) {
  return violations
    .map(({ path, line, rule }) => `${path}:${line}: ${rule}`)
    .join("\n");
}

function isDirectExecution() {
  return process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1];
}

if (isDirectExecution()) {
  try {
    const result = await verifyOllamaManagerContracts({ repoRoot: process.cwd() });
    if (result.violations.length > 0) {
      process.stderr.write(`${formatViolations(result.violations)}\n`);
      process.exitCode = 1;
    } else {
      process.stdout.write(`Verified ${result.scannedFiles} source file(s).\n`);
    }
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : "contract scan failed"}\n`);
    process.exitCode = 1;
  }
}
