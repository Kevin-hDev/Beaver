import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createHost } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostScript = join(root, "src-tauri/target/extension-host/host.mjs");

test("loads TypeScript tools, events and core calls through Jiti", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-extension-host-test-"));
  const source = join(directory, "index.ts");
  await writeFile(
    source,
    `import { defineExtension } from "@beaver/sdk";
    export default defineExtension(async function (api: any) {
      process.stdout.write("third-party-noise\\n");
      api.on("session.turn.started", async () => {});
      api.registerTool({
        name: "echo",
        description: "Echo a value",
        parameters: {
          type: "object",
          properties: { text: { type: "string" } },
          required: ["text"],
          additionalProperties: false
        },
        async execute(args: { text: string }) {
          const info: any = await api.info();
          return { content: info.apiVersion + ":" + args.text };
        }
      });
    });`,
    { mode: 0o600 },
  );

  const host = createHost(hostScript);
  try {
    const hello = await host.request("host.hello", {});
    assert.equal(hello.apiVersion, "1");
    assert.equal(hello.jitiVersion, "2.7.0");

    const sync = await host.request("host.sync", {
      extensions: [{
        id: "com.beaver.test",
        mainPath: source,
        manifest: { apiLevel: "stable" },
      }],
    });
    assert.equal(sync.extensions[0].contributions.tools[0].name, "com.beaver.test.echo");
    assert.deepEqual(sync.extensions[0].contributions.events, ["session.turn.started"]);

    const event = await host.request("event.emit", {
      event: "session.turn.started",
      payload: { sessionId: "test" },
    });
    assert.equal(event.delivered, 1);

    const result = await host.request("tool.call", {
      name: "com.beaver.test.echo",
      arguments: { text: "hello" },
    });
    assert.equal(result.content, "1:hello");
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});

test("isolates a failed extension and supports explicit advanced replacements", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-extension-host-test-"));
  const invalidSource = join(directory, "invalid.ts");
  const advancedSource = join(directory, "advanced.ts");
  await writeFile(invalidSource, "export const value = true;", { mode: 0o600 });
  await writeFile(
    advancedSource,
    `export default function (api: any) {
      api.unstable.registerReplacement({
        name: "web_search",
        description: "Custom search",
        parameters: { type: "object" },
        execute() { return "custom"; }
      });
    }`,
    { mode: 0o600 },
  );

  const host = createHost(hostScript);
  try {
    const sync = await host.request("host.sync", {
      extensions: [
        {
          id: "com.beaver.invalid",
          mainPath: invalidSource,
          manifest: { apiLevel: "stable" },
        },
        {
          id: "com.beaver.advanced",
          mainPath: advancedSource,
          manifest: { apiLevel: "advanced" },
        },
      ],
    });
    assert.equal(sync.extensions[0].error, "load_failed");
    assert.equal(sync.extensions[0].diagnostic.stage, "activate");
    assert.equal(sync.extensions[0].diagnostic.code, "activation_failed");
    assert.equal(sync.extensions[1].contributions.tools[0].name, "web_search");
    assert.equal(sync.extensions[1].contributions.tools[0].replacesCore, true);

    const result = await host.request("tool.call", {
      name: "web_search",
      arguments: {},
    });
    assert.equal(result.content, "custom");
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});
