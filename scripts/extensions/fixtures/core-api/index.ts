import { defineExtension } from "@beaver/sdk";

export default defineExtension((beaver) => {
  const supports = (capability: string) =>
    beaver.capabilities?.includes(capability as never) === true;
  let completedTurns = 0;

  beaver.on("session.turn.completed", () => {
    completedTurns += 1;
  });

  if (supports("models") && beaver.models) {
    beaver.registerTool({
      name: "generate_summary",
      description: "Generate a short summary with a configured Beaver model.",
      parameters: { type: "object", properties: { prompt: { type: "string" } }, required: ["prompt"] },
      effect: "external-read",
      async execute({ prompt }) {
        return JSON.stringify(await beaver.models!.generate({ prompt: String(prompt) }));
      },
    });
  }

  if (supports("memory") && beaver.memory) {
    beaver.registerTool({
      name: "remember_topic",
      description: "Create a bounded topic in Beaver memory.",
      parameters: { type: "object", properties: { content: { type: "string" } }, required: ["content"] },
      effect: "local-write",
      async execute({ content }) {
        return JSON.stringify(await beaver.memory!.write({ scope: "global", content: String(content) }));
      },
    });
  }

  if (supports("automations") && beaver.automations) {
    beaver.registerTool({
      name: "propose_wakeup",
      description: "Create an inactive wakeup that the user must approve in Beaver.",
      parameters: { type: "object", properties: { prompt: { type: "string" } }, required: ["prompt"] },
      effect: "local-write",
      async execute({ prompt }) {
        const automation = await beaver.automations!.create({
          name: "Extension follow-up",
          prompt: String(prompt),
          schedule: { kind: "after_completion", delay_minutes: 10 },
        });
        return JSON.stringify({ ...automation, requiresUserApproval: true });
      },
    });
  }

  if (supports("subagents") && beaver.subagents) {
    beaver.registerTool({
      name: "start_explorer",
      description: "Start and return an attributed explorer child.",
      parameters: { type: "object", properties: { prompt: { type: "string" } }, required: ["prompt"] },
      effect: "process",
      async execute({ prompt }) {
        return JSON.stringify(await beaver.subagents!.spawn("explorer", String(prompt)));
      },
    });
  }

  beaver.registerTool({
    name: "observed_turns",
    description: "Return the number of completed turns observed by this extension.",
    parameters: { type: "object" },
    effect: "read-only",
    execute: () => String(completedTurns),
  });

  if (supports("toolInterception") && beaver.interceptTool) {
    beaver.interceptTool((call) => call.effect === "process"
      ? { decision: "deny", reason: "example_process_guard" }
      : { decision: "continue" });
  }
});
