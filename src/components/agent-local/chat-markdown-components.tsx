import { Children, type ReactElement, type ReactNode } from "react";
import type { Components } from "react-markdown";
import { open } from "@tauri-apps/plugin-shell";
import { highlightSkillNodes } from "@/lib/skill-text";
import { CodeBlock } from "./code-block";

const ALLOWED_LINK_PROTOCOLS = new Set(["http:", "https:", "mailto:"]);
const MAX_LINK_LENGTH = 2048;

function openMarkdownLink(href: string): void {
  if (href.length > MAX_LINK_LENGTH) return;
  try {
    const url = new URL(href);
    if (!ALLOWED_LINK_PROTOCOLS.has(url.protocol)) return;
    void open(url.toString()).catch(() => undefined);
  } catch {
    // Invalid and relative links stay inert in the desktop chat.
  }
}

function renderSkillText(
  children: ReactNode,
  skillNames: string[],
  builtInNames: string[],
): ReactNode[] {
  return highlightSkillNodes(Children.toArray(children), skillNames, { builtInNames });
}

function formatTableCell(children: ReactNode): ReactNode {
  if (typeof children !== "string") return children;
  const parts = children
    .split(/(\s+)(?=(?:[2-9]|[1-9]\d)[.)]\s|[2-9]️⃣\s)/g)
    .filter((part) => part.trim().length > 0);
  if (parts.length <= 1) return children;
  return parts.map((part, index) => (
    index === 0 ? part : <span key={index} className="chat-md-cell-list-break">{part}</span>
  ));
}

export function createChatMarkdownComponents(
  skillNames: string[] = [],
  builtInNames: string[] = [],
): Components {
  const renderText = (children: ReactNode) =>
    renderSkillText(children, skillNames, builtInNames);

  return {
    p({ children, node: _node, ...props }) {
      return <p {...props}>{renderText(children)}</p>;
    },
    h1({ children, node: _node, ...props }) {
      return <h1 {...props}>{renderText(children)}</h1>;
    },
    h2({ children, node: _node, ...props }) {
      return <h2 {...props}>{renderText(children)}</h2>;
    },
    h3({ children, node: _node, ...props }) {
      return <h3 {...props}>{renderText(children)}</h3>;
    },
    h4({ children, node: _node, ...props }) {
      return <h4 {...props}>{renderText(children)}</h4>;
    },
    h5({ children, node: _node, ...props }) {
      return <h5 {...props}>{renderText(children)}</h5>;
    },
    h6({ children, node: _node, ...props }) {
      return <h6 {...props}>{renderText(children)}</h6>;
    },
    li({ children, node: _node, ...props }) {
      return <li {...props}>{renderText(children)}</li>;
    },
    strong({ children, node: _node, ...props }) {
      return <strong {...props}>{renderText(children)}</strong>;
    },
    em({ children, node: _node, ...props }) {
      return <em {...props}>{renderText(children)}</em>;
    },
    del({ children, node: _node, ...props }) {
      return <del {...props}>{renderText(children)}</del>;
    },
    table({ children, node: _node, ...props }) {
      return (
        <div className="chat-md-table-scroll">
          <table {...props}>{children}</table>
        </div>
      );
    },
    th({ children, node: _node, ...props }) {
      return <th {...props}>{renderText(formatTableCell(children))}</th>;
    },
    td({ children, node: _node, ...props }) {
      return <td {...props}>{renderText(formatTableCell(children))}</td>;
    },
    pre({ children }) {
      const child = children as ReactElement<{ className?: string; children?: ReactNode }>;
      const className = child?.props?.className || "";
      const language = /language-(\w+)/.exec(className)?.[1] || "";
      const raw = child?.props?.children;
      const code = (typeof raw === "string" ? raw : "").replace(/\n$/, "");
      return <CodeBlock language={language} code={code} />;
    },
    a({ href, children, node: _node, ...props }) {
      return (
        <a
          {...props}
          className="chat-link"
          href={href ?? "#"}
          title={href ?? ""}
          onClick={(event) => {
            event.preventDefault();
            if (href) openMarkdownLink(href);
          }}
        >
          {children}
        </a>
      );
    },
  };
}
