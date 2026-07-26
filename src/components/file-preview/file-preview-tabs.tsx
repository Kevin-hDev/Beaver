import { useState } from "react";
import { detectEditorsForFile } from "@/services/file-preview";
import type { FileOperation, FilePreviewActiveTab, FilePreviewListMode, PreviewEditor } from "@/types/file-preview";
import { FileTabMenu } from "./file-tab-menu";
import { FilePreviewTab } from "./file-preview-tab";
import { FilePreviewSummarySelector } from "./file-preview-summary-selector";

interface FilePreviewTabsProps {
  tabs: FileOperation[];
  activeTab: FilePreviewActiveTab;
  listMode: FilePreviewListMode;
  baseDir?: string;
  onSelect: (id: FilePreviewActiveTab) => void;
  onListModeChange: (mode: FilePreviewListMode) => void;
  onClose: (id: string) => void;
  onOpenDefault: (operation: FileOperation) => void;
  onOpenWith: (operation: FileOperation, editorPath: string) => void;
}

export function FilePreviewTabs({
  tabs,
  activeTab,
  listMode,
  baseDir,
  onSelect,
  onListModeChange,
  onClose,
  onOpenDefault,
  onOpenWith,
}: FilePreviewTabsProps) {
  const [menu, setMenu] = useState<{
    x: number;
    y: number;
    operation: FileOperation;
    editors: PreviewEditor[];
  } | null>(null);

  const handleContextMenu = (event: React.MouseEvent, tab: FileOperation) => {
    event.preventDefault();
    event.stopPropagation();
    if (tab.source) return;
    detectEditorsForFile(tab.path, baseDir)
      .then((editors) => {
        setMenu({ x: event.clientX, y: event.clientY, operation: tab, editors });
      })
      .catch(() => {
        setMenu({ x: event.clientX, y: event.clientY, operation: tab, editors: [] });
      });
  };

  return (
    <div className="fp-tabs">
      <FilePreviewSummarySelector
        active={activeTab === "summary"}
        mode={listMode}
        onSelect={() => onSelect("summary")}
        onModeChange={onListModeChange}
      />
      <div className="fp-tabs-files">
        {tabs.map((tab) => (
          <FilePreviewTab
            key={tab.id}
            operation={tab}
            label={tab.name}
            active={activeTab === tab.id}
            onSelect={() => onSelect(tab.id)}
            onClose={() => onClose(tab.id)}
            onContextMenu={(event) => handleContextMenu(event, tab)}
          />
        ))}
      </div>
      {menu && (
        <FileTabMenu
          x={menu.x}
          y={menu.y}
          editors={menu.editors}
          onOpen={() => {
            onOpenDefault(menu.operation);
            setMenu(null);
          }}
          onOpenWith={(editorPath) => {
            onOpenWith(menu.operation, editorPath);
            setMenu(null);
          }}
          onClose={() => setMenu(null)}
        />
      )}
    </div>
  );
}
