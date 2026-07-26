import { useCallback, useRef, useState, type UIEvent } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, CaretLeft } from "@/components/ui/icons";
import { CommitIcon } from "@/components/ui/session-summary-icons";
import { FileIcon } from "@/components/file-preview/file-icon";
import type { GitCommitFile, GitCommitSummary } from "@/hooks/git-types";
import type { SessionSummaryGitState } from "./session-summary-git-types";
import "./session-summary-commits.css";

const MAX_LOADED_COMMITS = 1_000;
const LOAD_THRESHOLD = 40;

interface SessionSummaryCommitsProps {
  git?: SessionSummaryGitState;
  onOpenFile?: (commit: GitCommitSummary, file: GitCommitFile) => void;
}

export function SessionSummaryCommits({ git, onOpenFile }: SessionSummaryCommitsProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [commits, setCommits] = useState<GitCommitSummary[]>([]);
  const [cursor, setCursor] = useState<string | undefined>();
  const [selected, setSelected] = useState<GitCommitSummary | null>(null);
  const [files, setFiles] = useState<GitCommitFile[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(false);
  const loadingRef = useRef(false);
  const requestRef = useRef(0);
  const selectionRequestRef = useRef(0);

  const loadPage = useCallback(async () => {
    if (!git || loadingRef.current || commits.length >= MAX_LOADED_COMMITS) return;
    loadingRef.current = true;
    const request = ++requestRef.current;
    setLoading(true);
    setError(false);
    try {
      const page = await git.listCommits(cursor);
      if (request !== requestRef.current) return;
      setCommits((current) => [...current, ...page.commits].slice(0, MAX_LOADED_COMMITS));
      setCursor(page.next_cursor);
    } catch {
      if (request !== requestRef.current) return;
      setError(true);
    } finally {
      if (request === requestRef.current) {
        loadingRef.current = false;
        setLoading(false);
      }
    }
  }, [commits.length, cursor, git]);

  if (!git?.isGitRepo || !git.currentBranch || git.currentBranch === "HEAD") return null;

  const toggle = () => {
    const next = !open;
    setOpen(next);
    if (next && commits.length === 0) void loadPage();
    if (!next) {
      requestRef.current += 1;
      loadingRef.current = false;
      setLoading(false);
      setCommits([]);
      setCursor(undefined);
      setSelected(null);
      setFiles(null);
      setError(false);
      selectionRequestRef.current += 1;
    }
  };

  const selectCommit = async (commit: GitCommitSummary) => {
    const request = ++selectionRequestRef.current;
    setSelected(commit);
    setFiles(null);
    setError(false);
    try {
      const nextFiles = (await git.listCommitFiles(commit.id)).slice(0, 200);
      if (request !== selectionRequestRef.current) return;
      setFiles(nextFiles);
    } catch {
      if (request !== selectionRequestRef.current) return;
      setError(true);
      setFiles([]);
    }
  };

  const handleScroll = (event: UIEvent<HTMLDivElement>) => {
    const element = event.currentTarget;
    const nearEnd = element.scrollHeight - element.scrollTop - element.clientHeight < LOAD_THRESHOLD;
    if (nearEnd && cursor && !loading) void loadPage();
  };

  return (
    <>
      <button
        className="ssb-row ssb-commit-toggle"
        type="button"
        aria-expanded={open}
        aria-label={t("agentLocal.sessionSummary.commits.toggle")}
        onClick={toggle}
      >
        <CommitIcon size="var(--icon-md)" className="ssb-row-icon" />
        <span className="ssb-row-label">{t("agentLocal.sessionSummary.commits.title")}</span>
        <CaretDown className={`ssb-section-caret ${open ? "ssb-section-caret-open" : ""}`} size="var(--icon-sm)" />
      </button>
      <div className={`ssb-accordion ${open ? "ssb-accordion-open" : ""}`}>
        <div className="ssb-accordion-inner">
          <div className="ssbc-panel">
            {selected ? (
              <CommitFiles
                commit={selected}
                files={files}
                error={error}
                onBack={() => {
                  selectionRequestRef.current += 1;
                  setSelected(null);
                  setFiles(null);
                  setError(false);
                }}
                onOpen={(file) => onOpenFile?.(selected, file)}
              />
            ) : (
              <CommitList
                commits={commits}
                loading={loading}
                error={error}
                onScroll={handleScroll}
                onSelect={(commit) => void selectCommit(commit)}
              />
            )}
          </div>
        </div>
      </div>
    </>
  );
}

function CommitList({ commits, loading, error, onScroll, onSelect }: {
  commits: GitCommitSummary[];
  loading: boolean;
  error: boolean;
  onScroll: (event: UIEvent<HTMLDivElement>) => void;
  onSelect: (commit: GitCommitSummary) => void;
}) {
  const { t, i18n } = useTranslation();
  if (loading && commits.length === 0) return <div className="ssb-empty">{t("common.loading")}</div>;
  if (error && commits.length === 0) return <div className="ssb-empty">{t("agentLocal.sessionSummary.commits.error")}</div>;
  if (commits.length === 0) return <div className="ssb-empty">{t("agentLocal.sessionSummary.commits.empty")}</div>;
  return (
    <div className="ssbc-scroll" onScroll={onScroll}>
      {commits.map((commit) => (
        <button key={commit.id} className="ssbc-commit" type="button" onClick={() => onSelect(commit)}>
          <span className="ssbc-message">{commit.message || t("agentLocal.sessionSummary.commits.noMessage")}</span>
          <span className="ssbc-meta">{formatDate(commit.timestamp, i18n.language)} · {commit.short_id}</span>
        </button>
      ))}
      {loading && <div className="ssbc-loading">{t("common.loading")}</div>}
    </div>
  );
}

function CommitFiles({ commit, files, error, onBack, onOpen }: {
  commit: GitCommitSummary;
  files: GitCommitFile[] | null;
  error: boolean;
  onBack: () => void;
  onOpen: (file: GitCommitFile) => void;
}) {
  const { t } = useTranslation();
  return (
    <div>
      <button className="ssbc-back" type="button" onClick={onBack}>
        <CaretLeft size="var(--icon-sm)" />
        <span>{commit.message || commit.short_id}</span>
      </button>
      {files === null ? <div className="ssb-empty">{t("common.loading")}</div> : null}
      {error ? <div className="ssb-empty">{t("agentLocal.sessionSummary.commits.error")}</div> : null}
      {files?.length === 0 && !error ? <div className="ssb-empty">{t("agentLocal.sessionSummary.commits.noFiles")}</div> : null}
      <div className="ssbc-files">
        {files?.map((file) => (
          <button key={`${file.status}:${file.path}`} className="ssbc-file" type="button" onClick={() => onOpen(file)}>
            <FileIcon name={file.path} size="var(--icon-md)" />
            <span>{file.path}</span>
            <span className="ssbc-stats"><b>+{file.additions}</b><i>-{file.deletions}</i></span>
          </button>
        ))}
      </div>
    </div>
  );
}

function formatDate(timestamp: number, locale: string) {
  const date = new Date(timestamp * 1000);
  if (!Number.isFinite(date.getTime())) return "";
  return new Intl.DateTimeFormat(locale, { dateStyle: "short" }).format(date);
}
