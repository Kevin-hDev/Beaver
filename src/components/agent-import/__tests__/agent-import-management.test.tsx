import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";
import type { AgentImportItem, AgentSourceSummary } from "@/types/agent-import";
import { AgentImportWizard } from "../agent-import-wizard";

vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));

const invokeMock = vi.mocked(invoke);
const showToastMock = vi.mocked(showToast);

function item(id: string, kind: AgentImportItem["kind"]): AgentImportItem {
  return {
    id,
    name: id,
    description: "",
    sourceId: "claude",
    sourceName: "Claude Code",
    kind,
    selected: true,
    available: true,
    updateAvailable: false,
  };
}

function configuredSource(enabled: boolean): AgentSourceSummary {
  return {
    id: "claude",
    displayName: "Claude Code",
    status: "detected",
    partial: false,
    configured: true,
    enabled,
    documents: [item("CLAUDE.md", "document")],
    rules: [item("style", "rule")],
    skills: [item("review", "skill")],
  };
}

beforeEach(() => {
  invokeMock.mockReset();
  showToastMock.mockReset();
  invokeMock.mockImplementation((command) => {
    if (command === "scan_external_agent_sources") {
      return Promise.resolve([configuredSource(true)]);
    }
    if (command === "save_external_agent_source_selection") {
      return Promise.resolve({ saved: true, conflicts: [] });
    }
    return Promise.resolve(undefined);
  });
});

describe("gestion d'une migration existante", () => {
  it("marque la source et ne confirme que les changements", async () => {
    render(<AgentImportWizard />);

    expect(await screen.findByText(i18n.t("agentImport.card.migrated"))).toBeInTheDocument();
    fireEvent.click(screen.getByText("Claude Code"));

    expect(screen.getByRole("img", { name: "Claude Code" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Claude Code" })).not.toBeInTheDocument();
    expect(screen.getByText(i18n.t("agentImport.card.active"))).toBeInTheDocument();
    expect(screen.getByText(i18n.t("agentImport.detail.imported"))).toBeInTheDocument();
    expect(screen.getByRole("button", {
      name: i18n.t("agentImport.actions.disable"),
    })).toBeInTheDocument();
    expect(screen.queryByRole("button", {
      name: i18n.t("agentImport.actions.confirmSource"),
    })).not.toBeInTheDocument();

    const ruleSection = screen
      .getByText(i18n.t("agentImport.sections.rules"))
      .closest("section");
    fireEvent.click(within(ruleSection as HTMLElement).getByRole("checkbox"));

    expect(screen.getByRole("button", {
      name: i18n.t("agentImport.actions.confirmChanges"),
    })).toBeInTheDocument();
  });

  it("propose explicitement de réactiver une source désactivée", async () => {
    invokeMock.mockImplementation((command) => {
      if (command === "scan_external_agent_sources") {
        return Promise.resolve([configuredSource(false)]);
      }
      if (command === "save_external_agent_source_selection") {
        return Promise.resolve({ saved: true, conflicts: [] });
      }
      return Promise.resolve(undefined);
    });
    render(<AgentImportWizard />);

    expect(await screen.findByText(i18n.t("agentImport.card.disabled"))).toBeInTheDocument();
    expect(screen.queryByText(i18n.t("agentImport.card.migrated"))).not.toBeInTheDocument();
    fireEvent.click(await screen.findByText("Claude Code"));
    fireEvent.click(screen.getByRole("button", {
      name: i18n.t("agentImport.actions.reactivate"),
    }));

    await waitFor(() => {
      const saveCall = invokeMock.mock.calls.find(
        ([command]) => command === "save_external_agent_source_selection",
      );
      expect(saveCall?.[1]).toMatchObject({
        selection: { enabled: true },
      });
    });
  });

  it("signale un remplacement refusé au lieu de rester silencieux", async () => {
    const source = configuredSource(true);
    source.configured = false;
    source.enabled = false;
    let saveAttempt = 0;
    invokeMock.mockImplementation((command) => {
      if (command === "scan_external_agent_sources") {
        return Promise.resolve([source]);
      }
      if (command === "save_external_agent_source_selection") {
        saveAttempt += 1;
        return Promise.resolve({
          saved: false,
          conflicts: saveAttempt === 1 ? ["CLAUDE.md"] : [],
        });
      }
      return Promise.resolve(undefined);
    });
    render(<AgentImportWizard />);

    fireEvent.click(await screen.findByText("Claude Code"));
    fireEvent.click(screen.getByRole("button", {
      name: i18n.t("agentImport.actions.confirmSource"),
    }));
    fireEvent.click(await screen.findByRole("button", {
      name: i18n.t("agentImport.conflict.replace"),
    }));

    await waitFor(() => {
      expect(showToastMock).toHaveBeenCalledWith(
        i18n.t("errors.saveFailed"),
        "error",
      );
    });
  });
});
