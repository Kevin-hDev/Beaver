import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@/i18n";
import "@/styles/global.css";
import { UpdateProgressWindow } from "@/components/updates/window/update-progress-window";

const root = document.getElementById("update-root");
if (!root) throw new Error("update-window-root-missing");

createRoot(root).render(
  <StrictMode>
    <UpdateProgressWindow />
  </StrictMode>,
);
