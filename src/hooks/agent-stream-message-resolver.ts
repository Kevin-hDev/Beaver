import { readFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import {
  expandSegmentsToChat,
  expandToolActivities,
  type ChatMsg,
} from "./agent-chat-utils";
import type { AgentMessage, FileAttachment } from "@/types/agent";

const IMAGE_EXTS = ["png", "jpg", "jpeg", "gif", "webp"];
const TEXT_EXTS = [
  "md", "txt", "ts", "tsx", "js", "jsx", "json", "yaml", "yml", "toml", "rs", "py",
  "sh", "css", "html", "xml", "csv", "sql", "env", "cfg", "conf", "ini", "log", "svelte", "vue",
];
const MAX_TEXT_CHARS_PER_FILE = 120_000;
const MAX_TEXT_CHARS_PER_MESSAGE = 300_000;
const MAX_TOOL_CALLS_PER_TURN = 64;

export async function resolveAgentStreamMessages(
  messages: AgentMessage[],
): Promise<ChatMsg[]> {
  const resolved = await Promise.all(messages.map(resolveMessage));
  const flattened = resolved.flatMap(({ message, content, images }) => {
    if (message.role === "assistant" && message.segments?.length && !message.tool_calls) {
      return expandSegmentsToChat(message.segments, content);
    }
    if (message.role === "assistant" && message.tool_activities?.length && !message.tool_calls) {
      return expandToolActivities(message.tool_activities, content);
    }
    return [{
      role: message.role, content, images,
      tool_calls: message.tool_calls ?? null, tool_name: message.tool_name ?? null,
    }];
  });
  return linkToolCallsToResults(flattened);
}

function linkToolCallsToResults(messages: ChatMsg[]): ChatMsg[] {
  let pendingIds: string[] = [];
  const linked = messages.map((message) => {
    if (message.role === "assistant" && message.tool_calls?.length) {
      if (pendingIds.length > 0 || message.tool_calls.length > MAX_TOOL_CALLS_PER_TURN) {
        throw new Error("Invalid tool call history.");
      }
      const toolCalls = message.tool_calls.map((call) => ({
        ...call,
        id: crypto.randomUUID(),
      }));
      pendingIds = toolCalls.map((call) => call.id);
      return { ...message, tool_calls: toolCalls };
    }
    if (message.role === "tool") {
      const toolCallId = pendingIds.shift();
      if (!toolCallId) throw new Error("Invalid tool result history.");
      return { ...message, tool_call_id: toolCallId };
    }
    if (pendingIds.length > 0) throw new Error("Incomplete tool result history.");
    return message;
  });
  if (pendingIds.length > 0) throw new Error("Incomplete tool result history.");
  return linked;
}

async function resolveMessage(message: AgentMessage) {
  let images: string[] | null = null;
  let content = message.content;
  let remainingTextChars = MAX_TEXT_CHARS_PER_MESSAGE;
  const imageFiles = message.files?.filter((file) => hasExtension(file.name, IMAGE_EXTS)) ?? [];
  if (imageFiles.length > 0) {
    const loaded = await Promise.all(imageFiles.map(imageAttachmentToBase64));
    images = loaded.filter((image): image is string => image !== null);
    if (images.length === 0) images = null;
  }
  const textFiles = message.files?.filter((file) => file.path && hasExtension(file.name, TEXT_EXTS)) ?? [];
  for (const file of textFiles) {
    if (!await restoreAttachmentAccess(file)) continue;
    try {
      const text = new TextDecoder().decode(await readFile(file.path));
      const allowed = Math.min(MAX_TEXT_CHARS_PER_FILE, remainingTextChars);
      const body = allowed > 0
        ? clipTextForContext(text, allowed)
        : "[File omitted: text attachment budget reached]";
      remainingTextChars = Math.max(0, remainingTextChars - allowed);
      content += `\n\n--- Fichier: ${file.name} ---\n${body}`;
    } catch { /* Erreur volontairement masquée. */ }
  }
  return { message, content, images };
}

function hasExtension(name: string, allowed: string[]): boolean {
  return allowed.includes(name.split(".").pop()?.toLowerCase() ?? "");
}

async function imageAttachmentToBase64(file: FileAttachment): Promise<string | null> {
  if (file.path && await restoreAttachmentAccess(file)) {
    try {
      return uint8ToBase64(await readFile(file.path));
    } catch {
      return thumbnailToBase64(file.thumbnail);
    }
  }
  return thumbnailToBase64(file.thumbnail);
}

async function restoreAttachmentAccess(file: FileAttachment): Promise<boolean> {
  if (!file.path || !file.access_grant) return false;
  try {
    await invoke("restore_attachment_access", {
      path: file.path,
      accessGrant: file.access_grant,
    });
    return true;
  } catch {
    return false;
  }
}

function uint8ToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function thumbnailToBase64(thumbnail?: string): string | null {
  const marker = ";base64,";
  const markerIndex = thumbnail?.indexOf(marker) ?? -1;
  if (!thumbnail?.startsWith("data:image/") || markerIndex < 0) return null;
  return thumbnail.slice(markerIndex + marker.length).trim() || null;
}

function clipTextForContext(text: string, maxChars: number): string {
  const chars = Array.from(text);
  if (chars.length <= maxChars) return text;
  return `${chars.slice(0, maxChars).join("")}\n[File truncated: use read_file if full content is needed]`;
}
