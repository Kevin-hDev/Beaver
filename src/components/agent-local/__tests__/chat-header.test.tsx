import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChatHeader } from "../chat-header";
import type { SessionSummaryHookState } from "@/hooks/use-session-summary";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

afterEach(() => {
  cleanup();
});

const noop = () => {};

describe("ChatHeader", () => {
  it("masque la ligne séparatrice sans session active", () => {
    const { container } = render(
      <ChatHeader
        sessionName={null}
        sessionId={null}
        terminalOpen={false}
        previewOpen={false}
        onToggleTerminal={noop}
        onTogglePreview={noop}
      />,
    );

    expect(container.querySelector(".chat-header-empty")).not.toBeNull();
  });

  it("garde la ligne séparatrice avec une session active", () => {
    const { container } = render(
      <ChatHeader
        sessionName="Session"
        sessionId="s1"
        terminalOpen={false}
        previewOpen={false}
        onToggleTerminal={noop}
        onTogglePreview={noop}
      />,
    );

    expect(container.querySelector(".chat-header-empty")).toBeNull();
  });

  it("affiche le bouton résumé même si l'état Git est absent", () => {
    const { getByRole } = render(
      <ChatHeader
        sessionName="Session"
        sessionId="s1"
        terminalOpen={false}
        previewOpen={false}
        onToggleTerminal={noop}
        onTogglePreview={noop}
        sessionSummary={summary()}
      />,
    );

    expect(getByRole("button", { name: "agentLocal.sessionSummary.tooltip" })).toBeTruthy();
  });
});

function summary(): SessionSummaryHookState {
  return {
    session: null,
    changes: { additions: 0, deletions: 0, files: 0 },
    todoRuns: [],
    plans: [],
    subagents: [],
  };
}
