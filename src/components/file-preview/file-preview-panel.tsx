import { useTranslation } from "react-i18next";
import { Maximize2, Minimize2, FolderTree } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { openPreviewFile, openPreviewWithEditor } from "@/services/file-preview";
import type { FileOperation, FilePreviewActiveTab, FilePreviewListMode } from "@/types/file-preview";
import { shouldWrapFile } from "@/lib/code-language";
import { FilePreviewBreadcrumb } from "./file-preview-breadcrumb";
import { FilePreviewContent } from "./file-preview-content";
import { FilePreviewPlan } from "./file-preview-plan";
import { FilePreviewSummary } from "./file-preview-summary";
import { FilePreviewTabs } from "./file-preview-tabs";
import "./file-preview-panel.css";
import "./file-preview-tabs.css";
import "./file-preview-highlight.css";

interface FilePreviewPanelProps {
  fullscreen: boolean;
  allOperations: FileOperation[];
  latestOperations: FileOperation[];
  uncommittedOperations: FileOperation[];
  tabs: FileOperation[];
  activeTab: FilePreviewActiveTab;
  listMode: FilePreviewListMode;
  baseDir?: string;
  onFullscreenChange: (fullscreen: boolean) => void;
  onActiveTabChange: (tab: FilePreviewActiveTab) => void;
  onListModeChange: (mode: FilePreviewListMode) => void;
  onOpenOperation: (operation: FileOperation) => void;
  onOpenFilePath: (path: string) => void;
  onCloseTab: (id: string) => void;
  hasProject?: boolean;
  treeOpen?: boolean;
  onToggleTree?: () => void;
}

export function FilePreviewPanel(props: FilePreviewPanelProps) {
  const { t } = useTranslation();
  const activeOperation = props.tabs.find((tab) => tab.id === props.activeTab);
  const summaryOperations = props.listMode === "latest"
    ? props.latestOperations
    : props.listMode === "uncommitted"
      ? props.uncommittedOperations
      : props.allOperations;

  const openDefault = (operation: FileOperation) => {
    if (operation.source) return;
    openPreviewFile(operation.path, props.baseDir).catch(() => {});
  };

  const openWith = (operation: FileOperation, editorPath: string) => {
    if (operation.source) return;
    openPreviewWithEditor(operation.path, editorPath, props.baseDir).catch(() => {});
  };

  return (
    <div className="fp-preview-root">
          <div className="fp-head">
            <FilePreviewTabs
              tabs={props.tabs}
              activeTab={props.activeTab}
              listMode={props.listMode}
              baseDir={props.baseDir}
              onSelect={props.onActiveTabChange}
              onListModeChange={props.onListModeChange}
              onClose={props.onCloseTab}
              onOpenDefault={openDefault}
              onOpenWith={openWith}
            />
            {props.hasProject && (
              <Tooltip label={t("fileTree.toggleTree")}>
                <button
                  className={`icon-btn fp-icon-btn ${props.treeOpen ? "fp-icon-btn-active" : ""}`}
                  onClick={props.onToggleTree}
                >
                  <FolderTree size="var(--icon-md)" />
                </button>
              </Tooltip>
            )}
            <Tooltip label={props.fullscreen ? t("filePreview.reduce") : t("filePreview.fullscreen")} align="right">
              <button
                className="icon-btn fp-icon-btn"
                onClick={() => props.onFullscreenChange(!props.fullscreen)}
              >
                {props.fullscreen ? <Minimize2 size="var(--icon-md)" /> : <Maximize2 size="var(--icon-md)" />}
              </button>
            </Tooltip>
          </div>
          <div className="fp-body">
            {props.activeTab === "summary" || !activeOperation ? (
              <div className="fp-summary-scroll">
                <FilePreviewSummary
                  operations={summaryOperations}
                  baseDir={props.baseDir}
                  onOpen={props.onOpenOperation}
                  onOpenFile={(operation) => props.onOpenFilePath(operation.path)}
                />
              </div>
            ) : activeOperation.kind === "plan" ? (
              <FilePreviewPlan operation={activeOperation} baseDir={props.baseDir} />
            ) : (
              <>
                <FilePreviewBreadcrumb
                  operation={activeOperation}
                  baseDir={props.baseDir}
                />
                <div className={`fp-code-scroll ${shouldWrapFile(activeOperation.path) ? "" : "fp-nowrap"}`}>
                  <FilePreviewContent key={activeOperation.id} operation={activeOperation} baseDir={props.baseDir} />
                </div>
              </>
            )}
          </div>
    </div>
  );
}
