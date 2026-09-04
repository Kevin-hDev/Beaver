export default function activate(beaver) {
  beaver.ui.register({
    type: "theme", id: "midnight", order: 20,
    label: { default: "Acceptance midnight" }, base: "dark",
    tokens: { "--void": "#080B12", "--ink": "#F5F7FF", "--pulse": "#6E8DFF" },
  });
}
