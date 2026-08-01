import { useMemo } from "react";
import { Spinner } from "@/components/ui/icons";
import { CaretDown, CaretUp } from "@/components/ui/icons";
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import type { ToolActivityCounts, ToolActivityGroup } from "@/lib/tool-activity-summary";
import { groupIcon } from "@/lib/tool-activity-summary";
import type { RenderableTool } from "./tool-detail-row";
import { ToolDetailRow } from "./tool-detail-row";
import { ToolIcon } from "./tool-icons";
import { ToolStatusIcon } from "./tool-status-icon";
import { useCollapsiblePresence } from "./use-collapsible-presence";

const COUNT_ORDER: Array<keyof ToolActivityCounts> = [
  "files",
  "searches",
  "lists",
  "writes",
  "edits",
  "commands",
  "webSearches",
  "webFetches",
  "gitActions",
  "otherActions",
];

function joinParts(parts: string[]): string {
  if (parts.length <= 1) return parts[0] ?? "";
  return parts.join(", ");
}

function summaryDetails(
  group: ToolActivityGroup<RenderableTool>,
  t: TFunction,
): string {
  const parts = COUNT_ORDER
    .filter((key) => group.counts[key] > 0)
    .map((key) => t(`agentLocal.toolActivity.counts.${key}`, { count: group.counts[key] }));
  return joinParts(parts);
}

function ToolActivityGroupRow({
  group,
  onFilePreview,
  projectPath,
}: {
  group: ToolActivityGroup<RenderableTool>;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const { t } = useTranslation();
  const { open: isOpen, mounted, toggle, onTransitionEnd } = useCollapsiblePresence();
  const label = t(`agentLocal.toolActivity.groups.${group.kind}`);
  const details = useMemo(
    () => summaryDetails(group, t),
    [group, t],
  );
  const groupActive = group.tools.some((tool) => tool.isActive);

  return (
    <div className={`tb-group tb-group-${group.kind}`}>
      <button
        type="button"
        className={`tb-group-toggle${groupActive && !isOpen ? " stream-active" : ""}`}
        aria-expanded={isOpen}
        aria-label={t("agentLocal.toolActivity.toggleDetails")}
        onClick={toggle}
      >
        <ToolIcon name={groupIcon(group.kind)} size="var(--icon-sm)" className="tb-group-icon" aria-hidden="true" />
        <span className={`tb-group-title${groupActive && !isOpen ? " stream-active-label" : ""}`}>
          {details ? (
            <>
              {label}
              <span className="tb-group-sep" aria-hidden="true"> · </span>
              <span className="tb-group-details-text">{details}</span>
            </>
          ) : (
            label
          )}
          {group.isPending && (
            <span className="tb-group-progress"> {t("agentLocal.toolActivity.inProgress")}</span>
          )}
          {!group.isPending && group.kind === "memory" && (
            <span className="tb-group-progress"> {t("agentLocal.toolActivity.completed")}</span>
          )}
        </span>
        <span className="tb-arrow tb-group-arrow" aria-hidden="true">
          {isOpen ? <CaretUp size="var(--icon-sm)" weight="bold" /> : <CaretDown size="var(--icon-sm)" weight="bold" />}
        </span>
        <span className="tb-group-state" aria-hidden="true">
          {group.isPending && <Spinner size="var(--icon-sm)" className="tb-spinner" />}
          {!group.isPending && group.hasError && (
            <ToolStatusIcon message={t("agentLocal.toolActivity.groupError")} />
          )}
        </span>
      </button>
      <div className={`tb-group-accordion${isOpen ? " tb-open" : ""}`} onTransitionEnd={onTransitionEnd}>
        {mounted && (
          <div className="tb-group-details">
            {group.tools.map((tool, index) => (
              <ToolDetailRow
                key={`${tool.name}-${index}-${tool.summary}`}
                tool={tool}
                previousTools={group.tools.slice(0, index)}
                isActive={isOpen && tool.isActive}
                onFilePreview={onFilePreview}
                projectPath={projectPath}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function ToolActivityGroupList({
  groups,
  onFilePreview,
  projectPath,
}: {
  groups: Array<ToolActivityGroup<RenderableTool>>;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  return (
    <div className="tb-group-list">
      {groups.map((group) => (
        group.tools.length === 1 && group.kind !== "memory" ? (
          <ToolDetailRow
            key={group.kind}
            tool={group.tools[0]}
            previousTools={[]}
            isActive={group.tools[0].isActive}
            onFilePreview={onFilePreview}
            projectPath={projectPath}
          />
        ) : (
          <ToolActivityGroupRow
            key={group.kind}
            group={group}
            onFilePreview={onFilePreview}
            projectPath={projectPath}
          />
        )
      ))}
    </div>
  );
}
