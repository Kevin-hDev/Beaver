import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EditMessage } from "../edit-message";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      "agentLocal.editMessage": "Modifier et relancer",
      "agentLocal.cancel": "Annuler",
      "agentLocal.send": "Envoyer",
    })[key] ?? key,
  }),
}));

let measuredHeight = 20;
const originalScrollHeight = Object.getOwnPropertyDescriptor(
  HTMLTextAreaElement.prototype,
  "scrollHeight",
);

beforeEach(() => {
  Object.defineProperty(HTMLTextAreaElement.prototype, "scrollHeight", {
    configurable: true,
    get: () => measuredHeight,
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  measuredHeight = 20;
  if (originalScrollHeight) {
    Object.defineProperty(HTMLTextAreaElement.prototype, "scrollHeight", originalScrollHeight);
  } else {
    Reflect.deleteProperty(HTMLTextAreaElement.prototype, "scrollHeight");
  }
});

describe("EditMessage", () => {
  it("utilise deux lignes minimum puis grandit avec le texte", async () => {
    render(<EditMessage initialContent="Court" onSave={vi.fn()} onCancel={vi.fn()} />);
    const textarea = screen.getByRole("textbox", { name: "Modifier et relancer" });

    expect(textarea).toHaveAttribute("rows", "2");
    expect(textarea).toHaveStyle({ height: "44px" });

    measuredHeight = 150;
    fireEvent.change(textarea, { target: { value: "Texte\nplus\nlong" } });
    await waitFor(() => expect(textarea).toHaveStyle({ height: "150px" }));
  });

  it("se limite à vingt lignes et garde la fin visible", async () => {
    render(<EditMessage initialContent="Début" onSave={vi.fn()} onCancel={vi.fn()} />);
    const textarea = screen.getByRole("textbox", { name: "Modifier et relancer" });
    const longText = Array.from({ length: 30 }, (_, index) => `Ligne ${index + 1}`).join("\n");

    measuredHeight = 900;
    fireEvent.change(textarea, { target: { value: longText } });

    await waitFor(() => {
      expect(textarea).toHaveStyle({ height: "434px" });
      expect(textarea).toHaveClass("em-textarea-scroll");
      expect(textarea.scrollTop).toBe(900);
    });
  });

  it("envoie avec Cmd+Entrée et annule avec Échap", () => {
    const onSave = vi.fn();
    const onCancel = vi.fn();
    render(<EditMessage initialContent="Message modifié" onSave={onSave} onCancel={onCancel} />);
    const textarea = screen.getByRole("textbox", { name: "Modifier et relancer" });

    fireEvent.keyDown(textarea, { key: "Enter", metaKey: true });
    expect(onSave).toHaveBeenCalledWith("Message modifié");

    fireEvent.keyDown(textarea, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("garde une largeur stable dès son affichage", () => {
    const frames: FrameRequestCallback[] = [];
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => {
      frames.push(callback);
      return frames.length;
    });
    const { container } = render(
      <EditMessage initialContent="Message stable" onSave={vi.fn()} onCancel={vi.fn()} />,
    );
    const root = container.querySelector(".em-root");
    const initialClassName = root?.className;

    act(() => {
      frames.splice(0).forEach((callback) => callback(0));
    });

    expect(root).toHaveClass("chat-column-surface", "em-root");
    expect(root).not.toHaveClass("em-root-expanded");
    expect(root?.className).toBe(initialClassName);
  });
});
