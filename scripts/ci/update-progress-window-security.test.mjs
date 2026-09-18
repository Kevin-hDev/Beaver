import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

// Decode file URLs through Node so Windows drive letters remain valid paths.
const root = fileURLToPath(new URL("../../", import.meta.url));
const capabilityPath = join(root, "src-tauri/capabilities/update-progress.json");
const htmlPath = join(root, "update-window.html");
const mainPath = join(root, "src/update-window-main.tsx");

test("update progress capability exposes only the four required permissions", () => {
  const capability = JSON.parse(readFileSync(capabilityPath, "utf8"));
  assert.deepEqual(capability.windows, ["update-progress"]);
  assert.deepEqual(capability.permissions, [
    "core:event:allow-listen",
    "core:event:allow-unlisten",
    "core:window:allow-close",
    "core:window:allow-start-dragging",
  ]);
  const serialized = JSON.stringify(capability);
  for (const forbidden of [
    "core:default", "core:event:default", "allow-emit", "allow-emit-to",
    "allow-set-size", "remote", "fs:", "shell:", "dialog:",
  ]) assert.equal(serialized.includes(forbidden), false, forbidden);
});

test("update progress entry points keep a closed local content surface", () => {
  const componentRoot = join(root, "src/components/updates/window");
  const components = existsSync(componentRoot)
    ? readdirSync(componentRoot, { recursive: true })
      .filter((entry) => /\.(?:css|ts|tsx)$/.test(String(entry)))
      .map((entry) => readFileSync(join(componentRoot, String(entry)), "utf8"))
    : [];
  const source = [readFileSync(htmlPath, "utf8"), readFileSync(mainPath, "utf8"), ...components]
    .join("\n");
  const sourceWithoutLocalIpc = source.replaceAll("http://ipc.localhost", "");
  for (const forbidden of [
    "innerHTML", "dangerouslySetInnerHTML", "eval(", "new Function", "http://", "https://",
    "window.location", "location.href",
  ]) assert.equal(sourceWithoutLocalIpc.includes(forbidden), false, forbidden);
  assert.match(source, /Content-Security-Policy/);
  assert.match(source, /connect-src ipc: http:\/\/ipc\.localhost/);
  assert.match(source, /script-src 'self'/);
});

test("update progress CSP admits Vite development styles without admitting inline scripts", () => {
  const html = readFileSync(htmlPath, "utf8");
  const policy = html.match(/content="([^"]+)"/)?.[1] ?? "";
  assert.match(policy, /style-src 'self' 'unsafe-inline'/);
  assert.doesNotMatch(policy, /script-src[^;]*'unsafe-inline'/);
});
