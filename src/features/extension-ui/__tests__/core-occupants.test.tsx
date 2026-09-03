/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { allowsThirdPartyComposerUi } from "../slot-contexts";
import { SlotProvider } from "../slot-provider";
import { SlotRenderer } from "../slot-renderer";

afterEach(cleanup);

describe("core slot occupants", () => {
  it("renders the four main tabs from the provider registry", () => {
    render(
      <SlotProvider>
        <SlotRenderer
          placement="app.navigation.primary"
          context={null}
          render={(occupant) => <button>{occupant.target}</button>}
        />
      </SlotProvider>,
    );

    expect(screen.getAllByRole("button").map(({ textContent }) => textContent)).toEqual([
      "agent-local", "heartbeat", "personality", "settings",
    ]);
  });

  it("renders each toolbar action and the core composer menu exactly once", () => {
    render(
      <SlotProvider>
        <SlotRenderer
          placement="app.toolbar.primary"
          context={null}
          render={(occupant) => <span data-testid="toolbar-action">{occupant.target}</span>}
        />
        <SlotRenderer
          placement="agent.composer.leading"
          source="core"
          context={null}
          render={(occupant) => <span data-testid="composer-action">{occupant.target}</span>}
        />
      </SlotProvider>,
    );

    expect(screen.getAllByTestId("toolbar-action")).toHaveLength(6);
    expect(screen.getAllByTestId("composer-action")).toHaveLength(1);
    expect(screen.getByTestId("composer-action").textContent).toBe("plus-menu");
  });

  it("mounts the third-party composer insertion only in Agent and Plan", () => {
    expect(allowsThirdPartyComposerUi("chat", false)).toBe(false);
    expect(allowsThirdPartyComposerUi("auto", false)).toBe(true);
    expect(allowsThirdPartyComposerUi("manual", true)).toBe(true);
  });
});
