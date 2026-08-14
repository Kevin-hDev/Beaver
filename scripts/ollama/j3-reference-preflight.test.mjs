import assert from "node:assert/strict";
import { mkdtemp, open, readFile, rm, stat, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";
import { test } from "node:test";

import {
  parseJ3ReferenceInventory,
  verifyJ3ReferenceArchive,
} from "./j3-reference-preflight.mjs";

const TEST_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const VALID_FIXTURE = join(TEST_DIRECTORY, "fixtures/j3-reference/valid-inventory.md");
const MISSING_COMMIT_FIXTURE = join(TEST_DIRECTORY, "fixtures/j3-reference/missing-commit.md");
const EXPECTED_HEAD = "50b5515c6d849945f08073d04eebb0aecb479f26";
const EXPECTED_COMMITS = [
  "d565b66410110287e5087c33c33c8edcfd3b2db3",
  "57d380a0b954794626462f92e88aaa74393e791b",
  "fe9010d4b8360d2cb52248a288a083f8fb4a2e10",
  "8a6b90e562e601aec979415a6b89a0497a53f160",
  "b87293901235817b3840977634d233de0f6f3066",
  "50b5515c6d849945f08073d04eebb0aecb479f26",
];
const MAX_INVENTORY_BYTES = 256 * 1024;
const GENERIC_FAILURE = /J3 reference preflight failed/u;

async function readFixture(path) {
  return readFile(path, "utf8");
}

function archiveGitOutput(inventory, note = "REPRISE JALON 3 — BRANCHE HISTORIQUE RESTAURÉE") {
  const calls = [];
  const runGit = async (args) => {
    calls.push(args);
    assert.equal(Array.isArray(args), true);
    assert.equal(args.every((argument) => typeof argument === "string"), true);
    const [command] = args;
    if (command === "rev-parse") return `${inventory.archiveHead}\n`;
    if (command === "cat-file") return "";
    if (command === "merge-base") return "";
    if (command === "notes") return note;
    throw new Error(`unexpected git command: ${args.join(" ")}`);
  };
  return { calls, runGit };
}

test("parse l'inventaire J3 valide avec six commits complets", async () => {
  const parsed = parseJ3ReferenceInventory(await readFixture(VALID_FIXTURE));

  assert.deepEqual(parsed, {
    archiveRef: "refs/heads/codex/fix-app-shutdown-lifecycle",
    archiveHead: EXPECTED_HEAD,
    notesRef: "refs/notes/commits",
    commits: EXPECTED_COMMITS,
  });
});

test("vérifie la tête distante exacte, les objets, les ancêtres et la note", async () => {
  const inventory = parseJ3ReferenceInventory(await readFixture(VALID_FIXTURE));
  const { calls, runGit } = archiveGitOutput(inventory);

  const result = await verifyJ3ReferenceArchive({
    repoRoot: TEST_DIRECTORY,
    inventoryPath: VALID_FIXTURE,
    runGit,
  });

  assert.deepEqual(result, {
    archiveHead: EXPECTED_HEAD,
    checkedCommits: EXPECTED_COMMITS,
    noteMatched: true,
  });
  assert.equal(calls.some((args) => args[0] === "rev-parse"), true);
  assert.equal(calls.filter((args) => args[0] === "cat-file").length, 7);
  assert.equal(calls.filter((args) => args[0] === "merge-base").length, 6);
  assert.equal(calls.some((args) => args[0] === "notes"), true);
});

test("bloque une tête divergente", async () => {
  const inventory = parseJ3ReferenceInventory(await readFixture(VALID_FIXTURE));
  const { runGit } = archiveGitOutput({ ...inventory, archiveHead: "a".repeat(40) });

  await assert.rejects(
    () => verifyJ3ReferenceArchive({ repoRoot: TEST_DIRECTORY, inventoryPath: VALID_FIXTURE, runGit }),
    GENERIC_FAILURE,
  );
});

test("bloque une note absente", async () => {
  const inventory = parseJ3ReferenceInventory(await readFixture(VALID_FIXTURE));
  const { runGit } = archiveGitOutput(inventory, "");

  await assert.rejects(
    () => verifyJ3ReferenceArchive({ repoRoot: TEST_DIRECTORY, inventoryPath: VALID_FIXTURE, runGit }),
    GENERIC_FAILURE,
  );
});

test("bloque un objet absent", async () => {
  const inventory = parseJ3ReferenceInventory(await readFixture(VALID_FIXTURE));
  const { runGit } = archiveGitOutput(inventory);
  let catFileCalls = 0;
  const runMissingObject = async (args) => {
    if (args[0] === "cat-file" && ++catFileCalls === 2) throw new Error("missing object");
    return runGit(args);
  };

  await assert.rejects(
    () => verifyJ3ReferenceArchive({ repoRoot: TEST_DIRECTORY, inventoryPath: VALID_FIXTURE, runGit: runMissingObject }),
    GENERIC_FAILURE,
  );
});

test("bloque un commit qui n'est pas ancêtre", async () => {
  const inventory = parseJ3ReferenceInventory(await readFixture(VALID_FIXTURE));
  const { runGit } = archiveGitOutput(inventory);
  const runNonAncestor = async (args) => {
    if (args[0] === "merge-base" && args[2] === EXPECTED_COMMITS[0]) throw new Error("not ancestor");
    return runGit(args);
  };

  await assert.rejects(
    () => verifyJ3ReferenceArchive({ repoRoot: TEST_DIRECTORY, inventoryPath: VALID_FIXTURE, runGit: runNonAncestor }),
    GENERIC_FAILURE,
  );
});

test("bloque une ligne J3 supplémentaire", async () => {
  const markdown = `${await readFixture(VALID_FIXTURE)}| 23 | \`1111111111111111111111111111111111111111\` | extra |\n`;

  assert.throws(() => parseJ3ReferenceInventory(markdown), GENERIC_FAILURE);
});

test("bloque un SHA court", async () => {
  const markdown = (await readFixture(VALID_FIXTURE)).replace(EXPECTED_COMMITS[0], "d565b66");

  assert.throws(() => parseJ3ReferenceInventory(markdown), GENERIC_FAILURE);
});

test("bloque un SHA J3 de 39 caractères", async () => {
  const markdown = (await readFixture(VALID_FIXTURE)).replace(EXPECTED_COMMITS[0], EXPECTED_COMMITS[0].slice(0, 39));

  assert.throws(() => parseJ3ReferenceInventory(markdown), GENERIC_FAILURE);
});

test("bloque un SHA J3 de 41 caractères", async () => {
  const markdown = (await readFixture(VALID_FIXTURE)).replace(EXPECTED_COMMITS[0], `${EXPECTED_COMMITS[0]}0`);

  assert.throws(() => parseJ3ReferenceInventory(markdown), GENERIC_FAILURE);
});

test("bloque une valeur dupliquée", async () => {
  const markdown = (await readFixture(VALID_FIXTURE)).replace(EXPECTED_COMMITS[1], EXPECTED_COMMITS[0]);

  assert.throws(() => parseJ3ReferenceInventory(markdown), GENERIC_FAILURE);
});

test("bloque une commande Git non nulle", async () => {
  const inventory = parseJ3ReferenceInventory(await readFixture(VALID_FIXTURE));
  const runGit = async () => {
    throw new Error("private git detail");
  };

  await assert.rejects(
    () => verifyJ3ReferenceArchive({ repoRoot: TEST_DIRECTORY, inventoryPath: VALID_FIXTURE, runGit }),
    GENERIC_FAILURE,
  );
});

test("bloque l'inventaire auquel il manque un commit", async () => {
  const markdown = await readFixture(MISSING_COMMIT_FIXTURE);
  assert.throws(() => parseJ3ReferenceInventory(markdown), GENERIC_FAILURE);
});

test("refuse un fichier de plus de 256 Kio avant sa lecture complète", async () => {
  const temporaryRoot = await mkdtemp(join(tmpdir(), "j3-reference-"));
  const inventoryPath = join(temporaryRoot, "oversized.md");
  await writeFile(inventoryPath, Buffer.alloc(MAX_INVENTORY_BYTES + 1, "x"));
  let statCalls = 0;
  let openCalls = 0;
  const statFile = async (...args) => {
    statCalls += 1;
    return stat(...args);
  };
  const openFile = async () => {
    openCalls += 1;
    throw new Error("open should not be called");
  };

  try {
    await assert.rejects(
      () => verifyJ3ReferenceArchive({
        repoRoot: temporaryRoot,
        inventoryPath,
        runGit: async () => "",
        statFile,
        openFile,
      }),
      GENERIC_FAILURE,
    );
    assert.equal(statCalls, 1);
    assert.equal(openCalls, 0);
  } finally {
    await rm(temporaryRoot, { recursive: true, force: true });
  }
});

test("borne la lecture après une course de taille", async () => {
  const temporaryRoot = await mkdtemp(join(tmpdir(), "j3-reference-race-"));
  const inventoryPath = join(temporaryRoot, "race.md");
  await writeFile(inventoryPath, Buffer.alloc(MAX_INVENTORY_BYTES + 32, "x"));
  let openCalls = 0;
  let requestedBytes = 0;
  const statFile = async () => ({ size: 1 });
  const openFile = async (...args) => {
    openCalls += 1;
    const handle = await open(...args);
    return {
      read: async (...readArgs) => {
        requestedBytes = readArgs[2];
        return handle.read(...readArgs);
      },
      close: (...closeArgs) => handle.close(...closeArgs),
    };
  };

  try {
    await assert.rejects(
      () => verifyJ3ReferenceArchive({ repoRoot: temporaryRoot, inventoryPath, runGit: async () => "", statFile, openFile }),
      GENERIC_FAILURE,
    );
    assert.equal(openCalls, 1);
    assert.equal(requestedBytes, MAX_INVENTORY_BYTES + 1);
  } finally {
    await rm(temporaryRoot, { recursive: true, force: true });
  }
});
