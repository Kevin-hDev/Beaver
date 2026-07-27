# Beaver Extension SDK — API v1

Beaver extensions are trusted local code. They run in a separate Node.js host with the rights and environment of the current user account. The host is not a sandbox.

`access` and `apiLevel` describe compatibility and intended use. They are not process-isolation or security boundaries. Beaver validates registered contributions again in Rust, but extension code remains fully trusted code.

## Minimal manifest

Create `beaver-extension.json` in the extension folder:

```json
{
  "id": "com.example.hello",
  "name": "Hello",
  "version": "1.0.0",
  "beaverApi": "1",
  "runtime": "node",
  "main": "./index.ts",
  "access": "full",
  "apiLevel": "stable"
}
```

## Minimal extension

```ts
import { defineExtension } from "@beaver/sdk";

export default defineExtension(async (beaver) => {
  beaver.on("session.turn.started", async ({ sessionId }) => {
    // React to a Beaver event.
  });

  beaver.registerTool({
    name: "hello",
    description: "Return a greeting.",
    parameters: {
      type: "object",
      properties: {
        name: { type: "string" }
      },
      required: ["name"],
      additionalProperties: false
    },
    async execute({ name }) {
      return `Hello ${name}`;
    }
  });
});
```

Beaver namespaces this tool as `com.example.hello.hello`.

## Stable API

- `beaver.info()`
- `beaver.registerTool(definition)`
- `beaver.on(event, handler)`
- `beaver.sessions.list()` and `beaver.sessions.get(id)`
- `beaver.projects.list()`
- `beaver.mcp.listConnectors()` and `beaver.mcp.callTool(...)`
- `beaver.channels.getConfig()`
- `beaver.secrets.getProviderKey(...)`
- `beaver.secrets.getMcpOAuthToken(...)`
- `beaver.secrets.getMcpEnvValue(...)`
- `beaver.secrets.getChannelToken(...)`
- `beaver.call(method, params)` for the versioned low-level bridge

## Advanced API

Set `"apiLevel": "advanced"` to use:

```ts
beaver.unstable.registerReplacement({
  name: "web_search",
  description: "My replacement",
  parameters: { type: "object" },
  async execute() {
    return "replacement result";
  }
});
```

`beaver.unstable.call(...)` and replacement points may change between Beaver versions.

The host is shared by the enabled local extensions. Changing the enabled set restarts the host so that removed or disabled code is terminated; other extensions are therefore activated again.

Secrets are zeroized by Beaver on the Rust side after transfer. Once a secret crosses into JavaScript, immutable strings and the JavaScript garbage collector prevent Beaver from guaranteeing immediate memory erasure.

Safe loading diagnostics (stage, category, source filename, and position when available) appear in **Settings → Extensions → Host and Diagnostics**. Raw extension output is not persisted because it may contain secrets.

The extension author and user are responsible for any secret, file, process, or network access performed after activation.
