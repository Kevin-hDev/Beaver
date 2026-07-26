#!/usr/bin/env node

const baseUrl = process.env.PROVIDER_BASE_URL ?? "https://api.openai.com/v1";
const apiKey = process.env.PROVIDER_API_KEY ?? process.env.OPENAI_API_KEY;
const model = process.env.PROVIDER_MODEL ?? "gpt-4o-mini";

if (!apiKey) {
  console.error("Set PROVIDER_API_KEY or OPENAI_API_KEY before running this check.");
  process.exit(2);
}

const response = await fetch(`${baseUrl}/chat/completions`, {
  method: "POST",
  headers: {
    "content-type": "application/json",
    authorization: `Bearer ${apiKey}`,
  },
  body: JSON.stringify({
    model,
    messages: [{ role: "user", content: "Reply with one short sentence." }],
    stream: true,
    stream_options: { include_usage: true },
  }),
});

if (!response.ok || !response.body) {
  console.error(`Request failed: ${response.status} ${await response.text()}`);
  process.exit(1);
}

const decoder = new TextDecoder();
let buffer = "";
let usageSeen = false;

for await (const chunk of response.body) {
  buffer += decoder.decode(chunk, { stream: true });
  const lines = buffer.split("\n");
  buffer = lines.pop() ?? "";
  for (const line of lines) {
    if (!line.startsWith("data: ")) continue;
    const data = line.slice(6).trim();
    if (!data || data === "[DONE]") continue;
    const parsed = JSON.parse(data);
    if (parsed.usage) {
      usageSeen = true;
      console.log(JSON.stringify(parsed.usage, null, 2));
    }
  }
}

if (!usageSeen) {
  console.error("No usage chunk received.");
  process.exit(1);
}
