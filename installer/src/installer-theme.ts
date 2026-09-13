const palettes = {
  dark: ["dark", ""],
  light: ["light", ""],
  "astral-mist": ["dark", "astral-mist"],
  "cobalt-frost": ["light", "cobalt-frost"],
  "crimson-eclipse": ["dark", "crimson-eclipse"],
  "emerald-night": ["dark", "emerald-night"],
} as const;

export function applyInstallerTheme(): void {
  const requested = new URLSearchParams(window.location.search).get("theme");
  const allowed = import.meta.env.DEV && requested && requested in palettes;
  const selected = allowed
    ? (requested as keyof typeof palettes)
    : window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  const [theme, palette] = palettes[selected];
  document.documentElement.dataset.theme = theme;
  if (palette) document.documentElement.dataset.palette = palette;
  else delete document.documentElement.dataset.palette;
}
