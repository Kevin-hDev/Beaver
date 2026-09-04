export default function activate(beaver) {
  beaver.ui.register({
    type: "theme", id: "outside-contract", order: 20,
    label: { default: "Invalid acceptance theme" }, base: "dark",
    tokens: { "--not-a-public-token": "#010203" },
  });
}
