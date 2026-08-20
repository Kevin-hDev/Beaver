import { render } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { describe, expect, it, vi } from "vitest";
import { WorkStreamSummary } from "../work-stream-summary";

const chatCss = readFileSync("src/components/agent-local/chat.css", "utf8");
const workSummaryCss = readFileSync(
  "src/components/agent-local/work-stream-summary.css",
  "utf8",
);

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => null,
  CaretRight: () => null,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: () => "A travaillé" }),
}));

describe("WorkStreamSummary layout", () => {
  it("reste aligné avec la réponse quand sa feuille charge après chat.css", () => {
    const style = document.createElement("style");
    const columnRule = chatCss.match(/\.chat-messages > \* \{[^}]+\}/u)?.[0];
    expect(columnRule).toBeDefined();
    style.textContent = `${columnRule}\n${workSummaryCss}`;
    document.head.append(style);

    const { container, unmount } = render(
      <div className="chat-messages">
        <WorkStreamSummary>phase de travail</WorkStreamSummary>
        <div className="msg-assistant">réponse finale</div>
      </div>,
    );
    const work = container.querySelector<HTMLElement>(".wss-root");
    const answer = container.querySelector<HTMLElement>(".msg-assistant");

    expect(work).not.toBeNull();
    expect(answer).not.toBeNull();
    const answerStyle = getComputedStyle(answer!);
    const workStyle = getComputedStyle(work!);
    expect(answerStyle.marginLeft).toBe("auto");
    expect(answerStyle.marginRight).toBe("auto");
    expect(workStyle.marginLeft).toBe("auto");
    expect(workStyle.marginRight).toBe("auto");
    expect(workStyle.maxWidth).toBe(answerStyle.maxWidth);
    expect(workStyle.paddingLeft).toBe(answerStyle.paddingLeft);
    expect(workStyle.paddingRight).toBe(answerStyle.paddingRight);

    unmount();
    style.remove();
  });
});
