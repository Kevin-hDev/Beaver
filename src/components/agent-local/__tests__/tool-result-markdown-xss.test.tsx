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

import { ToolResultMarkdown } from "../tool-result-markdown";

describe("ToolResultMarkdown - batterie XSS", () => {
  beforeEach(() => {
    opened.length = 0;
  });

  for (const payload of XSS_PAYLOADS) {
    it(`neutralise : ${payload.name}`, () => {
      const { container, unmount } = render(<ToolResultMarkdown content={payload.input} />);

      const violations = findViolations(container);
      expect(violations, `violations pour « ${payload.name} » : ${violations.join(", ")}`).toEqual([]);

      for (const link of Array.from(container.querySelectorAll("a"))) {
        fireEvent.click(link);
      }
      for (const url of opened) {
        expect(url).toMatch(/^(https?|mailto):/i);
      }
      unmount();
    });
  }
});
