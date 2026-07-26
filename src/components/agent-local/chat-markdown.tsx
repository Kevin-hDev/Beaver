import { useMemo } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkBreaks from "remark-breaks";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { LinkPreviewCard } from "./link-preview-card";
import { createChatMarkdownComponents } from "./chat-markdown-components";
import "./chat-markdown.css";

const MAX_PREVIEWS = 5;
const MAX_URL_LENGTH = 2048;
const URL_PATTERN = /https?:\/\/[^\s<>"')\]]+/g;
const remarkPlugins = [remarkGfm, remarkBreaks];
const rehypePlugins = [rehypeRaw, rehypeSanitize];
const EMPTY_NAMES: string[] = [];

interface ChatMarkdownProps {
  content: string;
  skillNames?: string[];
  builtInNames?: string[];
}

function extractUrls(text: string): string[] {
  const urls: string[] = [];
  const seen = new Set<string>();
  for (const match of text.matchAll(URL_PATTERN)) {
    const url = match[0];
    if (url.length > MAX_URL_LENGTH) continue;
    if (seen.has(url)) continue;
    seen.add(url);
    urls.push(url);
    if (urls.length >= MAX_PREVIEWS) break;
  }
  return urls;
}

function isPreviewEnabled(): boolean {
  if (typeof localStorage === "undefined") return true;
  try {
    return localStorage.getItem("clgo-link-preview") !== "false";
  } catch {
    return true;
  }
}

function closeUnclosedCodeBlocks(text: string): string {
  const count = (text.match(/```/g) || []).length;
  return count % 2 === 0 ? text : `${text}\n\`\`\``;
}

export function ChatMarkdown({
  content,
  skillNames = EMPTY_NAMES,
  builtInNames = EMPTY_NAMES,
}: ChatMarkdownProps) {
  const prepared = useMemo(() => closeUnclosedCodeBlocks(content), [content]);
  const urls = useMemo(
    () => isPreviewEnabled() ? extractUrls(content) : [],
    [content],
  );
  const components = useMemo(
    () => createChatMarkdownComponents(skillNames, builtInNames),
    [builtInNames, skillNames],
  );

  return (
    <>
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        components={components}
      >
        {prepared}
      </ReactMarkdown>
      {urls.length > 0 && (
        <div className="chat-previews-block">
          {urls.map((url) => <LinkPreviewCard key={url} url={url} />)}
        </div>
      )}
    </>
  );
}
