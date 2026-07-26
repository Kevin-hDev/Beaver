import { useMemo, type ReactNode } from "react";
import ReactMarkdown from "react-markdown";
import type { Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkBreaks from "remark-breaks";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import { open } from "@tauri-apps/plugin-shell";
import { languageFromPath } from "@/lib/code-language";
import { highlightCodeNodes, type HighlightNode } from "@/lib/highlight";
import "./tool-result-markdown.css";
import "@/components/file-preview/file-preview-highlight.css";

const remarkPlugins = [remarkGfm, remarkBreaks];
const rehypePlugins = [rehypeRaw, rehypeSanitize];

function renderHighlightNode(node: HighlightNode, index: number): ReactNode {
  if (typeof node === "string") return node;
  return (
    <span key={index} className={node.className}>
      {node.children.map(renderHighlightNode)}
    </span>
  );
}

/**
 * Rendu Markdown des résultats de commandes shell (bash, grep, glob, list_dir,
 * web_search, web_fetch). Réutilise la même pipeline que les bulles assistant
 * (même fond, même typo, listes/tableaux), mais les blocs de code sont rendus
 * SANS coloration syntaxique — la sortie d'une commande n'est pas du code.
 */
export function ToolResultMarkdown({ content }: { content: string }) {
  const components = useMemo<Components>(() => ({
    table({ children }) {
      return (
        <div className="chat-md-table-scroll">
          <table>{children}</table>
        </div>
      );
    },
    pre({ children }) {
      // Bloc de code sans coloration : juste police mono dans un cadre sobre.
      return <pre className="tb-md-codeblock">{children}</pre>;
    },
    a({ href, children }) {
      return (
        <a
          className="chat-link"
          href={href ?? "#"}
          title={href ?? ""}
          onClick={(e) => { e.preventDefault(); if (href) void open(href); }}
        >
          {children}
        </a>
      );
    },
  }), []);

  const prepared = useMemo(() => content, [content]);

  return (
    <div className="chat-column-surface tb-result-md chat-md">
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        components={components}
      >
        {prepared}
      </ReactMarkdown>
    </div>
  );
}

export function ToolResultCode({ content, path }: { content: string; path: string }) {
  const language = useMemo(() => languageFromPath(path), [path]);
  const highlighted = useMemo(() => highlightCodeNodes(content, language), [content, language]);

  return (
    <div className="chat-column-surface tb-result-md">
      <pre className="tb-result-code">
        <code>{highlighted.map(renderHighlightNode)}</code>
      </pre>
    </div>
  );
}
