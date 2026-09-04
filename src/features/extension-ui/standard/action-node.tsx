import { localizedText } from "./localized-text";
import type { StandardView } from "./types";

type ButtonNode = Extract<StandardView, { type: "button" }>;

export function StandardActionNode({
  node,
  busy,
  onRun,
}: {
  node: ButtonNode;
  busy: boolean;
  onRun: (actionId: string) => void;
}) {
  return (
    <button
      type="button"
      className="btn btn-sm btn-secondary xui-action"
      disabled={busy}
      aria-busy={busy || undefined}
      onClick={() => onRun(node.actionId)}
    >
      {localizedText(node.label)}
    </button>
  );
}
