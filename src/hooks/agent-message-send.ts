import { invoke } from "@tauri-apps/api/core";
import {
  createUserMessage,
  pendingFilesToAttachments,
  type PendingChatFile,
} from "./agent-message-builders";
import { showToast } from "@/lib/toast-emitter";
import { admissionErrorMessage } from "@/lib/admission-error";
import i18n from "@/i18n";
import type { AgentMessage } from "@/types/agent";
import type {
  NewUserTurnInput,
  SkillReference,
  TurnStart,
} from "@/types/agent-turn.generated";

export interface AgentSendPayload {
  text: string;
  sentFiles?: PendingChatFile[];
  workingDir?: string;
  projectId?: string;
  skills?: SkillReference[];
}

interface PersistAgentMessageOptions extends AgentSendPayload {
  sessionId: string;
  messages: AgentMessage[];
  permissionMode?: string;
  doStream: (
    turn: TurnStart,
    displayMessages: AgentMessage[],
    sessionId: string,
    workingDir?: string,
    baseTokenCount?: number,
    permissionMode?: string,
    optimisticUserMessageId?: string,
  ) => Promise<void>;
  queueStreamMessage?: (
    sessionId: string,
    input: NewUserTurnInput,
    displayMessage: AgentMessage,
  ) => Promise<boolean>;
}

export async function persistAgentMessage(options: PersistAgentMessageOptions) {
  if (options.projectId && options.messages.length === 0) {
    try {
      await invoke("update_session_project", {
        id: options.sessionId,
        projectId: options.projectId,
      });
    } catch (error) {
      showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
      return;
    }
  }
  const files = pendingFilesToAttachments(options.sentFiles);
  const skillNames = options.skills?.flatMap((skill) => skill.name ? [skill.name] : []);
  const userMessage = createUserMessage(options.text || "", files, skillNames);
  const displayMessages = [...options.messages, userMessage];
  const input: NewUserTurnInput = {
    content: options.text || "",
    files,
    skills: options.skills ?? [],
  };
  if (await options.queueStreamMessage?.(
    options.sessionId,
    input,
    userMessage,
  )) return;
  await options.doStream(
    { type: "new", input },
    displayMessages,
    options.sessionId,
    options.workingDir,
    undefined,
    options.permissionMode,
    userMessage.id,
  );
}
