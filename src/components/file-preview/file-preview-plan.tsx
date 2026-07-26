import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ChatMarkdown } from "@/components/agent-local/chat-markdown";
import { readFilePreview } from "@/services/file-preview";
import type { FileOperation } from "@/types/file-preview";
import "./file-preview-plan.css";

interface FilePreviewPlanProps {
  operation: FileOperation;
  baseDir?: string;
}

export function FilePreviewPlan({ operation, baseDir }: FilePreviewPlanProps) {
  const { t } = useTranslation();
  const requestKey = `${baseDir ?? ""}\0${operation.path}`;
  const [state, setState] = useState({ key: requestKey, loading: true, content: "", error: false });

  useEffect(() => {
    let alive = true;
    readFilePreview(operation.path, baseDir)
      .then((content) => {
        if (alive) setState({ key: requestKey, loading: false, content, error: false });
      })
      .catch(() => {
        if (alive) setState({ key: requestKey, loading: false, content: "", error: true });
      });
    return () => { alive = false; };
  }, [operation.path, baseDir, requestKey]);

  if (state.key !== requestKey || state.loading) {
    return <div className="fp-empty">{t("filePreview.loading")}</div>;
  }
  if (state.error) {
    return <div className="fp-empty">{t("filePreview.fileNotFound")}</div>;
  }

  return (
    <div className="fp-plan-scroll">
      <article className="fp-plan-doc chat-md">
        <ChatMarkdown content={state.content} />
      </article>
    </div>
  );
}
