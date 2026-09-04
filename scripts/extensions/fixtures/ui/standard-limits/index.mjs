export const negativeCases = Object.freeze({
  contributions: 33,
  themes: 9,
  actions: 65,
  viewNodes: 257,
  viewDepth: 13,
  fields: 33,
  options: 65,
  textChars: 2001,
  extensionBytes: 262145,
  actionPayloadBytes: 65537,
  actionResultBytes: 262145,
  protectedMutations: Object.freeze([
    { placement: "app.navigation.primary", targetId: "beaver.settings", operation: "remove" },
    { placement: "settings.navigation.integrations", targetId: "beaver.extensions", operation: "remove" },
  ]),
});

export default function activate(beaver) {
  for (let index = 0; index < negativeCases.contributions; index += 1) {
    beaver.ui.register({
      type: "action", id: `overflow-${index}`, placement: "app.toolbar.primary",
      order: index, label: { default: "Overflow" }, actionId: `overflow-action-${index}`,
    });
  }
}
