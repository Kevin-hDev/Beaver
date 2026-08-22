import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NewWakeupDialog } from "../new-wakeup-dialog";

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
    }]]]),
  }),
  withoutInteractiveOnlyModels: (groups: Map<string, unknown[]>) => groups,
}));

describe("NewWakeupDialog", () => {
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
});
