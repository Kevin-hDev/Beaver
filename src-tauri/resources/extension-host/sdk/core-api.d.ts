import type { JsonValue } from "./index";
import type { ModelFinishReason } from "./contract";

export type BeaverPage<T> = {
  items: T[];
  nextCursor?: string;
  incomplete?: boolean;
};

export interface BeaverModelDescriptor {
  connectionId: string;
  canonicalProvider: string;
  transportFamily: string;
  modelId: string;
  name: string;
  generationSupported: boolean;
}

export interface BeaverModelGeneration {
  text: string;
  finishReason: ModelFinishReason;
  usage: { inputTokens?: number; outputTokens?: number; ledgerRecorded: boolean };
}

export interface BeaverModelsApi {
  list(options?: { connectionId?: string; cursor?: string }): Promise<BeaverPage<BeaverModelDescriptor>>;
  generate(options: {
    prompt: string;
    connectionId?: string;
    modelId?: string;
    maxOutputTokens?: number;
  }): Promise<BeaverModelGeneration>;
}

export type BeaverMemoryScope = "global" | "project";
export interface BeaverMemoryTopicSummary {
  id: string;
  title: string;
  updatedAt: string;
}
export interface BeaverMemoryTopic extends BeaverMemoryTopicSummary {
  content: string;
}
export interface BeaverMemoryMutation {
  topic: BeaverMemoryTopic;
  applied: boolean;
  indexUpdated: boolean;
}
export interface BeaverMemoryApi {
  list(options: { scope: BeaverMemoryScope; cursor?: string }): Promise<BeaverPage<BeaverMemoryTopicSummary>>;
  read(options: { scope: BeaverMemoryScope; topicId: string }): Promise<BeaverMemoryTopic>;
  write(options: {
    scope: BeaverMemoryScope;
    topicId?: string;
    expectedUpdatedAt?: string;
    content: string;
  }): Promise<BeaverMemoryMutation>;
  archive(options: { scope: BeaverMemoryScope; topicId: string }): Promise<BeaverMemoryMutation>;
}

export interface BeaverAutomation {
  id: string;
  revision: number;
  name: string;
  description: string | null;
  prompt: string;
  schedule: JsonValue;
  active: boolean;
}
export interface BeaverAutomationsApi {
  list(options?: { cursor?: string }): Promise<BeaverPage<BeaverAutomation>>;
  create(definition: {
    name: string;
    description?: string;
    prompt: string;
    schedule: JsonValue;
  }): Promise<BeaverAutomation>;
  update(automationId: string, revision: number, patch: {
    name?: string;
    description?: string;
    prompt?: string;
    schedule?: JsonValue;
  }): Promise<BeaverAutomation>;
  setActive(automationId: string, revision: number, active: boolean): Promise<BeaverAutomation>;
  delete(automationId: string, revision: number): Promise<{ deleted: boolean }>;
}

export type BeaverSubagentType = "explorer" | "coder";
export interface BeaverSubagent {
  id: string;
  type: BeaverSubagentType;
  status: string;
  report?: string;
}
export interface BeaverSubagentsApi {
  spawn(type: BeaverSubagentType, prompt: string): Promise<BeaverSubagent>;
  list(options?: { cursor?: string }): Promise<BeaverPage<BeaverSubagent>>;
  get(subagentId: string): Promise<BeaverSubagent>;
  send(subagentId: string, prompt: string): Promise<{ accepted: boolean }>;
  cancel(subagentId: string): Promise<{ accepted: boolean; stopped: boolean }>;
}

export interface BeaverToolInterception {
  toolName: string;
  effect: string;
}
export type BeaverToolInterceptionDecision =
  | { decision: "continue" }
  | { decision: "deny"; reason?: string };
export type BeaverToolInterceptor = (
  call: BeaverToolInterception,
) => BeaverToolInterceptionDecision | Promise<BeaverToolInterceptionDecision>;
