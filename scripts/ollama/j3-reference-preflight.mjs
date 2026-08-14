import { execFile } from "node:child_process";
import { readFile, stat } from "node:fs/promises";
import { isAbsolute, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const MAX_COLLECTION_SIZE = 6;
const MAX_MARKDOWN_BYTES = 256 * 1024;
const MAX_OUTPUT_BYTES = 64 * 1024;
const MAX_PATH_LENGTH = 4096;
const MAX_REF_LENGTH = 512;
const MAX_TIMEOUT_MS = 30_000;
const SHA_PATTERN = /(?<![0-9A-Fa-f])[0-9a-f]{40}(?![0-9A-Fa-f])/gu;
const HEAD_REF_PATTERN = /refs\/heads\/[A-Za-z0-9][A-Za-z0-9._/-]{0,255}/gu;
const NOTES_REF_PATTERN = /refs\/notes\/[A-Za-z0-9][A-Za-z0-9._/-]{0,255}/gu;
const FAILURE = "J3 reference preflight failed";

function fail() {
  throw new Error(FAILURE);
}
function isSafeText(value, maximumLength, allowLineBreaks = false) {
  return typeof value === "string"
    && value.length > 0
    && value.length <= maximumLength
    && !/\0/u.test(value)
    && (allowLineBreaks || !/[\r\n]/u.test(value));
}
function assertSafePath(value) {
  if (!isSafeText(value, MAX_PATH_LENGTH)) fail();
  if (value.split(/[\\/]+/u).includes("..")) fail();
}
function collectMatches(value, pattern) {
  const matches = [];
  for (const match of value.matchAll(pattern)) {
    matches.push(match[0]);
    if (matches.length > MAX_COLLECTION_SIZE) fail();
  }
  return matches;
}
function assertSingle(matches) {
  if (matches.length !== 1) fail();
  return matches[0];
}
function validateRef(value, prefix) {
  if (!isSafeText(value, MAX_REF_LENGTH) || !value.startsWith(prefix)) fail();
  const path = value.slice(prefix.length);
  const segments = path.split("/");
  if (segments.length === 0 || segments.some((segment) => (
    segment.length === 0 || segment === "." || segment === ".."
  ))) fail();
  return value;
}
function parseArchiveHead(lines) {
  const markedLines = lines.filter((line) => /(?:tête\s+immuable|archive\s+head)/iu.test(line));
  if (markedLines.length !== 1 || markedLines.length > MAX_COLLECTION_SIZE) fail();
  const marker = /(?:tête\s+immuable|archive\s+head)/iu.exec(markedLines[0]);
  const valueWindow = markedLines[0].slice((marker?.index ?? 0) + (marker?.[0].length ?? 0));
  const valueCell = /`([^`]+)`/u.exec(valueWindow)?.[1] ?? valueWindow.split(/[.;]/u, 1)[0];
  return assertSingle(collectMatches(valueCell, SHA_PATTERN));
}
function splitMarkdownCells(line) {
  const trimmed = line.trim();
  if (!trimmed.startsWith("|")) return [];
  const content = trimmed.replace(/^\|/u, "").replace(/\|$/u, "");
  const cells = content.split("|").map((cell) => cell.trim());
  if (cells.length > 32) fail();
  return cells;
}
function parseCommits(lines) {
  const headers = lines.filter((line) => /SHA\s+complet\s+J3/iu.test(line));
  if (headers.length !== 1 || headers.length > MAX_COLLECTION_SIZE) fail();
  const headerIndex = lines.indexOf(headers[0]);
  const shaColumn = splitMarkdownCells(headers[0]).findIndex((cell) => /SHA\s+complet\s+J3/iu.test(cell));
  if (shaColumn < 0) fail();
  const commits = [];
  let tableStarted = false;
  for (let index = headerIndex + 1; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.trim().length === 0) {
      if (tableStarted) break;
      continue;
    }
    if (!line.trimStart().startsWith("|")) {
      if (tableStarted) break;
      continue;
    }
    tableStarted = true;
    const cells = splitMarkdownCells(line);
    if (cells.length <= shaColumn) fail();
    const shaCell = cells[shaColumn];
    if (/^:?-{3,}:?$/u.test(shaCell)) continue;
    const rowCommits = collectMatches(shaCell, SHA_PATTERN);
    if (rowCommits.length !== 1) fail();
    commits.push(rowCommits[0]);
    if (commits.length > MAX_COLLECTION_SIZE) fail();
  }
  if (commits.length !== MAX_COLLECTION_SIZE || new Set(commits).size !== commits.length) fail();
  return commits;
}
export function parseJ3ReferenceInventory(markdown) {
  if (!isSafeText(markdown, MAX_MARKDOWN_BYTES, true)
    || Buffer.byteLength(markdown, "utf8") > MAX_MARKDOWN_BYTES) fail();
  const lines = markdown.split(/\r?\n/u);
  if (lines.length > 4096) fail();
  const archiveRef = validateRef(
    assertSingle(collectMatches(markdown, HEAD_REF_PATTERN)),
    "refs/heads/",
  );
  const notesRef = validateRef(
    assertSingle(collectMatches(markdown, NOTES_REF_PATTERN)),
    "refs/notes/",
  );
  const archiveHead = parseArchiveHead(lines);
  const commits = parseCommits(lines);
  return { archiveRef, archiveHead, notesRef, commits };
}
function validateGitArgs(args) {
  if (!Array.isArray(args) || args.length === 0 || args.length > 16) fail();
  if (!args.every((argument) => isSafeText(argument, MAX_PATH_LENGTH))) fail();
}
function normalizeGitOutput(result) {
  const output = typeof result === "string" ? result : result?.stdout;
  if (typeof output !== "string" || Buffer.byteLength(output, "utf8") > MAX_OUTPUT_BYTES) fail();
  if (result && typeof result === "object" && result.status !== undefined && result.status !== 0) fail();
  return output;
}
async function callGit(runGit, args) {
  validateGitArgs(args);
  try {
    return normalizeGitOutput(await runGit([...args]));
  } catch {
    fail();
  }
}
function remoteArchiveRef(archiveRef) {
  const branch = archiveRef.slice("refs/heads/".length);
  return validateRef(`refs/remotes/origin/${branch}`, "refs/remotes/");
}
async function readInventory(inventoryPath, repoRoot, statFile = stat, readInventoryFile = readFile) {
  assertSafePath(repoRoot);
  assertSafePath(inventoryPath);
  if (typeof statFile !== "function" || typeof readInventoryFile !== "function") fail();
  const path = isAbsolute(inventoryPath) ? inventoryPath : resolve(repoRoot, inventoryPath);
  try {
    const metadata = await statFile(path);
    if (!metadata || !Number.isSafeInteger(metadata.size) || metadata.size < 0 || metadata.size > MAX_MARKDOWN_BYTES) fail();
    const markdown = await readInventoryFile(path, "utf8");
    if (Buffer.byteLength(markdown, "utf8") > MAX_MARKDOWN_BYTES) fail();
    return markdown;
  } catch {
    fail();
  }
}
function createGitRunner(repoRoot) {
  return async (args) => {
    validateGitArgs(args);
    const result = await execFileAsync("git", args, {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: MAX_OUTPUT_BYTES,
      shell: false,
      timeout: MAX_TIMEOUT_MS,
      windowsHide: true,
    });
    return result.stdout;
  };
}
export async function verifyJ3ReferenceArchive({
  repoRoot,
  inventoryPath,
  runGit = createGitRunner(repoRoot),
  statFile = stat,
  readInventoryFile = readFile,
}) {
  try {
    if (typeof runGit !== "function") fail();
    const inventory = parseJ3ReferenceInventory(await readInventory(inventoryPath, repoRoot, statFile, readInventoryFile));
    const remoteRef = remoteArchiveRef(inventory.archiveRef);
    const actualHead = (await callGit(runGit, ["rev-parse", "--verify", `${remoteRef}^{commit}`])).trim();
    if (actualHead !== inventory.archiveHead) fail();
    await callGit(runGit, ["cat-file", "-e", `${inventory.archiveHead}^{commit}`]);
    for (const commit of inventory.commits) {
      await callGit(runGit, ["cat-file", "-e", `${commit}^{commit}`]);
      await callGit(runGit, ["merge-base", "--is-ancestor", commit, inventory.archiveHead]);
    }
    const note = await callGit(runGit, [
      "notes",
      `--ref=${inventory.notesRef}`,
      "show",
      inventory.archiveHead,
    ]);
    if (!/REPRISE\s+JALON\s+3/iu.test(note)) fail();
    return {
      archiveHead: inventory.archiveHead,
      checkedCommits: inventory.commits,
      noteMatched: true,
    };
  } catch {
    fail();
  }
}
function parseArguments(argv) {
  if (!Array.isArray(argv) || argv.length !== 2 || argv[0] !== "--inventory") fail();
  const inventoryPath = argv[1];
  assertSafePath(inventoryPath);
  return inventoryPath;
}
function isDirectExecution() {
  return process.argv[1]
    && fileURLToPath(import.meta.url) === resolve(process.argv[1]);
}
if (isDirectExecution()) {
  try {
    const repoRoot = process.cwd();
    const result = await verifyJ3ReferenceArchive({
      repoRoot,
      inventoryPath: parseArguments(process.argv.slice(2)),
    });
    process.stdout.write(`${result.checkedCommits.length} J3 reference commits verified at ${result.archiveHead}.\n`);
  } catch {
    process.stderr.write(`${FAILURE}.\n`);
    process.exitCode = 1;
  }
}
