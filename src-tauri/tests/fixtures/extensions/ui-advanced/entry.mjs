document.documentElement.dataset.extensionUiRuntimeProof = "loaded";
export const extensionUiRuntimeProof = "loaded";

export function activate(context) {
  context.mount("app.toolbar.primary", (container) => {
    container.dataset.extensionUiFixture = "mounted";
    return () => { delete container.dataset.extensionUiFixture; };
  });
  return () => { document.documentElement.dataset.extensionUiFixtureCleanup = "done"; };
}

export function deactivate() {
  document.documentElement.dataset.extensionUiFixtureDeactivate = "done";
}
