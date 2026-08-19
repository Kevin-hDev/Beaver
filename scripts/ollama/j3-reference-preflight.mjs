import { execFile } from "node:child_process";
import { open } from "node:fs/promises";
import { isAbsolute, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const COMMIT_COUNT = 6;
const MAX_AUTHORITY_BYTES = 16 * 1024;
const MAX_OUTPUT_BYTES = 64 * 1024;
const MAX_PATH_LENGTH = 4096;
const MAX_TIMEOUT_MS = 30_000;
const PREFLIGHT_NOTES_REF = "refs/notes/beaver-j3-preflight";
const SHA_PATTERN = /^[0-9a-f]{40}$/u;
const REF_PATTERN = /^refs\/(?:heads|notes)\/[A-Za-z0-9][A-Za-z0-9._/-]{0,255}$/u;
const FAILURE = "J3 reference preflight failed";

function fail() {
  throw new Error(FAILURE);
}

function safeText(value, maximumLength) {
  return typeof value === "string"
    && value.length > 0
    && value.length <= maximumLength
    && !/[\0\r\n]/u.test(value);
}

function assertSafePath(value) {
  if (!safeText(value, MAX_PATH_LENGTH) || value.split(/[\\/]+/u).includes("..")) fail();
}

function validateRef(value, kind) {
  if (!safeText(value, 512) || !REF_PATTERN.test(value) || !value.startsWith(`refs/${kind}/`)) fail();
  if (value.split("/").some((segment) => !segment || segment === "." || segment === "..")) fail();
  return value;
}

export function parseJ3ReferenceAuthority(raw) {
  if (typeof raw !== "string" || raw.length === 0 || /\0/u.test(raw)
    || Buffer.byteLength(raw, "utf8") > MAX_AUTHORITY_BYTES) fail();
  let value;
  try {
    value = JSON.parse(raw);
  } catch {
    fail();
  }
  const keys = Object.keys(value ?? {}).sort();
  const expected = ["archiveHead", "archiveRef", "commits", "notesRef", "schemaVersion"];
  if (JSON.stringify(keys) !== JSON.stringify(expected)) fail();
  const archiveRef = validateRef(value.archiveRef, "heads");
  const notesRef = validateRef(value.notesRef, "notes");
  if (value.schemaVersion !== 1 || !SHA_PATTERN.test(value.archiveHead)) fail();
  if (!Array.isArray(value.commits) || value.commits.length !== COMMIT_COUNT) fail();
  if (!value.commits.every((commit) => SHA_PATTERN.test(commit))) fail();
  if (new Set(value.commits).size !== COMMIT_COUNT) fail();
  return { archiveRef, archiveHead: value.archiveHead, notesRef, commits: [...value.commits] };
}

function validateGitArgs(args) {
  if (!Array.isArray(args) || args.length === 0 || args.length > 16) fail();
  if (!args.every((argument) => safeText(argument, MAX_PATH_LENGTH))) fail();
}

async function callGit(runGit, args) {
  validateGitArgs(args);
  try {
    const result = await runGit([...args]);
    const output = typeof result === "string" ? result : result?.stdout;
    if (typeof output !== "string" || Buffer.byteLength(output, "utf8") > MAX_OUTPUT_BYTES) fail();
    if (result && typeof result === "object" && result.status !== undefined && result.status !== 0) fail();
    return output;
  } catch {
    fail();
  }
}

function remoteArchiveRef(archiveRef) {
  return `refs/remotes/origin/${archiveRef.slice("refs/heads/".length)}`;
}

async function readAuthority(authorityPath, repoRoot, openFile = open) {
  assertSafePath(repoRoot);
  assertSafePath(authorityPath);
  const path = isAbsolute(authorityPath) ? authorityPath : resolve(repoRoot, authorityPath);
  try {
    const handle = await openFile(path, "r");
    try {
      const metadata = await handle.stat();
      if (!Number.isSafeInteger(metadata.size) || metadata.size < 1 || metadata.size > MAX_AUTHORITY_BYTES) fail();
      const buffer = Buffer.alloc(MAX_AUTHORITY_BYTES + 1);
      const { bytesRead } = await handle.read(buffer, 0, buffer.length, 0);
      if (!Number.isSafeInteger(bytesRead) || bytesRead < 1 || bytesRead > MAX_AUTHORITY_BYTES) fail();
      return buffer.subarray(0, bytesRead).toString("utf8");
    } finally {
      await handle.close();
    }
  } catch {
    fail();
  }
}

function createGitRunner(repoRoot) {
  return async (args) => {
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
  authorityPath,
  runGit = createGitRunner(repoRoot),
  openFile = open,
}) {
  try {
    if (typeof runGit !== "function" || typeof openFile !== "function") fail();
    const authority = parseJ3ReferenceAuthority(await readAuthority(authorityPath, repoRoot, openFile));
    const remoteRef = remoteArchiveRef(authority.archiveRef);
    // This dedicated notes ref preserves the developer's official local notes authority.
    await callGit(runGit, [
      "fetch", "--no-tags", "--force", "origin",
      `+${authority.archiveRef}:${remoteRef}`,
      `+${authority.notesRef}:${PREFLIGHT_NOTES_REF}`,
    ]);
    const actualHead = (await callGit(runGit, ["rev-parse", "--verify", `${remoteRef}^{commit}`])).trim();
    if (actualHead !== authority.archiveHead) fail();
    await callGit(runGit, ["cat-file", "-e", `${authority.archiveHead}^{commit}`]);
    for (const commit of authority.commits) {
      await callGit(runGit, ["cat-file", "-e", `${commit}^{commit}`]);
      await callGit(runGit, ["merge-base", "--is-ancestor", commit, authority.archiveHead]);
    }
    const note = await callGit(runGit, [
      "notes", `--ref=${PREFLIGHT_NOTES_REF}`, "show", authority.archiveHead,
    ]);
    if (!/REPRISE\s+JALON\s+3/iu.test(note)) fail();
    return { archiveHead: authority.archiveHead, checkedCommits: authority.commits, noteMatched: true };
  } catch {
    fail();
  }
}

function parseArguments(argv) {
  if (!Array.isArray(argv) || argv.length !== 2 || argv[0] !== "--authority") fail();
  assertSafePath(argv[1]);
  return argv[1];
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  try {
    const result = await verifyJ3ReferenceArchive({
      repoRoot: process.cwd(),
      authorityPath: parseArguments(process.argv.slice(2)),
    });
    process.stdout.write(`${result.checkedCommits.length} J3 reference commits verified at ${result.archiveHead}.\n`);
  } catch {
    process.stderr.write(`${FAILURE}.\n`);
    process.exitCode = 1;
  }
}
