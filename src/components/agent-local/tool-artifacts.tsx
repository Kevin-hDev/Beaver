import { useTranslation } from "react-i18next";
import type { ToolArtifact } from "@/types/agent";
import { formatByteSize } from "@/lib/format-byte-size";
import "./tool-artifacts.css";

export function ToolArtifacts({
  artifacts,
  onFilePreview,
}: {
  artifacts?: ToolArtifact[];
  onFilePreview?: (path: string) => void;
}) {
  const { t, i18n } = useTranslation();
  if (!artifacts?.length) return null;

  return (
    <ul className="ta-list" aria-label={t("agentLocal.toolActivity.artifacts") }>
      {artifacts.map((artifact, index) => {
        const workspacePath = artifact.source.kind === "workspace_file"
          ? artifact.source.path : undefined;
        const sourceId = artifact.source.kind === "workspace_file"
          ? artifact.source.path : artifact.source.resource_id;
        const canPreview = workspacePath !== undefined && onFilePreview !== undefined;
        const content = (
          <>
            <span className="ta-name">{artifact.name}</span>
            <span className="ta-meta">{artifact.mime_type} · {formatByteSize(artifact.bytes, i18n.language)}</span>
            <span className="ta-status">
              {t(`agentLocal.toolActivity.artifactStatus.${artifact.verification ?? "unverified"}`)}
            </span>
          </>
        );
        return (
          <li className="ta-item" key={`${sourceId}:${artifact.sha256}:${index}`}>
            {canPreview ? (
              <button
                className="ta-row ta-open"
                type="button"
                onClick={() => {
                  if (onFilePreview && workspacePath) onFilePreview(workspacePath);
                }}
                aria-label={t("agentLocal.toolActivity.openArtifact", { name: artifact.name })}
              >
                {content}
              </button>
            ) : <div className="ta-row">{content}</div>}
          </li>
        );
      })}
    </ul>
  );
}
