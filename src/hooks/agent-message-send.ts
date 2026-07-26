import { invoke } from "@tauri-apps/api/core";
import {
  createUserMessage,
  pendingFilesToAttachments,
  type PendingChatFile,
} from "./agent-message-builders";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import type { AgentMessage } from "@/types/agent";

export interface AgentSendPayload {
  text: string;
  sentFiles?: PendingChatFile[];
  workingDir?: string;
  projectId?: string;
  skills?: { name: string; content: string }[];
}

interface PersistAgentMessageOptions extends AgentSendPayload {
  sessionId: string;
  messages: AgentMessage[];
  permissionMode?: string;
  doStream: (
    llmMessages: AgentMessage[],
    displayMessages: AgentMessage[],
    sessionId: string,
    workingDir?: string,
    baseTokenCount?: number,
    permissionMode?: string,
  ) => Promise<void>;
  queueStreamMessage?: (
    sessionId: string,
    messages: AgentMessage[],
    displayMessage: AgentMessage,
  ) => Promise<boolean>;
}

export async function persistAgentMessage(options: PersistAgentMessageOptions) {
  if (options.projectId && options.messages.length === 0) {
    const session = await invoke<Record<string, unknown>>("get_agent_session", {
      id: options.sessionId,
    });
    if (!session.project_id) {
      session.project_id = options.projectId;
      await invoke("save_agent_session", { session }).catch(() => {
        showToast(i18n.t("errors.sessionSaveFailed"), "error");
      });
    }
  }
  const files = pendingFilesToAttachments(options.sentFiles);
  const skillNames = options.skills?.map((skill) => skill.name);
  const userMessage = createUserMessage(options.text || "", files, skillNames);
  const displayMessages = [...options.messages, userMessage];
  const queuedLlmMessages: AgentMessage[] = [];
  for (const skill of options.skills ?? []) {
    queuedLlmMessages.push({
      id: `skill-${crypto.randomUUID()}`,
      role: "user",
      content: `The user has loaded the following skill. Follow its instructions exactly:\n\n${skill.content}`,
      files: [],
      timestamp: new Date().toISOString(),
    });
  }
  queuedLlmMessages.push(userMessage);
  if (await options.queueStreamMessage?.(
    options.sessionId,
    queuedLlmMessages,
    userMessage,
  )) return;
  const llmMessages = [...options.messages, ...queuedLlmMessages];
  await invoke("add_messages_to_session", {
    id: options.sessionId,
    messages: [userMessage],
    tokens: 0,
  }).catch(() => showToast(i18n.t("errors.sessionSaveFailed"), "error"));
  await options.doStream(
    llmMessages,
    displayMessages,
    options.sessionId,
    options.workingDir,
    undefined,
    options.permissionMode,
  );
}
