import { ThemedIcon } from "./themed-icon";
import ollamaDark from "@/assets/ollama-icon-dark.svg";
import ollamaLight from "@/assets/ollama-icon-light.svg";
import "./ollama-brand-icon.css";

export function OllamaBrandIcon({ size = 40 }: { size?: number | string }) {
  return (
    <ThemedIcon
      darkSrc={ollamaDark}
      lightSrc={ollamaLight}
      className="obi-icon"
      size={size}
      style={{ borderRadius: "var(--radius-sm)" }}
    />
  );
}
