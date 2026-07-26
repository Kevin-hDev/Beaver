import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { CloneSessionDialog } from "../clone-session-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  X: () => <span data-testid="close-icon" />,
}));

function renderDialog(overrides: Partial<React.ComponentProps<typeof CloneSessionDialog>> = {}) {
  const props = {
    canSummarize: true,
    busy: false,
    error: null,
    onCancel: vi.fn(),
    onAbort: vi.fn(),
    onSubmit: vi.fn(),
    ...overrides,
  };
  render(<CloneSessionDialog {...props} />);
  return props;
}

describe("CloneSessionDialog", () => {
  it("affiche les trois choix et envoie le focus personnalisé", () => {
    const props = renderDialog();

    fireEvent.click(screen.getByText("agentLocal.clone.summaryFocus"));
    fireEvent.change(screen.getByPlaceholderText("agentLocal.clone.focusPlaceholder"), {
      target: { value: "fichiers modifiés" },
    });
    fireEvent.click(screen.getByText("agentLocal.clone.create"));

    expect(props.onSubmit).toHaveBeenCalledWith("summary", "fichiers modifiés");
  });

  it("désactive les choix résumé quand le message est le dernier", () => {
    renderDialog({ canSummarize: false });

    expect(screen.getByText("agentLocal.clone.summary")).toBeDisabled();
    expect(screen.getByText("agentLocal.clone.summaryFocus")).toBeDisabled();
  });

  it("masque les choix pendant le résumé et annule le backend", () => {
    const props = renderDialog({ busy: true });

    expect(screen.getByText("agentLocal.clone.running")).toBeTruthy();
    expect(screen.queryByText("agentLocal.clone.cut")).toBeNull();
    fireEvent.click(screen.getByText("agentLocal.cancel"));

    expect(props.onAbort).toHaveBeenCalledOnce();
    expect(props.onCancel).not.toHaveBeenCalled();
  });
});
