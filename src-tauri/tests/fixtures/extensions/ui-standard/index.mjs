export default function activate(api) {
  api.ui.register({
    type: "action",
    id: "toolbar-proof",
    placement: "app.toolbar.primary",
    order: 10,
    label: { default: "Run proof", fr: "Lancer la preuve" },
    icon: "sparkle",
    actionId: "run-proof",
  });
  api.ui.onAction("run-proof", ({ fields }) => ({
    type: "notification",
    level: "success",
    message: {
      default: `Proof ${String(fields?.value ?? "ready")}`,
      fr: `Preuve ${String(fields?.value ?? "prête")}`,
    },
  }));
}
