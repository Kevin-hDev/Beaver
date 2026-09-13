import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { InstallerApp } from "./installer-app";

const root = document.getElementById("root");

if (!root) throw new Error("installer root unavailable");

createRoot(root).render(
  <StrictMode>
    <InstallerApp />
  </StrictMode>,
);
