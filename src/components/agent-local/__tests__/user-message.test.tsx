import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { UserMessage } from "../user-message";

vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "agentLocal.showMore") return "Show more";
      if (key === "agentLocal.showLess") return "Show less";
      return key;
    },
  }),
}));

let measuredHeight = 100;
const originalScrollHeight = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "scrollHeight");

beforeEach(() => {
  localStorage.setItem("clgo-link-preview", "false");
  Object.defineProperty(HTMLElement.prototype, "scrollHeight", {
    configurable: true,
    get: () => measuredHeight,
  });
});

describe("UserMessage", () => {
  it("garde les messages courts sans bouton de dépliage", () => {
    measuredHeight = 100;

    render(<UserMessage content="Message court" />);

    expect(screen.getByText("Message court")).toBeTruthy();
    expect(screen.queryByText("Show more")).toBeNull();
  });

  it("affiche le Markdown complet dans la bulle utilisateur", () => {
    const { container } = render(
      <UserMessage content={"# Bonjour\n\n## Salut\n\n- Premier\n- Second\n\n---"} />,
    );

    expect(screen.getByRole("heading", { level: 1, name: "Bonjour" })).toBeTruthy();
    expect(screen.getByRole("heading", { level: 2, name: "Salut" })).toBeTruthy();
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(container.querySelector(".msg-user-content.chat-md-user")).toBeTruthy();
    expect(container.querySelector("hr")).toBeTruthy();
  });

  it("limite un long message et permet de le déplier puis replier", () => {
    measuredHeight = 900;

    const { container } = render(<UserMessage content={"Long message\n".repeat(40)} />);
    const content = container.querySelector(".msg-user-content");

    expect(screen.getByText("Show more")).toBeTruthy();
    expect(content).toHaveStyle({ maxHeight: "434px" });

    fireEvent.click(screen.getByText("Show more"));

    expect(screen.getByText("Show less")).toBeTruthy();
    expect(content).toHaveStyle({ maxHeight: "900px" });

    fireEvent.click(screen.getByText("Show less"));

    expect(screen.getByText("Show more")).toBeTruthy();
    expect(content).toHaveStyle({ maxHeight: "434px" });
  });

  it("rétablit les actions au survol après avoir annulé une édition", () => {
    const { container } = render(<UserMessage content="Message à modifier" onEdit={vi.fn()} />);
    const initialMessage = container.querySelector(".msg-user");

    fireEvent.mouseEnter(initialMessage!);
    expect(initialMessage).toHaveClass("msg-hovered");
    fireEvent.click(container.querySelector(".msg-action-btn")!);
    fireEvent.click(screen.getByText("agentLocal.cancel"));

    const restoredMessage = container.querySelector(".msg-user");
    expect(restoredMessage).not.toBe(initialMessage);
    fireEvent.mouseEnter(restoredMessage!);
    expect(restoredMessage).toHaveClass("msg-hovered");
  });
});

afterEach(() => {
  localStorage.removeItem("clgo-link-preview");
  measuredHeight = 100;
  if (originalScrollHeight) {
    Object.defineProperty(HTMLElement.prototype, "scrollHeight", originalScrollHeight);
  } else {
    Reflect.deleteProperty(HTMLElement.prototype, "scrollHeight");
  }
});
