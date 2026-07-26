import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeSanitize from "rehype-sanitize";
import "./forecast-docs-markdown.css";

interface ForecastDocsMarkdownProps {
  content: string;
}

export function ForecastDocsMarkdown({ content }: ForecastDocsMarkdownProps) {
  return (
    <div className="fd-markdown">
      <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
        {content}
      </ReactMarkdown>
    </div>
  );
}
