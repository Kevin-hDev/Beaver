export default function activate(beaver) {
  const label = { default: "Acceptance" };
  const detail = { type: "text", text: label };
  beaver.ui.register({
    type: "tab", id: "navigation", placement: "app.navigation.primary",
    order: 40, label, detail,
  });
  beaver.ui.register({
    type: "settingsTab", id: "settings", placement: "settings.navigation.preferences",
    order: 40, label, detail,
  });
  beaver.ui.register({
    type: "action", id: "toolbar", placement: "app.toolbar.primary",
    order: 40, label, actionId: "run-toolbar",
  });
  beaver.ui.register({
    type: "action", id: "composer", placement: "agent.composer.leading",
    order: 40, label, actionId: "run-composer",
  });
  beaver.ui.onAction("run-toolbar", result);
  beaver.ui.onAction("run-composer", result);
}

function result() {
  return { type: "notification", level: "success", message: { default: "Accepted" } };
}
