import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { NewWakeupDialog } from "../new-wakeup-dialog";
import type { WakeupDefinition } from "@/types/wakeup";

const onCreate = vi.fn().mockResolvedValue(undefined);

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/hooks/use-projects", () => ({
  useProjects: () => ({
    projects: [{
      id: "project-1",
      name: "Beaver",
      path: "/tmp/beaver",
      order: 0,
      created_at: "2026-08-22T00:00:00Z",
    }],
  }),
}));

vi.mock("@/hooks/use-available-models", () => ({
  useAvailableModels: () => ({
    groups: new Map([["ollama", [{
      id: "tool-model",
      provider_id: "ollama",
      provider_name: "Ollama",
      auth_source: "local",
      is_local: true,
      supports_tools: true,
    }, {
      id: "tool-model-2", provider_id: "ollama", provider_name: "Ollama",
      auth_source: "local", is_local: true, supports_tools: true,
    }]], ["codex-oauth", [{
      id: "codex-model", provider_id: "codex-oauth", provider_name: "Codex",
      auth_source: "oauth", is_local: false, supports_tools: true,
    }]]]),
  }),
  withoutInteractiveOnlyModels: (groups: Map<string, unknown[]>) => groups,
}));

describe("NewWakeupDialog", () => {
  beforeEach(() => onCreate.mockClear());

  it("enregistre le projet facultatif choisi pour le réveil", async () => {
    render(
      <NewWakeupDialog
        initial={null}
        onClose={vi.fn()}
        onCreate={onCreate}
        onUpdate={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByPlaceholderText("heartbeat.form.namePlaceholder"), {
      target: { value: "Revue" },
    });
    fireEvent.change(screen.getByPlaceholderText("heartbeat.form.promptPlaceholder"), {
      target: { value: "Analyse le projet" },
    });
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.form.project" }));
    fireEvent.click(screen.getByRole("option", { name: "Beaver" }));
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.form.create" }));

    await waitFor(() => expect(onCreate).toHaveBeenCalledWith(expect.objectContaining({
      project_id: "project-1",
    })));
  });

  it("propose ponctuel, cron et délai après exécution sans éditeur cron visuel", () => {
    render(<NewWakeupDialog initial={null} onClose={vi.fn()} onCreate={onCreate} onUpdate={vi.fn()} />);
    expect(screen.getByRole("button", { name: "heartbeat.form.scheduleKind.once" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.form.scheduleKind.cron" }));
    expect(screen.getByRole("textbox", { name: "heartbeat.form.cronExpression" })).toHaveValue("0 8 * * *");
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.form.scheduleKind.after_completion" }));
    expect(screen.getByRole("spinbutton", { name: "heartbeat.form.delayMinutes" })).toHaveValue(10);
  });

  it("garde le fournisseur fixe en édition et limite les modèles à ce fournisseur", () => {
    const initial: WakeupDefinition = {
      id: "w1", revision: 1, name: "CI", provider: "ollama", model: "tool-model",
      target: { mode: "new_session", project_id: null },
      schedule: { kind: "after_completion", delay_minutes: 10 }, status: "active",
      description: null, prompt: "Vérifie", creator_session_id: null,
      created_at: "2026-09-11T00:00:00Z", anchor_at: null,
    };
    render(<NewWakeupDialog initial={initial} onClose={vi.fn()} onCreate={onCreate} onUpdate={vi.fn()} />);
    expect(screen.queryByRole("button", { name: "heartbeat.form.provider" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.form.model" }));
    expect(screen.getByRole("option", { name: "tool-model-2" })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: "codex-model" })).not.toBeInTheDocument();
  });
});
