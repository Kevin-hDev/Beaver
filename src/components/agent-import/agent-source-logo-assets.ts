import agentsLogo from "@/assets/agent-import/agents.svg";
import claudePage1 from "@/assets/agent-import/claude-page1.svg";
import claudePage2 from "@/assets/agent-import/claude-page2.svg";
import codexPage1 from "@/assets/agent-import/codex-page1.png";
import codexPage2 from "@/assets/agent-import/codex-page2.svg";
import hermesPage1Dark from "@/assets/agent-import/hermes-page1-dark.png";
import hermesPage1Light from "@/assets/agent-import/hermes-page1-light.png";
import hermesPage2 from "@/assets/agent-import/hermes-page2.svg";
import kimiPage1 from "@/assets/agent-import/kimi-page1.svg";
import kimiPage2 from "@/assets/agent-import/kimi-page2.svg";
import openClawPage1 from "@/assets/agent-import/openclaw-page1.svg";
import openClawPage2 from "@/assets/agent-import/openclaw-page2.svg";
import openCodePage1 from "@/assets/agent-import/opencode-page1.svg";
import openCodePage2 from "@/assets/agent-import/opencode-page2.svg";
import qwenPage1 from "@/assets/agent-import/qwen-page1.svg";
import qwenPage2 from "@/assets/agent-import/qwen-page2.svg";
import zcodePage1 from "@/assets/agent-import/zcode-page1.svg";
import zcodePage2 from "@/assets/agent-import/zcode-page2.svg";

export type AgentLogoAsset =
  | { kind: "color"; src: string; ratio?: number }
  | { kind: "mono"; src: string; ratio?: number }
  | {
    kind: "themed";
    lightSrc: string;
    darkSrc: string;
    ratio?: number;
  };

interface AgentLogoPair {
  card: AgentLogoAsset;
  detail: AgentLogoAsset;
}

export const AGENT_SOURCE_LOGOS: Record<string, AgentLogoPair> = {
  claude: {
    card: { kind: "color", src: claudePage1 },
    detail: { kind: "color", src: claudePage2 },
  },
  codex: {
    card: { kind: "color", src: codexPage1 },
    detail: { kind: "mono", src: codexPage2, ratio: 4.06 },
  },
  agents: {
    card: { kind: "mono", src: agentsLogo },
    detail: { kind: "mono", src: agentsLogo },
  },
  hermes: {
    card: {
      kind: "themed",
      lightSrc: hermesPage1Light,
      darkSrc: hermesPage1Dark,
    },
    detail: { kind: "mono", src: hermesPage2, ratio: 2.17 },
  },
  qwen: {
    card: { kind: "color", src: qwenPage1 },
    detail: { kind: "color", src: qwenPage2, ratio: 3.42 },
  },
  zcode: {
    card: { kind: "mono", src: zcodePage1 },
    detail: { kind: "mono", src: zcodePage2, ratio: 2.08 },
  },
  openclaw: {
    card: { kind: "color", src: openClawPage1 },
    detail: { kind: "mono", src: openClawPage2, ratio: 5.83 },
  },
  opencode: {
    card: { kind: "color", src: openCodePage1 },
    detail: { kind: "mono", src: openCodePage2 },
  },
  kimi: {
    card: { kind: "mono", src: kimiPage1 },
    detail: { kind: "mono", src: kimiPage2, ratio: 6.02 },
  },
};
