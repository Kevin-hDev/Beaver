import { StandardActionNode } from "./action-node";
import { StandardTextNode, StandardSeparator } from "./content-nodes";
import { StandardFieldNode } from "./field-nodes";
import { StandardLayoutNode } from "./layout-nodes";
import { useContributionAction } from "./use-contribution-action";
import { useStandardViewState } from "./view-state";
import type { StandardCatalogEntry, StandardView } from "./types";

export function StandardViewRenderer({
  entry,
  view,
}: {
  entry: StandardCatalogEntry;
  view: StandardView;
}) {
  const state = useStandardViewState(view);
  const action = useContributionAction(entry, state.payload, state.replaceView);
  return (
    <div className="xui-view">
      <StandardNode
        node={state.view}
        values={state.values}
        onValue={state.setValue}
        busyAction={action.busyAction}
        onAction={(id) => void action.run(id)}
      />
    </div>
  );
}

function StandardNode({
  node,
  values,
  onValue,
  busyAction,
  onAction,
}: {
  node: StandardView;
  values: ReadonlyMap<string, null | boolean | number | string>;
  onValue: (id: string, value: null | boolean | number | string) => void;
  busyAction: string | null;
  onAction: (id: string) => void;
}) {
  if (node.type === "stack" || node.type === "row") {
    return (
      <StandardLayoutNode type={node.type}>
        {node.children.map((child, index) => (
          <StandardNode
            key={`${child.type}-${index}`}
            node={child}
            values={values}
            onValue={onValue}
            busyAction={busyAction}
            onAction={onAction}
          />
        ))}
      </StandardLayoutNode>
    );
  }
  if (node.type === "heading" || node.type === "text" || node.type === "badge") {
    return <StandardTextNode type={node.type} text={node.text} />;
  }
  if (node.type === "separator") return <StandardSeparator />;
  if (node.type === "button") {
    return <StandardActionNode node={node} busy={busyAction !== null} onRun={onAction} />;
  }
  if (!isFieldNode(node)) return null;
  return (
    <StandardFieldNode
      node={node}
      value={values.get(node.id) ?? node.value}
      onChange={(value) => onValue(node.id, value)}
    />
  );
}

function isFieldNode(
  node: StandardView,
): node is Extract<StandardView, { id: string; value: unknown }> {
  return node.type === "textField" || node.type === "numberField"
    || node.type === "select" || node.type === "toggle";
}
