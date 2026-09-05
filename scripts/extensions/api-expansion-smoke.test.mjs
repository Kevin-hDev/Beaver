import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { test } from "node:test";

import { runApiExpansionSmoke } from "./api-expansion-smoke.mjs";
import { createHost } from "./host-test-client.mjs";

const root = resolve(".");
const hostScript = join(root, "src-tauri", "target", "extension-host", "host.mjs");
const executable = join(
  root,
  "src-tauri",
  "target",
  "extension-host",
  "runtime",
  process.platform === "win32" ? "node.exe" : "node",
);

test("la fixture API expansion exerce outils, skill, ressources et artefacts", async () => {
  const workingDirectory = await mkdtemp(join(tmpdir(), "beaver-api-expansion-"));
  const secondarySource = join(workingDirectory, "secondary.mjs");
  await writeFile(
    secondarySource,
    'export default (api) => api.registerTool({ name: "still_healthy", description: "Secondary fixture", parameters: { type: "object" }, effect: "read-only", execute: () => "ok" });',
  );
  const host = createHost(hostScript, { executable });
  try {
    const outcome = await runApiExpansionSmoke({
      host,
      workingDirectory,
      secondExtension: {
        id: "acceptance.secondary",
        mainPath: secondarySource,
        manifest: { apiLevel: "stable" },
      },
    });
    assert.deepEqual(outcome.toolNames, [
      "acceptance.api.expansion.catalog_probe",
      "acceptance.api.expansion.produce_artifacts",
    ]);
    assert.equal(outcome.skill.id, "reference-skill");
    assert.deepEqual(outcome.resourceIds, ["reference", "preview"]);
    assert.match(outcome.textResource, /API-P7/u);
    assert.equal(outcome.previewSignature.toString("hex"), "89504e470d0a1a0a");
    assert.equal(outcome.richResult.content.length, 3);
    assert.equal(outcome.richResult.content[1].purpose, "artifact");
    assert.equal(outcome.richResult.content[2].purpose, "preview");
    assert.equal(
      await readFile(join(workingDirectory, "acceptance-artifact.txt"), "utf8"),
      "API-P7 artifact\n",
    );
  } finally {
    host.stop();
    await host.exited;
    await rm(workingDirectory, { recursive: true, force: true });
  }
});
