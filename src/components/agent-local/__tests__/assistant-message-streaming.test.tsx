import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { AssistantMessage } from "../assistant-message";

afterEach(cleanup);

vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));
vi.mock("@/hooks/use-hover-class", () => ({ useHoverClass: () => ({ current: null }) }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("../messages.css", () => ({}));
vi.mock("../chat-markdown.css", () => ({}));

describe("AssistantMessage streaming", () => {
  it("ne rend plus le curseur de stream isolé", () => {
    const { queryByText } = render(<AssistantMessage content="" isStreaming />);

    expect(queryByText("▐")).toBeNull();
  });

  it("applique la variante trace aux textes de travail du stream", () => {
    const { container } = render(<AssistantMessage content="note" variant="trace" />);

    expect(container.querySelector(".msg-assistant-trace")).toBeTruthy();
  });
});
