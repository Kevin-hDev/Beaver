import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { InstallerApp } from "./installer-app";
import { installerPreviewApi } from "./installer-preview";
import { applyInstallerTheme } from "./installer-theme";
import "../../src/styles/tokens.css";
import "../../src/styles/tokens-icon-sizes.css";
import "../../src/styles/themes/light.css";
import "../../src/styles/themes/dark.css";
import "../../src/styles/themes/emerald-night.css";
import "../../src/styles/themes/cobalt-frost.css";
import "../../src/styles/themes/astral-mist.css";
import "../../src/styles/themes/crimson-eclipse.css";
import "../../src/styles/relief.css";
import "../../src/styles/focus.css";
import "../../src/styles/buttons.css";
import "../../src/styles/fields.css";
import "../../src/styles/callout.css";
import "../../src/styles/operation-progress.css";
import "./installer.css";
import "./installer-progress.css";
import "./installer-result.css";

const root = document.getElementById("root");

applyInstallerTheme();

const previewApi = import.meta.env.DEV ? installerPreviewApi() : undefined;

if (root) {
  createRoot(root).render(
    <StrictMode>
      <InstallerApp api={previewApi} />
    </StrictMode>,
  );
}
