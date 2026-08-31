import { useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ChatInputActionsRow } from "./chat-input-actions-row";
import { ChatInputEditor } from "./chat-input-editor";
import { ErrorBubble } from "./error-bubble";
import { InteractiveChoicePanel } from "./interactive-choice-panel";
import { useInteractiveChoiceFeedback } from "./use-interactive-choice-feedback";
import { useSlashCommands } from "@/hooks/use-slash-commands";
import { useActiveSkills } from "@/hooks/use-active-skills";
import { SlashAutocomplete } from "./slash-autocomplete";
import { FileThumbnail } from "./file-thumbnail";
import { useStopConfirmation } from "./use-stop-confirmation";
import type { ChatInputProps } from "./chat-input-types";
import { useComposerDraft } from "@/hooks/use-composer-draft";
import { sameChatFiles } from "./chat-input-snapshot";
import { matchesAppShortcut } from "@/lib/app-shortcuts";
import "./chat.css";
import "./chat-input-textarea.css";
import "./chat-input-responsive.css";

const K_UP = "ArrowUp";
const K_DOWN = "ArrowDown";
const K_ESC = "Escape";

export function ChatInput({
  draftKey, sessionId,
  modelName, providerName, isStreaming, reasoningMode, fastModeEnabled, fastModePending, files,
  contextUsed, contextMax, contextBreakdown, retryIndicator,
  interactiveRequest, onInteractiveResolved,
  permissionMode, availablePermissionModes, missingDirectory, missingDirectoryResolving,
  planModeEnabled = false, onPermissionModeChange, onResolveMissingDirectory, onPlanModeChange,
  onSend, onStop, onFileImport, onModelChange, onReasoningModeChange, onFastModeChange,
  onRemoveFile, onPreviewFile, onClearFiles,
}: ChatInputProps) {
  const { t } = useTranslation();
  const {
    text,
    skills: draftSkills,
    setText,
    rememberSkill,
    consume: consumeDraft,
    restore: restoreDraft,
  } = useComposerDraft(draftKey);
  const slash = useSlashCommands();
  const skills = useActiveSkills(
    slash,
    text,
    setText,
    draftSkills,
    rememberSkill,
  );
  const bubbleRef = useRef<HTMLDivElement>(null);
  const sendingRef = useRef(false);
  const filesRef = useRef(files);
  // eslint-disable-next-line react-hooks/refs -- latest props guard async snapshot cleanup
  filesRef.current = files;
  const { isConfirmingStop, requestStop, stopNow } = useStopConfirmation(isStreaming, onStop);

  const interactivePending = !!interactiveRequest;
  const interactiveFeedback = useInteractiveChoiceFeedback(
    interactiveRequest,
    onInteractiveResolved,
  );
  const hasText = text.trim().length > 0;
  const hasFiles = files != null && files.length > 0;
  const hasContent = hasText || hasFiles;

  const handleSend = useCallback(async () => {
    if (!hasContent || interactivePending || sendingRef.current) return;
    const sentDraft = { text, skills: [...draftSkills] };
    const sentFiles = files?.map((file) => ({ ...file }));
    const consumedOptimistically = !isStreaming;
    sendingRef.current = true;
    if (consumedOptimistically) consumeDraft(sentDraft);
    try {
      const accepted = await onSend(
        text.trim(), hasFiles ? files : undefined, skills.getSkillsPayload(),
      );
      if (accepted === false) {
        if (consumedOptimistically) restoreDraft(sentDraft);
        return;
      }
      if (!consumedOptimistically) consumeDraft(sentDraft);
      if (sameChatFiles(filesRef.current, sentFiles)) onClearFiles?.();
    } catch (error) {
      if (consumedOptimistically) restoreDraft(sentDraft);
      throw error;
    } finally {
      sendingRef.current = false;
    }
  }, [text, draftSkills, hasContent, hasFiles, files, skills, interactivePending, isStreaming, onSend, onClearFiles, consumeDraft, restoreDraft]);

  const handleChange = useCallback((value: string, cursorPos: number) => {
    setText(value);
    slash.handleInput(value, cursorPos);
  }, [setText, slash]);

  // Shared Enter logic. The editor gives the four chat control keys priority
  // only when this handler consumes them.
  const handleEnter = useCallback((): boolean => {
    if (slash.showDropdown) {
      const selected = slash.skills[slash.activeIndex];
      if (selected) void skills.handleSelectSkill(selected);
      return true;
    }
    void handleSend();
    return true;
  }, [handleSend, slash.showDropdown, slash.skills, slash.activeIndex, skills]);

  const handleKeyEvent = useCallback((event: KeyboardEvent): boolean | void => {
    const pressed = event.key;
    if (slash.showDropdown) {
      if (pressed === K_UP) { event.preventDefault(); slash.moveUp(); return true; }
      if (pressed === K_DOWN) { event.preventDefault(); slash.moveDown(); return true; }
      if (matchesAppShortcut(event, "sendMessage")) {
        event.preventDefault();
        return handleEnter();
      }
      if (pressed === K_ESC) { event.preventDefault(); slash.close(); return true; }
    }
    if (matchesAppShortcut(event, "sendMessage")) {
      event.preventDefault();
      return handleEnter();
    }
    if (matchesAppShortcut(event, "stopResponse") && isStreaming) {
      event.preventDefault();
      event.stopPropagation();
      requestStop();
      return true;
    }
  }, [handleEnter, isStreaming, requestStop, slash]);

  useEffect(() => {
    if (!isStreaming || interactivePending) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key !== K_ESC) return;
      e.preventDefault();
      requestStop();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [interactivePending, isStreaming, requestStop]);

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
    <>
      {interactiveFeedback.error && <ErrorBubble message={interactiveFeedback.error} />}
      <div className={`chat-input-bubble relief elev-float${interactivePending ? " chat-input-bubble-interactive" : ""}`} ref={bubbleRef}>
      {interactivePending ? (
        <InteractiveChoicePanel
          request={interactiveRequest ?? undefined}
          onResolved={interactiveFeedback.resolve}
          onError={interactiveFeedback.fail}
        />
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
            inputBubbleRef={bubbleRef}
            sessionId={sessionId}
            modelName={modelName}
            providerName={providerName}
            reasoningMode={reasoningMode}
            fastModeEnabled={fastModeEnabled}
            fastModePending={fastModePending}
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
            onFastModeChange={onFastModeChange}
            onSend={() => { void handleSend(); }}
            onStop={stopNow}
          />
        </>
      )}
      </div>
    </>
  );
}
