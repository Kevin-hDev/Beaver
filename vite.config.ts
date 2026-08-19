import { defineConfig } from "vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// Windows place Cargo à la racine pour partager CEF, updater et artefacts ; les
// autres lancements gardent la cible sous src-tauri. Vite ne doit surveiller aucun des deux.
const cargoTargetDirs = [
  path.resolve(import.meta.dirname, "src-tauri/target"),
  path.resolve(import.meta.dirname, "target"),
];

export default defineConfig({
  plugins: [
    react(),
    babel({ presets: [reactCompilerPreset()] }),
    tailwindcss(),
  ],
  resolve: {
    alias: { "@": path.resolve(import.meta.dirname, "./src") },
  },
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(import.meta.dirname, "index.html"),
        mascot: path.resolve(import.meta.dirname, "mascot.html"),
      },
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: (watchedPath) =>
        cargoTargetDirs.some(
          (targetDir) =>
            watchedPath === targetDir ||
            watchedPath.startsWith(`${targetDir}${path.sep}`),
        ),
    },
  },
  // CVE-2023-46115 : ne PAS exposer les variables d'env TAURI_ au frontend
  envPrefix: ["VITE_"],
});
