import { Fragment, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { highlightLines } from "@/lib/highlight";
import { shouldWrapFile } from "@/lib/code-language";
import { readGitDiffPreview } from "@/services/file-preview";
import type { GitDiffHunk, GitDiffPreview as GitDiffData, GitDiffPreviewSource } from "@/types/file-preview";
import "./git-diff-preview.css";

interface GitDiffPreviewProps {
  source: GitDiffPreviewSource;
  path: string;
  baseDir?: string;
}

interface HighlightedHunk extends GitDiffHunk {
  highlighted: string[];
}

export function GitDiffPreview({ source, path, baseDir }: GitDiffPreviewProps) {
  const [state, setState] = useState<{
    loading: boolean;
    data?: GitDiffData;
    error: boolean;
  }>({ loading: true, error: false });

  useEffect(() => {
    let alive = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    setState({ loading: true, error: false });
    readGitDiffPreview(source, baseDir)
      .then((data) => {
        if (alive) setState({ loading: false, data, error: false });
      })
      .catch(() => {
        if (alive) setState({ loading: false, error: true });
      });
    return () => { alive = false; };
  }, [source, baseDir]);

  return (
    <DiffPreviewView
      data={state.data}
      error={state.error}
      loading={state.loading}
      path={path}
      previousPath={source.previousPath}
      status={source.status}
    />
  );
}

interface RecordedDiffPreviewProps {
  data?: GitDiffData;
  path: string;
  status: "added" | "modified" | "deleted";
}

export function RecordedDiffPreview({ data, path, status }: RecordedDiffPreviewProps) {
  return (
    <DiffPreviewView
      data={data}
      error={!data}
      loading={false}
      path={path}
      status={status}
    />
  );
}

function DiffPreviewView({ data, error, loading, path, previousPath, status }: {
  data?: GitDiffData;
  error: boolean;
  loading: boolean;
  path: string;
  previousPath?: string;
  status: GitDiffPreviewSource["status"];
}) {
  const { t } = useTranslation();

  const hunks = useMemo<HighlightedHunk[]>(() => (
    data?.hunks.map((hunk) => ({
      ...hunk,
      highlighted: highlightLines(hunk.lines.map((line) => line.content).join("\n"), path),
    })) ?? []
  ), [data, path]);

  if (loading) return <div className="fp-empty">{t("filePreview.loading")}</div>;
  const isRename = Boolean(previousPath);
  const wrap = shouldWrapFile(path);
  if (error || data?.binary || (hunks.length === 0 && !isRename)) {
    return <div className="fp-empty">{t("filePreview.diffUnavailable")}</div>;
  }

  const content = (
    <>
      <div className={`gdp-status gdp-status-${status}`}>
        <span className="gdp-status-label">{t(`filePreview.gitStatus.${status}`)}</span>
        {previousPath && (
          <span className="gdp-status-paths">
            <span>{previousPath}</span>
            <span className="gdp-status-arrow" aria-hidden="true">→</span>
            <span>{path}</span>
          </span>
        )}
      </div>
      {hunks.map((hunk, hunkIndex) => (
        <Fragment key={`${hunk.old_start}:${hunk.new_start}:${hunkIndex}`}>
          {hunkIndex > 0 && <div className="gdp-hunk-separator" aria-hidden="true">…</div>}
          <div className="gdp-hunk">
            {hunk.lines.map((line, lineIndex) => {
              const mode = line.kind === "added" ? "ok" : line.kind === "deleted" ? "error" : "context";
              const prefix = line.kind === "added" ? "+" : line.kind === "deleted" ? "-" : " ";
              return (
                <div className={`tp-line tp-line-${mode}`} key={lineIndex}>
                  <span className="gdp-line-number">{line.old_line ?? ""}</span>
                  <span className="gdp-line-number">{line.new_line ?? ""}</span>
                  <span className={`tp-prefix tp-prefix-${mode}`}>{prefix}</span>
                  <span
                    className={`tp-code tp-code-${mode}`}
                    dangerouslySetInnerHTML={{ __html: hunk.highlighted[lineIndex] || " " }}
                  />
                </div>
              );
            })}
          </div>
        </Fragment>
      ))}
      {data?.truncated && <div className="gdp-note">{t("filePreview.diffTruncated")}</div>}
    </>
  );

  return (
    <div className={`tp-wrapper gdp-wrapper ${wrap ? "" : "tp-nowrap"}`}>
      {wrap ? content : <div className="tp-inner">{content}</div>}
    </div>
  );
}
