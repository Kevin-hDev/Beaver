import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ToolItem } from "../tool-item";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretUp: () => <span />,
  Spinner: () => <span />,
}));
vi.mock("../tool-icons", () => ({
  ToolIcon: () => <span />,
}));
vi.mock("../tool-status-icon", () => ({
  ToolStatusIcon: () => <span />,
}));
vi.mock("@/components/file-preview/file-icon", () => ({
  FileIcon: () => <span />,
}));
vi.mock("../tool-result-markdown", () => ({
  ToolResultCode: ({ content }: { content: string }) => <pre>{content}</pre>,
  ToolResultMarkdown: ({ content }: { content: string }) => <pre>{content}</pre>,
}));
vi.mock("@/lib/tool-file-path", () => ({ isFileTool: () => false }));

describe("ToolItem shell stop", () => {
  it("déplie la commande exacte et son résultat depuis le libellé d’arrêt", () => {
    const command = "npm run dev -- --host 127.0.0.1";
    const { container, getByRole } = render(
      <ToolItem
        name="bash_control"
        summary={command}
        displayName="Arrêt du processus"
        displaySummary=""
        commandPreview={command}
        done
        result="Processus arrêté."
      />,
    );
    const toggle = getByRole("button", { name: "Arrêt du processus" });

    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(container.textContent).not.toContain(command);
    expect(container.textContent).not.toContain("Processus arrêté.");

    fireEvent.click(toggle);

    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(container.querySelector(".tb-accordion.tb-open")).not.toBeNull();
    expect(container.textContent).toContain(command);
    expect(container.textContent).toContain("Processus arrêté.");
  });
});
