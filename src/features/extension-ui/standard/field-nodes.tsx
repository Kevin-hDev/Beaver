import { CustomSelect } from "@/components/ui/custom-select";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { localizedText } from "./localized-text";
import type { StandardFieldValue, StandardView } from "./types";

type FieldNode = Extract<StandardView, { type: "textField" | "numberField" | "select" | "toggle" }>;

export function StandardFieldNode({
  node,
  value,
  onChange,
}: {
  node: FieldNode;
  value: StandardFieldValue;
  onChange: (value: StandardFieldValue) => void;
}) {
  const label = localizedText(node.label);
  const htmlId = `xui-${safeDomId(node.id)}`;
  if (node.type === "toggle") {
    return (
      <div className="xui-field xui-toggle-field">
        <span id={`${htmlId}-label`} className="xui-label">{label}</span>
        <ToggleSwitch
          id={htmlId}
          checked={value === true}
          onCheckedChange={onChange}
          ariaLabel={label}
        />
      </div>
    );
  }
  if (node.type === "select") {
    return (
      <div className="xui-field" id={`${htmlId}-label`}>
        <span className="xui-label">{label}</span>
        <CustomSelect
          value={typeof value === "string" ? value : ""}
          options={node.options.map((option) => ({
            value: option.value,
            label: localizedText(option.label),
          }))}
          onChange={onChange}
          ariaLabel={label}
        />
      </div>
    );
  }
  const inputValue = typeof value === "string" || typeof value === "number" ? value : "";
  return (
    <label className="xui-field" htmlFor={htmlId}>
      <span className="xui-label">{label}</span>
      <input
        id={htmlId}
        className="form-input xui-input"
        type={node.type === "numberField" ? "number" : "text"}
        value={inputValue}
        onChange={(event) => onChange(node.type === "numberField"
          ? parseNumber(event.target.value)
          : event.target.value)}
      />
    </label>
  );
}

function parseNumber(value: string): number | null {
  if (value === "") return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function safeDomId(value: string): string {
  return value.replaceAll(".", "-");
}
