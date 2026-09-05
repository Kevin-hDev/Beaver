import { lstat, realpath } from "node:fs/promises";
import { resolve } from "node:path";

const fixture = resolve("src-tauri/tests/fixtures/extensions/api-expansion");
const manifest = resolve(fixture, "beaver-extension.json");
const fixtureMetadata = await lstat(fixture);
const metadata = await lstat(manifest);
if (
  !fixtureMetadata.isDirectory()
  || fixtureMetadata.isSymbolicLink()
  || !metadata.isFile()
  || metadata.isSymbolicLink()
) {
  throw new Error("Packaged API expansion fixture unavailable");
}
process.env.BEAVER_E2E_API_EXPANSION_FIXTURE = await realpath(fixture);
process.env.E2E_PACKAGED = "1";
await import("./run.mjs");
