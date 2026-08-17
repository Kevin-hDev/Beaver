import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render } from "@testing-library/react";
import { XSS_PAYLOADS, findViolations } from "@/lib/test-utils/xss-corpus";

const opened: string[] = [];

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: (url: string) => {
    opened.push(url);
    return Promise.resolve();
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  Channel: class {},
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

import { ChatMarkdown } from "../chat-markdown";

describe("ChatMarkdown - batterie XSS", () => {
  beforeEach(() => {
    opened.length = 0;
  });

  for (const payload of XSS_PAYLOADS) {
    it(`neutralise : ${payload.name}`, () => {
      const { container, unmount } = render(<ChatMarkdown content={payload.input} />);

      const violations = findViolations(container);
      expect(violations, `violations pour « ${payload.name} » : ${violations.join(", ")}`).toEqual([]);

      /* Cliquer chaque lien rendu ne doit ouvrir aucun protocole dangereux. */
      for (const link of Array.from(container.querySelectorAll("a"))) {
        fireEvent.click(link);
      }
      for (const url of opened) {
        expect(url).toMatch(/^(https?|mailto):/i);
      }
      unmount();
    });
  }

  it("ne laisse aucun marqueur d'exécution global après tout le corpus", () => {
    for (const payload of XSS_PAYLOADS) {
      const { unmount } = render(<ChatMarkdown content={payload.input} />);
      unmount();
    }
    const win = window as unknown as Record<string, unknown>;
    expect(win.__xss).toBeUndefined();
    expect(win.__pwned).toBeUndefined();
  });
});
