import { Fragment, useEffect, useMemo, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Collapsible } from "@/components/ui/collapsible";
import { highlightLines } from "@/lib/highlight";
import { shouldWrapFile } from "@/lib/code-language";
import { readGitDiffPreview } from "@/services/file-preview";
import type { GitDiffHunk, GitDiffLine, GitDiffPreview as GitDiffData, GitDiffPreviewSource } from "@/types/file-preview";
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
  heading?: ReactNode;
  open?: boolean;
}

export function RecordedDiffPreview({ data, path, status, heading, open }: RecordedDiffPreviewProps) {
  return (
    <DiffPreviewView
      data={data}
      error={!data}
      loading={false}
      path={path}
      status={status}
      heading={heading}
      open={open}
    />
  );
}

function DiffPreviewView({ data, error, loading, path, previousPath, status, heading, open }: {
  data?: GitDiffData;
  error: boolean;
  loading: boolean;
  path: string;
  previousPath?: string;
  status: GitDiffPreviewSource["status"];
  heading?: ReactNode;
  open?: boolean;
}) {
  const { t } = useTranslation();

  const hunks = useMemo<HighlightedHunk[]>(() => (
    data?.hunks.map((hunk) => ({
      ...hunk,
      highlighted: highlightHunk(hunk.lines, path),
    })) ?? []
  ), [data, path]);

  if (loading) return <div className="fp-empty">{t("filePreview.loading")}</div>;
  const isRename = Boolean(previousPath);
  const wrap = shouldWrapFile(path);
  const statusBar = (
    <div className={`gdp-status gdp-status-${status}`}>
      {!heading && <span className="gdp-status-label">{t(`filePreview.gitStatus.${status}`)}</span>}
      {previousPath && (
        <span className="gdp-status-paths">
          <span>{previousPath}</span>
          <span className="gdp-status-arrow" aria-hidden="true">→</span>
          <span>{path}</span>
        </span>
      )}
      {heading}
    </div>
  );
  const collapsible = (body: ReactNode) => open === undefined
    ? body
    : <Collapsible open={open}>{body}</Collapsible>;
  if (error || data?.binary || (hunks.length === 0 && !isRename)) {
    return heading ? (
      <>
        {statusBar}
        {collapsible(<div className="fp-empty">{t("filePreview.diffUnavailable")}</div>)}
      </>
    ) : <div className="fp-empty">{t("filePreview.diffUnavailable")}</div>;
  }

  const content = (
    <>
      {statusBar}
      {collapsible(<>
        {hunks.map((hunk, hunkIndex) => (
          <Fragment key={`${hunk.old_start}:${hunk.new_start}:${hunkIndex}`}>
            {hunkIndex > 0 && <div className="gdp-hunk-separator" aria-hidden="true">…</div>}
            <div className="gdp-hunk">
              {hunk.lines.map((line, lineIndex) => {
                const mode = lineMode(line.kind, status);
                const prefix = mode === "ok" ? "+" : mode === "error" ? "-" : " ";
                const lineNumber = line.kind === "deleted" ? line.old_line : line.new_line;
                return (
                  <div className={`tp-line tp-line-${mode}`} key={lineIndex}>
                    <span className="gdp-line-number">{lineNumber ?? ""}</span>
                    <span className={`tp-prefix tp-prefix-${mode}`}>{prefix}</span>
                    <span
                      className="tp-code"
                      dangerouslySetInnerHTML={{ __html: hunk.highlighted[lineIndex] || " " }}
                    />
                  </div>
                );
              })}
            </div>
          </Fragment>
        ))}
        {data?.truncated && <div className="gdp-note">{t("filePreview.diffTruncated")}</div>}
      </>)}
    </>
  );

  return (
    <div className={`tp-wrapper gdp-wrapper ${wrap ? "" : "tp-nowrap"}`}>
      {wrap ? content : <div className="tp-inner">{content}</div>}
    </div>
  );
}

/* Un fichier créé n'a rien à comparer : son libellé suffit. */
function lineMode(
  kind: GitDiffLine["kind"],
  status: GitDiffPreviewSource["status"],
): "ok" | "error" | "context" {
  if (status === "added") return "context";
  return kind === "added" ? "ok" : kind === "deleted" ? "error" : "context";
}

/* Les deux côtés sont coloriés séparément pour qu'un token retiré ne colore
   jamais la ligne qui le remplace. */
function highlightHunk(lines: GitDiffLine[], path: string): string[] {
  const side = (skipped: GitDiffLine["kind"]) => highlightLines(
    lines.filter((line) => line.kind !== skipped).map((line) => line.content).join("\n"),
    path,
  );
  const before = side("added");
  const after = side("deleted");
  let beforeIndex = 0;
  let afterIndex = 0;
  return lines.map((line) => {
    if (line.kind === "deleted") return before[beforeIndex++];
    if (line.kind === "context") beforeIndex++;
    return after[afterIndex++];
  });
}
