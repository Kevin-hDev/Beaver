import { useMemo } from "react";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import type { ToolActivityRecord } from "@/types/agent";
import { groupToolActivities } from "@/lib/tool-activity-summary";
import { isHiddenAgentTool } from "@/lib/hidden-agent-tools";
import { ToolActivityGroupList } from "./tool-activity-group";
import {
  type RenderableTool,
  savedToolToRenderable,
  streamToolToRenderable,
} from "./tool-detail-row";
import "./tool-bubble.css";
import "./tool-bubble-arrows.css";
import "./tool-bubble-detail.css";
import "./tool-bubble-status.css";
import "./stream-active.css";

export function ToolBubble({
  tools,
  activeTools = [],
  onFilePreview,
  projectPath,
}: {
  tools: ToolActivity[];
  activeTools?: ToolActivity[];
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const renderableTools = useMemo(
    () => tools
      .map((tool) => ({ tool, isActive: activeTools.includes(tool) }))
      .filter(({ tool }) => isVisibleTool(tool))
      .map(({ tool, isActive }) => streamToolToRenderable(tool, isActive)),
    [tools, activeTools],
  );
  return (
    <ToolActivityList
      tools={renderableTools}
      onFilePreview={onFilePreview}
      projectPath={projectPath}
    />
  );
}

export function SavedToolBubble({
  tools,
  onFilePreview,
  projectPath,
}: {
  tools: ToolActivityRecord[];
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const renderableTools = useMemo(
    () => tools.filter(isVisibleTool).map(savedToolToRenderable),
    [tools],
  );
  return <ToolActivityList tools={renderableTools} onFilePreview={onFilePreview} projectPath={projectPath} />;
}

function ToolActivityList({
  tools,
  onFilePreview,
  projectPath,
}: {
  tools: RenderableTool[];
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const groups = useMemo(() => groupToolActivities(tools), [tools]);
  if (tools.length === 0) return null;
  return (
    <div className="tb-stream">
      <ToolActivityGroupList
        groups={groups}
        onFilePreview={onFilePreview}
        projectPath={projectPath}
      />
    </div>
  );
}

function isVisibleTool(tool: { name: string }) {
  return !isHiddenAgentTool(tool.name);
}
