import "./style.css";

export function activate(context: {
  mount: (placement: string, render: (container: HTMLElement) => (() => void)) => void;
}) {
  context.mount("app.toolbar.primary", (container) => {
    const element = document.createElement("button");
    element.className = "acceptance-advanced-button";
    element.textContent = "Advanced acceptance";
    container.append(element);
    return () => element.remove();
  });
  return () => { document.documentElement.dataset.acceptanceAdvancedCleanup = "done"; };
}
