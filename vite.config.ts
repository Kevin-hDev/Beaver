import { defineConfig } from "vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

const rustTargetDir = path.resolve(import.meta.dirname, "src-tauri/target");

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
        watchedPath === rustTargetDir ||
        watchedPath.startsWith(`${rustTargetDir}${path.sep}`),
    },
  },
  // CVE-2023-46115 : ne PAS exposer les variables d'env TAURI_ au frontend
  envPrefix: ["VITE_"],
});
