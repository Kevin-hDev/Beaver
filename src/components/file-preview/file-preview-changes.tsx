import { useTranslation } from "react-i18next";
import type { FileOperation } from "@/types/file-preview";
import { FilePreviewContent } from "./file-preview-content";
import { FilePreviewStats } from "./file-preview-stats";
import { RecordedDiffPreview } from "./git-diff-preview";
import "./file-preview-changes.css";

export function FilePreviewChanges({ operation, baseDir }: {
  operation: FileOperation;
  baseDir?: string;
}) {
  const { t } = useTranslation();
  const changes = operation.changes ?? [operation];
  const older = operation.olderChanges;
  const count = changes.length + (older?.count ?? 0);

  return (
    <>
      {changes.map((change, index) => {
        const heading = (
          <span className="fpc-heading">
            <span>{t("filePreview.changeOf", { index: count - index, count })}</span>
            <FilePreviewStats operation={change} />
          </span>
        );
        return (
          <section key={change.id} className="fpc-change">
            {change.recordedStatus ? (
              <RecordedDiffPreview
                data={change.recordedDiff}
                path={change.path}
                status={change.recordedStatus}
                heading={heading}
              />
            ) : (
              <>
                <div className="gdp-status">{heading}</div>
                <FilePreviewContent operation={change} baseDir={baseDir} />
              </>
            )}
          </section>
        );
      })}
      {older && (
        <div className="gdp-status fpc-older">
          <span className="fpc-heading">
            <span>{t("filePreview.olderChanges", { count: older.count })}</span>
            <FilePreviewStats operation={older} />
          </span>
        </div>
      )}
    </>
  );
}
