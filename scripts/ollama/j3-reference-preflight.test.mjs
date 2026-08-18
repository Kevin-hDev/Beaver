import assert from "node:assert/strict";
import { mkdtemp, open, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";
import { test } from "node:test";

import {
  parseJ3ReferenceAuthority,
  verifyJ3ReferenceArchive,
} from "./j3-reference-preflight.mjs";

const DIRECTORY = dirname(fileURLToPath(import.meta.url));
const AUTHORITY = join(DIRECTORY, "j3-reference-authority.json");
const PACKAGE_MANIFEST = join(DIRECTORY, "../../package.json");
const EXPECTED_HEAD = "50b5515c6d849945f08073d04eebb0aecb479f26";
const EXPECTED_COMMITS = [
  "d565b66410110287e5087c33c33c8edcfd3b2db3",
  "57d380a0b954794626462f92e88aaa74393e791b",
  "fe9010d4b8360d2cb52248a288a083f8fb4a2e10",
  "8a6b90e562e601aec979415a6b89a0497a53f160",
  "b87293901235817b3840977634d233de0f6f3066",
  EXPECTED_HEAD,
];
const MAX_AUTHORITY_BYTES = 16 * 1024;
const GENERIC_FAILURE = /J3 reference preflight failed/u;

async function parsedAuthority() {
  return parseJ3ReferenceAuthority(await readFile(AUTHORITY, "utf8"));
}

function archiveGitOutput(authority, note = "REPRISE JALON 3") {
  const calls = [];
  return {
    calls,
    runGit: async (args) => {
      calls.push(args);
      if (args[0] === "fetch") return "";
      if (args[0] === "rev-parse") return `${authority.archiveHead}\n`;
      if (["cat-file", "merge-base"].includes(args[0])) return "";
      if (args[0] === "notes") return note;
      throw new Error("unexpected command");
    },
  };
}

test("the npm preflight uses the tracked authority", async () => {
  const manifest = JSON.parse(await readFile(PACKAGE_MANIFEST, "utf8"));
  assert.equal(
    manifest.scripts["preflight:j3-reference"],
    "node scripts/ollama/j3-reference-preflight.mjs --authority scripts/ollama/j3-reference-authority.json",
  );
});

test("parses the bounded tracked authority", async () => {
  assert.deepEqual(await parsedAuthority(), {
    archiveRef: "refs/heads/codex/fix-app-shutdown-lifecycle",
    archiveHead: EXPECTED_HEAD,
    notesRef: "refs/notes/commits",
    commits: EXPECTED_COMMITS,
  });
});

test("fetches remote notes into a dedicated ref and verifies all evidence", async () => {
  const authority = await parsedAuthority();
  const { calls, runGit } = archiveGitOutput(authority);
  const result = await verifyJ3ReferenceArchive({ repoRoot: DIRECTORY, authorityPath: AUTHORITY, runGit });

  assert.deepEqual(result, {
    archiveHead: EXPECTED_HEAD,
    checkedCommits: EXPECTED_COMMITS,
    noteMatched: true,
  });
  assert.deepEqual(calls[0], [
    "fetch", "--no-tags", "--force", "origin",
    "+refs/heads/codex/fix-app-shutdown-lifecycle:refs/remotes/origin/codex/fix-app-shutdown-lifecycle",
    "+refs/notes/commits:refs/notes/beaver-j3-preflight",
  ]);
  assert.equal(calls.some((args) => args.includes("refs/notes/commits:refs/notes/commits")), false);
  assert.deepEqual(calls.at(-1), ["notes", "--ref=refs/notes/beaver-j3-preflight", "show", EXPECTED_HEAD]);
});

test("rejects divergent, missing and non-ancestor evidence", async () => {
  const authority = await parsedAuthority();
  for (const failure of ["head", "note", "object", "ancestor"]) {
    const { runGit } = archiveGitOutput(authority, failure === "note" ? "" : "REPRISE JALON 3");
    const failingGit = async (args) => {
      if (failure === "head" && args[0] === "rev-parse") return `${"a".repeat(40)}\n`;
      if (failure === "object" && args[0] === "cat-file" && args[2]?.startsWith(EXPECTED_COMMITS[0])) throw new Error("missing");
      if (failure === "ancestor" && args[0] === "merge-base") throw new Error("not ancestor");
      return runGit(args);
    };
    await assert.rejects(
      verifyJ3ReferenceArchive({ repoRoot: DIRECTORY, authorityPath: AUTHORITY, runGit: failingGit }),
      GENERIC_FAILURE,
    );
  }
});

test("rejects malformed, duplicate, short and extra authority values", async () => {
  const valid = JSON.parse(await readFile(AUTHORITY, "utf8"));
  for (const invalid of [
    { ...valid, archiveHead: "short" },
    { ...valid, commits: [...valid.commits.slice(0, 5), valid.commits[0]] },
    { ...valid, commits: valid.commits.slice(0, 5) },
    { ...valid, extra: true },
    { ...valid, archiveRef: "refs/heads/../unsafe" },
  ]) {
    assert.throws(() => parseJ3ReferenceAuthority(JSON.stringify(invalid)), GENERIC_FAILURE);
  }
});

test("rejects an oversized authority before reading its body", async () => {
  const root = await mkdtemp(join(tmpdir(), "j3-reference-"));
  const path = join(root, "authority.json");
  await writeFile(path, Buffer.alloc(MAX_AUTHORITY_BYTES + 1, "x"));
  const openFile = async (...args) => {
    const handle = await open(...args);
    return {
      stat: (...statArgs) => handle.stat(...statArgs),
      read: () => { throw new Error("body must not be read"); },
      close: (...closeArgs) => handle.close(...closeArgs),
    };
  };
  try {
    await assert.rejects(
      verifyJ3ReferenceArchive({ repoRoot: root, authorityPath: path, runGit: async () => "", openFile }),
      GENERIC_FAILURE,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("bounds a size race to one extra byte", async () => {
  const root = await mkdtemp(join(tmpdir(), "j3-reference-race-"));
  const path = join(root, "authority.json");
  await writeFile(path, Buffer.alloc(MAX_AUTHORITY_BYTES + 8, "x"));
  let requestedBytes = 0;
  const openFile = async (...args) => {
    const handle = await open(...args);
    return {
      stat: async () => ({ size: 1 }),
      read: async (...readArgs) => {
        requestedBytes = readArgs[2];
        return handle.read(...readArgs);
      },
      close: (...closeArgs) => handle.close(...closeArgs),
    };
  };
  try {
    await assert.rejects(
      verifyJ3ReferenceArchive({ repoRoot: root, authorityPath: path, runGit: async () => "", openFile }),
      GENERIC_FAILURE,
    );
    assert.equal(requestedBytes, MAX_AUTHORITY_BYTES + 1);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
