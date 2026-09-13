import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretUp } from "@/components/ui/icons";
import { Collapsible } from "@/components/ui/collapsible";
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
        return (
          <FilePreviewChange
            key={change.id}
            change={change}
            index={count - index}
            count={count}
            baseDir={baseDir}
          />
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

function FilePreviewChange({ change, index, count, baseDir }: {
  change: FileOperation;
  index: number;
  count: number;
  baseDir?: string;
}) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(true);
  const heading = (
    <button
      type="button"
      className="fpc-heading"
      aria-expanded={open}
      onClick={() => setOpen((value) => !value)}
    >
      <span>{t("filePreview.changeOf", { index, count })}</span>
      <FilePreviewStats operation={change} />
      <span className="fpc-heading-caret" aria-hidden="true">
        {open ? <CaretUp size="var(--icon-sm)" /> : <CaretDown size="var(--icon-sm)" />}
      </span>
    </button>
  );

  return (
    <section className="fpc-change">
      {change.recordedStatus ? (
        <RecordedDiffPreview
          data={change.recordedDiff}
          path={change.path}
          status={change.recordedStatus}
          heading={heading}
          open={open}
        />
      ) : (
        <>
          <div className="gdp-status">{heading}</div>
          <Collapsible open={open}>
            <FilePreviewContent operation={change} baseDir={baseDir} />
          </Collapsible>
        </>
      )}
    </section>
  );
}
