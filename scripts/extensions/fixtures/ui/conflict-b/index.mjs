export default function activate(beaver) {
  beaver.ui.register({
    type: "tab", id: "peer", placement: "app.navigation.primary", order: 50,
    label: { default: "Conflict B" }, operation: "move", targetId: "beaver.settings",
    detail: { type: "text", text: { default: "B" } },
  });
}
