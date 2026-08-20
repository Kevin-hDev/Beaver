import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const wdioSource = readFileSync(new URL("../../wdio.conf.ts", import.meta.url), "utf8");
const invokeSource = readFileSync(
  new URL("../../src-tauri/src/invoke_handler.rs", import.meta.url),
  "utf8",
);
const commandSource = readFileSync(
  new URL("../../src-tauri/src/commands/e2e.rs", import.meta.url),
  "utf8",
);

test("every native E2E journey verifies chat_stream child read-only admission", () => {
  assert.match(
    wdioSource,
    /const childSessionReadOnlySpec = ["'].+child-session-read-only\.spec\.ts["']/u,
  );
  assert.match(
    wdioSource,
    /E2E_REQUIRE_CEF_SMOKE[\s\S]*native-cef-shutdown\.spec\.ts/u,
  );
  assert.match(
    wdioSource,
    /E2E_REQUIRE_WEBVIEW_SMOKE[\s\S]*native-webview-shutdown\.spec\.ts/u,
  );
  assert.match(
    wdioSource,
    /specs:\s*\[\[shutdownSpec,\s*childSessionReadOnlySpec\]\]/u,
  );
  assert.match(invokeSource, /#\[cfg\(feature = "e2e"\)\][\s\S]*e2e_verify_child_chat_stream_read_only/u);
  assert.match(commandSource, /pub async fn e2e_verify_child_chat_stream_read_only/u);
});
