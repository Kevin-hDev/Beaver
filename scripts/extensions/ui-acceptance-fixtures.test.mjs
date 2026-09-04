import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstat, mkdtemp, open, readFile, readdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { afterEach, test } from "node:test";

import { buildExtensionUi } from "./ui-build.mjs";

const root = resolve("scripts/extensions/fixtures/ui");
const contract = JSON.parse(await readFile(resolve("src-tauri/resources/extension-ui/contract.json")));
const expected = [
  "advanced-tampered", "advanced-throws", "advanced-valid", "conflict-a", "conflict-b",
  "standard-complete", "standard-limits", "theme-invalid", "theme-valid", "unicode",
];
const temporaryRoots = [];

afterEach(async () => {
  await Promise.all(temporaryRoots.splice(0).map((path) => rm(path, { recursive: true, force: true })));
});

test("the acceptance inventory is exact, bounded, offline and free of links", async () => {
  assert.deepEqual(await fixtureDirectories(), expected);
  const files = await collectFiles(root);
  assert.ok(files.length <= 40);
  let totalBytes = 0;
  for (const file of files) {
    const handle = await open(file, "r");
    try {
      const metadata = await handle.stat();
      assert.equal(metadata.isFile(), true);
      assert.ok(metadata.size <= 65_536);
      totalBytes += metadata.size;
      const text = await handle.readFile("utf8");
      assert.doesNotMatch(text, /https?:|fetch\s*\(|XMLHttpRequest|WebSocket|node:(?:fs|net|http|child_process)/u);
    } finally {
      await handle.close();
    }
  }
  assert.ok(totalBytes <= 524_288);
  assert.match(await readFile(join(root, "README.md"), "utf8"), /AGPL-3\.0-only/u);
});

test("every fixture has one strict manifest identity and declared local entry", async () => {
  const identities = new Set();
  for (const directory of await fixtureDirectories()) {
    const location = join(root, directory);
    const manifest = JSON.parse(await readFile(join(location, "beaver-extension.json"), "utf8"));
    assert.match(manifest.id, /^[a-z][a-z0-9.-]{0,95}$/u);
    assert.equal(identities.has(manifest.id), false);
    identities.add(manifest.id);
    assert.equal(manifest.beaverApi, "1");
    assert.equal(manifest.ui.apiVersion, "1");
    assert.equal(typeof manifest.main, "string");
    for (const entry of [manifest.main, manifest.ui.mode === "advanced" && manifest.ui.entry]
      .filter(Boolean)) {
      const entryPath = join(location, entry);
      assert.equal(await realpath(entryPath), entryPath);
      assert.equal((await lstat(entryPath)).isFile(), true);
    }
  }
});

test("the complete standard fixture executes all four public surface families", async () => {
  const contributions = [];
  const handlers = [];
  const loaded = await importFixture("standard-complete/index.mjs");
  loaded.default({
    ui: {
      register: (value) => { contributions.push(value); return () => {}; },
      onAction: (id, handler) => { handlers.push([id, handler]); return () => {}; },
    },
  });
  assert.deepEqual(contributions.map(({ placement }) => placement).sort(), [
    "agent.composer.leading",
    "app.navigation.primary",
    "app.toolbar.primary",
    "settings.navigation.preferences",
  ]);
  assert.equal(handlers.length, 2);
  assert.deepEqual(await handlers[0][1](), {
    type: "notification", level: "success", message: { default: "Accepted" },
  });
});

test("negative limits are derived as max plus one and name every protected mutation", async () => {
  const loaded = await importFixture("standard-limits/index.mjs");
  const limits = contract.limits;
  assert.deepEqual(loaded.negativeCases, {
    contributions: limits.maxContributionsPerExtension + 1,
    themes: limits.maxThemesPerExtension + 1,
    actions: limits.maxActionsPerExtension + 1,
    viewNodes: limits.maxViewNodes + 1,
    viewDepth: limits.maxViewDepth + 1,
    fields: limits.maxFieldsPerView + 1,
    options: limits.maxOptionsPerField + 1,
    textChars: limits.maxTextChars + 1,
    extensionBytes: limits.maxUiBytesPerExtension + 1,
    actionPayloadBytes: limits.maxActionPayloadBytes + 1,
    actionResultBytes: limits.maxActionResultBytes + 1,
    protectedMutations: contract.protectedOccupants.map((entry) => ({
      placement: entry.placement,
      targetId: entry.occupant,
      operation: entry.operations[0],
    })),
  });
  let accepted = 0;
  assert.throws(() => loaded.default({ ui: { register: () => {
    accepted += 1;
    if (accepted > limits.maxContributionsPerExtension) throw new Error("bounded");
  } } }), /bounded/u);
});

test("equal-priority fixtures target the same occupant without sharing identity", async () => {
  const values = [];
  for (const directory of ["conflict-a", "conflict-b"]) {
    const loaded = await importFixture(`${directory}/index.mjs`);
    loaded.default({ ui: { register: (value) => { values.push(value); } } });
  }
  assert.equal(values[0].order, values[1].order);
  assert.equal(values[0].targetId, "beaver.settings");
  assert.equal(values[1].targetId, "beaver.settings");
  assert.equal(values[0].operation, "move");
  assert.notEqual(values[0].label.default, values[1].label.default);
});

test("valid and invalid themes differ only at the public token boundary", async () => {
  const values = [];
  for (const directory of ["theme-valid", "theme-invalid"]) {
    const loaded = await importFixture(`${directory}/index.mjs`);
    loaded.default({ ui: { register: (value) => { values.push(value); } } });
  }
  const publicTokens = new Set(contract.themeTokens);
  assert.ok(Object.keys(values[0].tokens).every((name) => publicTokens.has(name)));
  assert.ok(Object.keys(values[1].tokens).some((name) => !publicTokens.has(name)));
});

test("advanced fixtures build CSS, preserve failure, and detect modified bytes", async () => {
  const output = await temporaryDirectory();
  const valid = await buildExtensionUi({
    inputRoot: join(root, "advanced-valid"), outputRoot: join(output, "valid"), entry: "entry.ts",
  });
  assert.ok(valid.outputs.some(({ type }) => type === "javascript"));
  assert.ok(valid.outputs.some(({ type }) => type === "css"));
  const throwing = await buildExtensionUi({
    inputRoot: join(root, "advanced-throws"), outputRoot: join(output, "throws"), entry: "entry.mjs",
  });
  assert.ok(throwing.outputs.some(({ type }) => type === "javascript"));
  const tamperedRoot = join(output, "tampered");
  await import("node:fs/promises").then(({ mkdir }) => mkdir(tamperedRoot));
  const tampered = await buildExtensionUi({
    inputRoot: join(root, "advanced-tampered"), outputRoot: tamperedRoot, entry: "entry.mjs",
  });
  const entry = tampered.outputs.find(({ type }) => type === "javascript").name;
  const artifactPath = join(tamperedRoot, entry);
  const approved = await readFile(artifactPath);
  await writeFile(artifactPath, `${approved.toString("utf8")}\n// modified after approval\n`, "utf8");
  const modified = await readFile(artifactPath);
  // This fixture only proves that tampering changes the approved digest. The
  // serving refusal itself is exercised by Rust's ui_artifact_tests.
  assert.notEqual(sha256(approved), sha256(modified));
});

test("the Unicode fixture carries bounded French, German, Chinese and Japanese text", async () => {
  const source = await readFile(join(root, "unicode/index.mjs"), "utf8");
  for (const sample of ["Éléments français", "Deutsche Oberflächenelemente", "中文界面", "日本語"]) {
    assert.match(source, new RegExp(sample, "u"));
  }
  const loaded = await importFixture("unicode/index.mjs");
  let contribution;
  loaded.default({ ui: { register: (value) => { contribution = value; } } });
  for (const locale of ["fr", "de", "zh", "ja"]) {
    assert.ok(Array.from(contribution.label[locale]).length <= contract.limits.maxTextChars);
  }
});

test("the public SDK documents and types the complete advanced UI lifecycle", async () => {
  const readme = await readFile(resolve("src-tauri/resources/extension-host/sdk/README.md"), "utf8");
  const definitions = await readFile(resolve("src-tauri/resources/extension-host/sdk/index.d.ts"), "utf8");

  for (const marker of [
    "### Advanced interface modules",
    '"mode": "advanced"',
    "completeWithoutMounts",
    "same Beaver WebView",
    "scripts/extensions/fixtures/ui/",
  ]) {
    assert.match(readme, new RegExp(marker.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&"), "u"));
  }
  for (const typeName of [
    "BeaverAdvancedUiCleanup",
    "BeaverAdvancedUiMount",
    "BeaverAdvancedUiContext",
    "BeaverAdvancedUiModule",
  ]) {
    assert.match(definitions, new RegExp(`export (?:type|interface) ${typeName}\\b`, "u"));
  }
  assert.match(definitions, /mount\(placement: ExtensionUiPlacementKey,/u);
});

async function fixtureDirectories() {
  return (await readdir(root, { withFileTypes: true }))
    .filter((entry) => entry.isDirectory())
    .map(({ name }) => name)
    .sort();
}

async function collectFiles(directory) {
  const result = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) result.push(...await collectFiles(path));
    else if (entry.isFile()) result.push(path);
    else throw new Error("Unsupported acceptance fixture entry");
  }
  return result;
}

async function importFixture(path) {
  return import(`${pathToFileURL(join(root, path)).href}?acceptance=${basename(path)}`);
}

async function temporaryDirectory() {
  const directory = await realpath(await mkdtemp(join(tmpdir(), "beaver-ui-acceptance-")));
  temporaryRoots.push(directory);
  await import("node:fs/promises").then(({ mkdir }) => Promise.all([
    mkdir(join(directory, "valid")), mkdir(join(directory, "throws")),
  ]));
  return directory;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
