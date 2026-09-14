/* @vitest-environment jsdom */
import "@testing-library/jest-dom/vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { InstallerApp, type InstallerApi } from "./installer-app";
import type { InstallerEvent, InstallerSnapshot } from "./installer-contract.generated";

function snapshot(overrides: Partial<InstallerSnapshot> = {}): InstallerSnapshot {
  return {
    version: "1.4.2",
    destination: "/Applications",
    installedVersion: null,
    beaverRunning: false,
    phase: "ready",
    stepIndex: 0,
    stepCount: 5,
    progressMode: "indeterminate",
    percent: null,
    canCancel: false,
    outcome: null,
    errorKey: null,
    ...overrides,
  };
}

function fixture(initial = snapshot()) {
  let publish: ((event: InstallerEvent) => void) | undefined;
  const api: InstallerApi = {
    snapshot: vi.fn().mockResolvedValue(initial),
    chooseDirectory: vi.fn().mockResolvedValue(snapshot({ destination: "/Users/kevin/Apps" })),
    start: vi.fn().mockImplementation((onEvent: (event: InstallerEvent) => void) => {
      publish = onEvent;
      return Promise.resolve();
    }),
    cancel: vi.fn().mockResolvedValue({
      sequence: 1,
      snapshot: initial,
      completedStepDurationsMs: [],
      logKey: null,
    }),
    launch: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  };
  return {
    api,
    publish(event: InstallerEvent) {
      act(() => publish?.(event));
    },
  };
}

beforeEach(() => {
  document.documentElement.lang = "fr";
  document.documentElement.dataset.theme = "dark";
});

describe("Beaver Installer", () => {
  it("sort de l'attente si le snapshot initial échoue et permet de réessayer", async () => {
    const setup = fixture();
    vi.mocked(setup.api.snapshot)
      .mockRejectedValueOnce(new Error("ipc unavailable"))
      .mockResolvedValueOnce(snapshot());
    render(<InstallerApp api={setup.api} />);

    expect(await screen.findByRole("heading", { name: /installation interrompue/i })).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));

    expect(await screen.findByRole("button", { name: /^installer$/i })).toBeEnabled();
  });

  it("affiche un échec si le démarrage IPC échoue sans événement", async () => {
    const setup = fixture();
    vi.mocked(setup.api.start).mockRejectedValue(new Error("start failed"));
    render(<InstallerApp api={setup.api} />);

    fireEvent.click(await screen.findByRole("button", { name: /^installer$/i }));

    expect(await screen.findByText(/installation a échoué/i)).toBeVisible();
  });

  it("affiche le dossier en lecture seule et permet de le parcourir", async () => {
    const { api } = fixture();
    vi.mocked(api.chooseDirectory).mockResolvedValue(snapshot({
      destination: "/Users/kevin/Apps",
      installedVersion: "1.4.1",
      beaverRunning: true,
    }));
    render(<InstallerApp api={api} />);
    const destination = await screen.findByRole("textbox", { name: /dossier d[’']installation/i });
    expect(destination).toHaveAttribute("aria-readonly", "true");
    expect(destination).toHaveTextContent("/Applications");
    fireEvent.click(screen.getByRole("button", { name: /parcourir/i }));
    await waitFor(() => expect(destination).toHaveTextContent("/Users/kevin/Apps"));
    expect(screen.getByRole("button", { name: /réinstaller/i })).toBeDisabled();
    expect(screen.getByText(/ferme Beaver avant de continuer/i)).toBeVisible();
  });

  it("propose Installer ou Réinstaller et bloque si Beaver tourne", async () => {
    const first = fixture();
    const view = render(<InstallerApp api={first.api} />);
    expect(await screen.findByRole("button", { name: /^installer$/i })).toBeEnabled();

    const reinstall = fixture(snapshot({ installedVersion: "1.4.1" }));
    view.rerender(<InstallerApp api={reinstall.api} />);
    expect(await screen.findByRole("button", { name: /réinstaller/i })).toBeEnabled();

    view.unmount();
    const running = fixture(snapshot({ beaverRunning: true }));
    render(<InstallerApp api={running.api} />);
    expect(await screen.findByRole("button", { name: /^installer$/i })).toBeDisabled();
    expect(screen.getByText(/ferme Beaver avant de continuer/i)).toBeVisible();
  });

  it("rend les cinq étapes, les deux progressions et masque Annuler au non-retour", async () => {
    const current = snapshot({
      phase: "downloading",
      stepIndex: 2,
      progressMode: "determinate",
      percent: 61,
      canCancel: true,
    });
    const { api } = fixture(current);
    const view = render(<InstallerApp api={api} />);
    expect(await screen.findAllByRole("listitem")).toHaveLength(5);
    expect(
      [...document.querySelectorAll(".binst-step-name")].map((item) => item.textContent),
    ).toEqual([
      "Vérification du système",
      "Téléchargement de Beaver",
      "Vérification d’intégrité",
      "Installation dans le dossier choisi",
      "Finitions",
    ]);
    expect(screen.getByText("61 %")).toBeVisible();
    expect(screen.getByRole("button", { name: /annuler/i })).toBeVisible();

    view.unmount();
    const nonReturn = fixture(snapshot({ phase: "installing", stepIndex: 4 }));
    render(<InstallerApp api={nonReturn.api} />);
    await screen.findByText(/étape 4 sur 5/i);
    expect(screen.getByRole("progressbar", { name: /installation dans/i })).not.toHaveAttribute(
      "aria-valuenow",
    );
    expect(screen.queryByRole("button", { name: /annuler/i })).not.toBeInTheDocument();
  });

  it("borne le journal à 64 entrées et ignore une séquence ancienne", async () => {
    const setup = fixture(snapshot());
    render(<InstallerApp api={setup.api} />);
    fireEvent.click(await screen.findByRole("button", { name: /^installer$/i }));
    await waitFor(() => expect(setup.api.start).toHaveBeenCalled());
    for (let sequence = 1; sequence <= 70; sequence += 1) {
      setup.publish({
        sequence,
        snapshot: snapshot({ phase: "downloading", stepIndex: 2, canCancel: true }),
        completedStepDurationsMs: [],
        logKey: "installer.log.downloading",
      });
    }
    setup.publish({
      sequence: 4,
      snapshot: snapshot({ phase: "checking", stepIndex: 1 }),
      completedStepDurationsMs: [],
      logKey: "installer.log.checking",
    });
    fireEvent.click(screen.getByRole("button", { name: /afficher les détails/i }));
    expect(screen.getAllByTestId("installer-log-entry")).toHaveLength(64);
    expect(screen.getByText(/étape 2 sur 5/i)).toBeVisible();
  });

  it("applique l'annulation avec la même séquence autoritaire que les événements", async () => {
    const current = snapshot({ phase: "downloading", stepIndex: 2, canCancel: true });
    const setup = fixture(current);
    vi.mocked(setup.api.cancel).mockResolvedValue({
      sequence: 8,
      snapshot: snapshot({ phase: "cancelling", stepIndex: 2 }),
      completedStepDurationsMs: [10],
      logKey: null,
    });
    render(<InstallerApp api={setup.api} />);

    fireEvent.click(await screen.findByRole("button", { name: /annuler/i }));

    await waitFor(() => expect(document.querySelector(".binst-window"))
      .toHaveAttribute("data-screen", "cancelling"));
  });

  it("affiche les fins annulée, échouée et réussie avec leurs actions", async () => {
    const cancelled = fixture(snapshot({ phase: "cancelled", stepIndex: 2 }));
    const view = render(<InstallerApp api={cancelled.api} />);
    expect(await screen.findByRole("heading", { name: "Installation annulée" })).toBeVisible();
    expect(screen.getByText(/dossier d[’']installation est intact/i)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));
    expect(cancelled.api.start).toHaveBeenCalledOnce();

    const failed = fixture(
      snapshot({ phase: "failed", stepIndex: 2, errorKey: "installer-download-failed" }),
    );
    view.rerender(<InstallerApp api={failed.api} />);
    expect(await screen.findByText(/téléchargement a échoué/i)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: /réessayer/i }));
    expect(failed.api.start).toHaveBeenCalledOnce();

    view.unmount();
    const complete = fixture(snapshot({ phase: "completed", outcome: "installed" }));
    render(<InstallerApp api={complete.api} />);
    fireEvent.click(await screen.findByRole("button", { name: /lancer Beaver/i }));
    expect(complete.api.launch).toHaveBeenCalledOnce();
  });
});
