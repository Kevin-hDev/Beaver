import { useCallback, useState } from "react";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import type { ExtensionUiActionPayload } from "@/types/extensions";
import type { StandardFieldValue, StandardView } from "./types";

interface ViewState {
  view: StandardView;
  declared: Array<[string, StandardFieldValue]>;
  values: Map<string, StandardFieldValue>;
}

export function useStandardViewState(initialView: StandardView) {
  const [state, setState] = useState(() => stateFor(initialView));
  const setValue = useCallback((id: string, value: StandardFieldValue) => {
    setState((current) => {
      if (!current.declared.some(([candidate]) => candidate === id)) return current;
      const next = new Map(current.values);
      next.set(id, value);
      return { ...current, values: next };
    });
  }, []);
  const replaceView = useCallback((view: StandardView) => setState(stateFor(view)), []);
  const payload = useCallback((): ExtensionUiActionPayload => ({
    fields: Object.fromEntries(state.declared.map(([id, fallback]) => [
      id,
      state.values.has(id) ? state.values.get(id)! : fallback,
    ])),
  }), [state.declared, state.values]);
  return { ...state, setValue, replaceView, payload };
}

function stateFor(view: StandardView): ViewState {
  const declared = collectFields(view);
  return { view, declared, values: new Map(declared) };
}

function collectFields(root: StandardView): Array<[string, StandardFieldValue]> {
  const result: Array<[string, StandardFieldValue]> = [];
  const visit = (node: StandardView) => {
    if (node.type === "stack" || node.type === "row") {
      node.children.forEach(visit);
    } else if (node.type === "textField" || node.type === "numberField"
      || node.type === "select" || node.type === "toggle") {
      result.push([node.id, node.value]);
    }
  };
  visit(root);
  if (result.length > UI_LIMITS.maxFieldsPerView) throw new Error("invalid_extension_ui_view");
  return result;
}
