import { useState, useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ChatInputActionsRow } from "./chat-input-actions-row";
import { ChatInputEditor } from "./chat-input-editor";
import { InteractiveChoicePanel } from "./interactive-choice-panel";
import { useSlashCommands } from "@/hooks/use-slash-commands";
import { useActiveSkills } from "@/hooks/use-active-skills";
import { SlashAutocomplete } from "./slash-autocomplete";
import { FileThumbnail } from "./file-thumbnail";
import { useStopConfirmation } from "./use-stop-confirmation";
import type { ChatInputProps } from "./chat-input-types";
import "./chat.css";
import "./chat-input-textarea.css";
import "./chat-input-responsive.css";

const K_UP = "ArrowUp";
const K_DOWN = "ArrowDown";
const K_ENTER = "Enter";
const K_ESC = "Escape";

export function ChatInput({
  modelName, providerName, isStreaming, reasoningMode, files,
  contextUsed, contextMax, contextBreakdown, retryIndicator,
  interactiveRequest, onInteractiveResolved,
  permissionMode, availablePermissionModes, missingDirectory, missingDirectoryResolving,
  planModeEnabled = false, onPermissionModeChange, onResolveMissingDirectory, onPlanModeChange,
  onSend, onStop, onFileImport, onModelChange, onReasoningModeChange,
  onRemoveFile, onPreviewFile, onClearFiles,
}: ChatInputProps) {
  const { t } = useTranslation();
  const [text, setText] = useState("");
  const slash = useSlashCommands();
  const skills = useActiveSkills(slash, text, setText);
  const bubbleRef = useRef<HTMLDivElement>(null);
  const { isConfirmingStop, requestStop } = useStopConfirmation(isStreaming, onStop);

  const interactivePending = !!interactiveRequest;
  const hasText = text.trim().length > 0;
  const hasFiles = files != null && files.length > 0;
  const hasContent = hasText || hasFiles;

  const handleSend = useCallback(() => {
    if (!hasContent || interactivePending) return;
    onSend(text.trim(), hasFiles ? files : undefined, skills.getSkillsPayload());
    setText("");
    skills.clearSkills();
    onClearFiles?.();
  }, [text, hasContent, hasFiles, files, skills, interactivePending, onSend, onClearFiles]);

  const handleChange = useCallback((value: string, cursorPos: number) => {
    setText(value);
    slash.handleInput(value, cursorPos);
  }, [slash]);

  // Shared Enter logic. The editor gives the four chat control keys priority
  // only when this handler consumes them.
  const handleEnter = useCallback((): boolean => {
    if (slash.showDropdown) {
      const selected = slash.skills[slash.activeIndex];
      if (selected) void skills.handleSelectSkill(selected);
      return true;
    }
    handleSend();
    return true;
  }, [handleSend, slash.showDropdown, slash.skills, slash.activeIndex, skills]);

  const handleKeyEvent = useCallback((event: KeyboardEvent): boolean | void => {
    const pressed = event.key;
    if (slash.showDropdown) {
      if (pressed === K_UP) { event.preventDefault(); slash.moveUp(); return true; }
      if (pressed === K_DOWN) { event.preventDefault(); slash.moveDown(); return true; }
      if (pressed === K_ENTER && !event.shiftKey) {
        event.preventDefault();
        return handleEnter();
      }
      if (pressed === K_ESC) { event.preventDefault(); slash.close(); return true; }
    }
    if (pressed === K_ENTER && !event.shiftKey) {
      event.preventDefault();
      return handleEnter();
    }
    if (pressed === K_ESC && isStreaming) {
      event.preventDefault();
      event.stopPropagation();
      requestStop();
      return true;
    }
  }, [handleEnter, isStreaming, requestStop, slash]);

  useEffect(() => {
    if (!isStreaming) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key !== K_ESC) return;
      e.preventDefault();
      requestStop();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [isStreaming, requestStop]);

  useEffect(() => {
    if (!slash.showDropdown) return;
    const handler = (e: MouseEvent) => {
      if (bubbleRef.current && !bubbleRef.current.contains(e.target as Node)) slash.close();
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [slash.showDropdown, slash]);

  const buttonState = hasContent && !interactivePending ? "send" as const
    : isStreaming ? (isConfirmingStop ? "confirmStop" as const : "stop" as const)
    : "hidden" as const;

  return (
    <div className={`chat-input-bubble${interactivePending ? " chat-input-bubble-interactive" : ""}`} ref={bubbleRef}>
      {interactivePending ? (
        <InteractiveChoicePanel request={interactiveRequest ?? undefined} onResolved={onInteractiveResolved} />
      ) : (
        <>
          {slash.showDropdown && (
            <SlashAutocomplete
              skills={slash.skills}
              activeIndex={slash.activeIndex}
              onSelect={(s) => void skills.handleSelectSkill(s)}
            />
          )}
          <ChatInputEditor
            value={text}
            placeholder={t("agentLocal.placeholder")}
            readOnly={false}
            activeSkills={skills.activeSkills}
            onTextChange={handleChange}
            onKeyEvent={handleKeyEvent}
          />
          {files && files.length > 0 && (
            <div className="chat-file-list">
              {files.map((f, i) => (
                <FileThumbnail
                  key={`${f.name}-${i}`}
                  file={f}
                  onRemove={() => onRemoveFile?.(i)}
                  onClick={() => onPreviewFile?.(f)}
                />
              ))}
            </div>
          )}
          <ChatInputActionsRow
            modelName={modelName}
            providerName={providerName}
            reasoningMode={reasoningMode}
            contextUsed={contextUsed}
            contextMax={contextMax}
            contextBreakdown={contextBreakdown}
            permissionMode={permissionMode}
            availablePermissionModes={availablePermissionModes}
            missingDirectory={missingDirectory}
            missingDirectoryResolving={missingDirectoryResolving}
            planModeEnabled={planModeEnabled}
            retryIndicator={retryIndicator}
            buttonState={buttonState}
            onPermissionModeChange={onPermissionModeChange}
            onResolveMissingDirectory={onResolveMissingDirectory}
            onPlanModeChange={onPlanModeChange}
            onFileImport={onFileImport}
            onModelChange={onModelChange}
            onReasoningModeChange={onReasoningModeChange}
            onSend={handleSend}
            onStop={requestStop}
          />
        </>
      )}
    </div>
  );
}
