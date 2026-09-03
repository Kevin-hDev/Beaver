import type { StandardLocalizedText } from "./types";
import { localizedText } from "./localized-text";

export function StandardTextNode({
  type,
  text,
}: {
  type: "heading" | "text" | "badge";
  text: StandardLocalizedText;
}) {
  const value = localizedText(text);
  if (type === "heading") return <h3 className="xui-heading">{value}</h3>;
  if (type === "badge") return <span className="xui-badge">{value}</span>;
  return <p className="xui-text">{value}</p>;
}

export function StandardSeparator() {
  return <hr className="xui-separator" />;
}
