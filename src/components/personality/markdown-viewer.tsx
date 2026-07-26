import { useTranslation } from "react-i18next";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { ArrowSquareOut } from "@/components/ui/icons";
import "github-markdown-css/github-markdown.css";
import "./markdown-viewer.css";

interface MarkdownViewerProps {
  content: string;
  fileName: string;
  onOpenEditor: () => void;
}

function stripFrontmatter(md: string): string {
  if (md.startsWith("---")) {
    const end = md.indexOf("---", 3);
    if (end > 0) return md.slice(end + 3).trim();
  }
  return md;
}

export function MarkdownViewer({
  content,
  fileName,
  onOpenEditor,
}: MarkdownViewerProps) {
  const { t } = useTranslation();

  return (
    <>
      <div className="md-header">
        <div className="md-title">{fileName}</div>
        <button className="btn" onClick={onOpenEditor}>
          <ArrowSquareOut size="var(--icon-sm)" /> {t("personality.open")}
        </button>
      </div>
      <div className="md-scroll">
        <div className="markdown-body">
          <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeRaw, rehypeSanitize]}>
            {stripFrontmatter(content)}
          </ReactMarkdown>
        </div>
      </div>
    </>
  );
}
