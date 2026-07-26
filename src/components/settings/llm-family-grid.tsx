import type { FamilyGroup } from "./llm-types";
import "./llm-explorer.css";

interface LlmFamilyGridProps {
  families: FamilyGroup[];
  onSelect: (family: string) => void;
}

export function LlmFamilyGrid({ families, onSelect }: LlmFamilyGridProps) {
  return (
    <div className="llm-family-grid">
      {families.map((f) => (
        <button
          key={f.name}
          className="llm-family-card"
          onClick={() => onSelect(f.name)}
        >
          <span className="llm-family-name">{f.name}</span>
          <span className="llm-family-count">({f.count})</span>
        </button>
      ))}
    </div>
  );
}
