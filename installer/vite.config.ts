import react from "@vitejs/plugin-react";
import path from "node:path";
import { defineConfig } from "vite";

const installerRoot = import.meta.dirname;

export default defineConfig({
  root: installerRoot,
  plugins: [react()],
  build: {
    outDir: path.join(installerRoot, "dist"),
    emptyOutDir: true,
    rollupOptions: {
      input: path.join(installerRoot, "index.html"),
    },
  },
  server: {
    port: 1421,
    strictPort: true,
    fs: {
      allow: [
        installerRoot,
        path.resolve(installerRoot, "../src/styles"),
        path.resolve(installerRoot, "../docs/fonctionnalites/install-&-update/assets"),
      ],
    },
  },
  envPrefix: ["VITE_"],
});
