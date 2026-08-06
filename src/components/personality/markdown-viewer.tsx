import { useTranslation } from "react-i18next";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { OpenExternalIcon } from "@/components/ui/panel-action-icons";
import { Tooltip } from "@/components/ui/tooltip";
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
        {/* Le libellé est parti : sans lui, l'infobulle et le nom accessible
            sont les deux seuls endroits qui disent encore ce que fait ce
            bouton. Le contour reste — posé sur un en-tête et non dans une
            carte, un bouton nu ne se distinguerait pas du fond. */}
        <Tooltip label={t("personality.open")} align="right">
          <button
            className="icon-btn icon-btn-secondary"
            type="button"
            aria-label={t("personality.open")}
            onClick={onOpenEditor}
          >
            <OpenExternalIcon size="var(--personality-open-icon-size)" />
          </button>
        </Tooltip>
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
