import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ToolDetailRow } from "../tool-detail-row";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key === "errors.admission.gatewayBusy"
      ? "Trop de messages sont déjà en cours."
      : key,
  }),
}));
vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretUp: () => <span />,
  Spinner: () => <span />,
}));
vi.mock("../tool-icons", () => ({ ToolIcon: () => <span /> }));
vi.mock("../tool-status-icon", () => ({
  ToolStatusIcon: ({ message }: { message?: string }) => (
    <span data-testid="status-icon-error" data-message={message ?? ""} />
  ),
}));
vi.mock("@/components/file-preview/file-icon", () => ({ FileIcon: () => <span /> }));
vi.mock("../tool-result-markdown", () => ({
  ToolResultCode: ({ content }: { content: string }) => <pre>{content}</pre>,
  ToolResultMarkdown: ({ content }: { content: string }) => <pre>{content}</pre>,
}));

describe("refus d'admission d'un outil", () => {
  it("traduit le refus sans rendre le code brut", () => {
    const { container, getByTestId } = render(
      <ToolDetailRow
        tool={{
          name: "delegate",
          summary: "worker",
          result: "gateway-busy",
          is_error: true,
        }}
        previousTools={[]}
      />,
    );

    expect(container.textContent).not.toContain("gateway-busy");
    expect(getByTestId("status-icon-error")).toHaveAttribute(
      "data-message",
      "Trop de messages sont déjà en cours.",
    );
  });
});
