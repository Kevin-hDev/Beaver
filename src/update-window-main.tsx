import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

const root = document.getElementById("update-root");
if (!root) throw new Error("update-window-root-missing");

createRoot(root).render(
  <StrictMode>
    <main aria-label="Mises à jour" data-tauri-drag-region />
  </StrictMode>,
);
