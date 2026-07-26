import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeSanitize from "rehype-sanitize";

interface ForecastNotesMarkdownProps {
  content: string;
}

export function ForecastNotesMarkdown({ content }: ForecastNotesMarkdownProps) {
  return (
    <div className="fcn-markdown">
      <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
        {content}
      </ReactMarkdown>
    </div>
  );
}
