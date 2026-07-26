import { useMemo, type ReactNode } from "react";
import { highlightCodeNodes, type HighlightNode } from "@/lib/highlight";
import "@/components/file-preview/file-preview-highlight.css";
import "./code-theme-preview.css";

// Exemple de code couvrant tous les types de tokens (keyword, string,
// number, comment, type, variable, function) pour illustrer chaque thème.
const SAMPLE_CODE = `// User service example
interface User {
  id: number;
  name: string;
}

function fetchUser(id: number): User | null {
  const users: User[] = loadFromCache();
  return users.find((u) => u.id === id) ?? null;
}

const active = fetchUser(42);
console.log("Active:", active?.name);`;

function renderHighlightNode(node: HighlightNode, index: number): ReactNode {
  if (typeof node === "string") return node;
  return (
    <span key={index} className={node.className}>
      {node.children.map(renderHighlightNode)}
    </span>
  );
}

interface PreviewBubbleProps {
  themeId: string;
  variant: "dark" | "light";
  label: string;
}

function PreviewBubble({ themeId, variant, label }: PreviewBubbleProps) {
  const highlighted = useMemo(
    () => highlightCodeNodes(SAMPLE_CODE, "typescript"),
    [],
  );

  return (
    <div className="ctp-bubble" data-code-theme={themeId} data-theme={variant}>
      <span className="ctp-bubble-label">{label}</span>
      <pre className="ctp-bubble-pre">
        <code>{highlighted.map(renderHighlightNode)}</code>
      </pre>
    </div>
  );
}

interface CodeThemePreviewProps {
  themeId: string;
}

export function CodeThemePreview({ themeId }: CodeThemePreviewProps) {
  return (
    <div className="ctp-root">
      <PreviewBubble themeId={themeId} variant="dark" label="Dark" />
      <PreviewBubble themeId={themeId} variant="light" label="Light" />
    </div>
  );
}
