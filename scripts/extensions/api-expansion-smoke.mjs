import { readFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { resetAndLoad } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const fixtureRoot = join(root, "src-tauri", "tests", "fixtures", "extensions", "api-expansion");

export async function runApiExpansionSmoke({ host, workingDirectory, secondExtension }) {
  if (!host || typeof host.request !== "function" || typeof workingDirectory !== "string") {
    throw new Error("API expansion smoke unavailable");
  }
  const manifest = JSON.parse(await readFile(join(fixtureRoot, "beaver-extension.json"), "utf8"));
  const primary = {
    id: manifest.id,
    mainPath: join(fixtureRoot, manifest.main),
    manifest,
  };
  const extensions = secondExtension ? [primary, secondExtension] : [primary];
  const loaded = await resetAndLoad(host, extensions);
  const fixture = loaded.extensions[0];
  if (fixture?.error || !fixture?.contributions) throw new Error("API expansion smoke unavailable");

  const probe = await host.request("tool.call", {
    name: `${manifest.id}.catalog_probe`,
    arguments: { query: "unrelated words" },
    context: { workingDirectory },
  });
  if (!String(probe.content).startsWith("Explicit fixture call:")) {
    throw new Error("API expansion smoke unavailable");
  }
  const richResult = await host.request("tool.call", {
    name: `${manifest.id}.produce_artifacts`,
    arguments: {},
    context: { workingDirectory },
  });
  const [skill, ...skills] = fixture.contributions.skills;
  if (!skill || skills.length !== 0) throw new Error("API expansion smoke unavailable");
  const resources = fixture.contributions.resources;
  const textResource = await readFile(join(fixtureRoot, "resources", "reference.txt"), "utf8");
  const preview = await readFile(join(fixtureRoot, "resources", "preview.png"));
  if (secondExtension) {
    const secondary = await host.request("tool.call", {
      name: `${secondExtension.id}.still_healthy`,
      arguments: {},
      context: { workingDirectory },
    });
    if (secondary.content !== "ok") throw new Error("API expansion smoke unavailable");
  }
  await host.request("host.reset", {});
  await host.request("tool.call", {
    name: `${manifest.id}.catalog_probe`,
    arguments: { query: "after reset" },
    context: { workingDirectory },
  }).then(() => { throw new Error("API expansion smoke unavailable"); }, () => undefined);
  if (secondExtension) {
    const reloaded = await resetAndLoad(host, [secondExtension]);
    if (reloaded.extensions[0]?.error) throw new Error("API expansion smoke unavailable");
    const secondary = await host.request("tool.call", {
      name: `${secondExtension.id}.still_healthy`,
      arguments: {},
      context: { workingDirectory },
    });
    if (secondary.content !== "ok") throw new Error("API expansion smoke unavailable");
  }
  return {
    toolNames: fixture.contributions.tools.map(({ name }) => name),
    skill,
    resourceIds: resources.map(({ id }) => id),
    textResource,
    previewSignature: preview.subarray(0, 8),
    richResult,
  };
}
