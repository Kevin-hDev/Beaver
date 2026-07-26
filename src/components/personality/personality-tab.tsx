import { useState, useEffect, useCallback, useMemo, memo } from "react";
import { useTranslation } from "react-i18next";
import type { PersonalityFile } from "@/types/personality";
import * as api from "@/services/personality";
import { useFsEvent } from "@/hooks/use-fs-event";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import { showToast } from "@/lib/toast-emitter";
import { PersonalityList } from "./personality-list";
import { MarkdownViewer } from "./markdown-viewer";
import { PanelSlot } from "@/components/layout/panel-slots";

interface PersonalityTabProps {
  activePath?: string | null;
  onPathChange?: (path: string | null) => void;
  listFocused?: boolean;
}

export const PersonalityTab = memo(function PersonalityTab({
  activePath,
  onPathChange,
  listFocused = true,
}: PersonalityTabProps) {
  const { t } = useTranslation();
  const [files, setFiles] = useState<PersonalityFile[]>([]);
  const [selectedPath, setSelectedPathState] = useState<string | null>(null);

  useEffect(() => {
    if (activePath !== undefined) setSelectedPathState(activePath);
  }, [activePath]);
  const [content, setContent] = useState<string>("");
  const [fileName, setFileName] = useState<string>("");
  const [injectionState, setInjectionState] = useState<Record<string, boolean>>({});

  const loadFiles = useCallback(() => {
    api.listFiles().then(setFiles).catch(() => showToast(t("personality.failedToLoad")));
  }, [t]);

  const loadInjectionState = useCallback(() => {
    api.getInjectionState().then(setInjectionState).catch(() => {});
  }, []);

  useEffect(() => {
    loadFiles();
    loadInjectionState();
  }, [loadFiles, loadInjectionState]);

  useFsEvent("fs:personality-changed", loadFiles);

  const reloadContent = useCallback(() => {
    if (selectedPath) {
      api.readFile(selectedPath).then(setContent).catch(() => showToast(t("personality.failedToLoad")));
    }
  }, [selectedPath, t]);
  useFsEvent("fs:personality-changed", reloadContent);

  const setSelectedPath = useCallback((path: string | null) => {
    setSelectedPathState(path);
    onPathChange?.(path);
  // eslint-disable-next-line react-hooks/exhaustive-deps -- onPathChange identity is stable per caller
  }, []);

  const handleSelect = useCallback(async (path: string) => {
    setSelectedPath(path);
    try {
      const text = await api.readFile(path);
      setContent(text);
      setFileName(path.split(/[\\/]/).pop() ?? "");
    } catch {
      showToast(t("personality.failedToRead"));
      setContent(t("errors.readError"));
    }
  }, [setSelectedPath, t]);

  useEffect(() => {
    if (!selectedPath && files.length > 0) {
      void handleSelect(files[0].path);
    }
  }, [files, selectedPath, handleSelect]);

  const handleOpen = useCallback(() => {
    if (selectedPath) {
      api.openInEditor(selectedPath).catch(() => showToast(t("personality.failedToLoad")));
    }
  }, [selectedPath, t]);

  const handleToggleInjection = useCallback(async (enabled: boolean) => {
    if (fileName === "AGENTS.md") return;
    try {
      await api.setInjectionState(fileName, enabled);
      setInjectionState((prev) => ({ ...prev, [fileName]: enabled }));
    } catch {
      showToast(t("personality.failedToLoad"));
    }
  }, [fileName, t]);

  const filePaths = useMemo(() => files.map((f) => f.path), [files]);
  useArrowNavigation({
    items: filePaths,
    selectedId: selectedPath,
    onSelect: (path) => void handleSelect(path),
    enabled: listFocused,
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const list = useMemo(() => (
    <PersonalityList
      files={files}
      selectedPath={selectedPath}
      injectionState={injectionState}
      selectedFileName={fileName}
      onSelect={(path) => void handleSelect(path)}
      onToggleInjection={(enabled) => void handleToggleInjection(enabled)}
    />
  ), [fileName, files, handleSelect, handleToggleInjection, injectionState, selectedPath]);

  const detail = useMemo(() => {
    if (!selectedPath) return (
      <div style={{ padding: "var(--space-lg)", color: "var(--ink-faint)" }}>
        {t("personality.selectFile")}
      </div>
    );
    return (
      <MarkdownViewer
        content={content}
        fileName={fileName}
        onOpenEditor={handleOpen}
      />
    );
  }, [content, fileName, handleOpen, selectedPath, t]);

  return <><PanelSlot name="list">{list}</PanelSlot><PanelSlot name="detail">{detail}</PanelSlot></>;
});
