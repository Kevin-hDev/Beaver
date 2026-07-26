import { defineConfig } from "vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  plugins: [
    react(),
    babel({ presets: [reactCompilerPreset()] }),
    tailwindcss(),
  ],
  resolve: {
    alias: { "@": path.resolve(__dirname, "./src") },
  },
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
        mascot: path.resolve(__dirname, "mascot.html"),
      },
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  // CVE-2023-46115 : ne PAS exposer les variables d'env TAURI_ au frontend
  envPrefix: ["VITE_"],
});
