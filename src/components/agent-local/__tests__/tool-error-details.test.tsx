import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ToolDetailRow } from "../tool-detail-row";

afterEach(cleanup);

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key === "agentLocal.toolActivity.resultTruncated"
    ? "Résultat tronqué." : key }),
}));
vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretUp: () => <span />,
  Spinner: () => <span />,
}));
vi.mock("../tool-icons", () => ({ ToolIcon: () => <span /> }));
vi.mock("../tool-status-icon", () => ({ ToolStatusIcon: () => <span /> }));
vi.mock("@/components/file-preview/file-icon", () => ({ FileIcon: () => <span /> }));
vi.mock("../tool-result-markdown", () => ({
  ToolResultCode: ({ content }: { content: string }) => <pre>{content}</pre>,
  ToolResultMarkdown: ({ content }: { content: string }) => <pre>{content}</pre>,
}));

describe("détails des erreurs d'outil", () => {
  it("rend une erreur web dépliable sans secret ni chemin interne", () => {
    const { container, getByRole } = render(
      <ToolDetailRow
        tool={{
          name: "web_fetch",
          summary: "https://example.test",
          result: "HTTP 500\nFile: /Users/dev/private.txt\ntoken=very-secret-token\nRetry failed",
          is_error: true,
          error: {
            code: "web_fetch_failed",
            category: "external",
            retryable: true,
          },
          warnings: ["La réponse distante était incomplète."],
          truncated: true,
        }}
        previousTools={[]}
      />,
    );

    const toggle = getByRole("button", { name: "web_fetch" });
    fireEvent.click(toggle);

    expect(container.textContent).toContain("HTTP 500");
    expect(container.textContent).toContain("Retry failed");
    expect(container.textContent).toContain("web_fetch_failed");
    expect(container.textContent).toContain("La réponse distante était incomplète.");
    expect(container.textContent).toContain("Résultat tronqué.");
    expect(container.textContent).not.toContain("/Users/dev/private.txt");
    expect(container.textContent).not.toContain("very-secret-token");
  });

  it("affiche les avertissements et la troncature dans le même accordéon", () => {
    const { container, getByRole } = render(
      <ToolDetailRow
        tool={{
          name: "custom_extension_tool",
          summary: "external action",
          result: "match",
          status: "partial",
          warnings: ["Un dossier n'a pas pu être lu."],
          truncated: true,
        }}
        previousTools={[]}
      />,
    );

    fireEvent.click(getByRole("button"));
    expect(container.textContent).toContain("match");
    expect(container.textContent).toContain("Un dossier n'a pas pu être lu.");
    expect(container.textContent).toContain("Résultat tronqué.");
  });
});
