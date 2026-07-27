# Beaver Extension SDK — API v1

Beaver extensions are trusted local code. They run in a separate Node.js host with the rights and environment of the current user account. The host is not a sandbox.

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

The extension author and user are responsible for any secret, file, process, or network access performed after activation.
