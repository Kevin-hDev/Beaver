import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { CompressionProfileView } from "@/types/compression-profile.generated";
import { ChatPlusMenu } from "../chat-plus-menu";

vi.mock("@/hooks/use-connectors", () => ({
  useConnectors: () => ({ configured: [], toggleChatEnabled: vi.fn() }),
}));
vi.mock("@/hooks/use-extensions", () => ({
  useExtensions: () => ({ extensions: [], busyIds: new Set(), setEnabled: vi.fn() }),
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => ({
    "chatMenu.plusButtonHint": "Plus",
    "chatMenu.addFile": "Ajouter un fichier",
    "chatMenu.compression": "Compression",
  })[key] ?? key }),
}));

const profile = (id: string, name: string) => ({ id, name }) as CompressionProfileView;

describe("ChatPlusMenu compression", () => {
  it("reste présent en mode Chatbot et sélectionne Beaver puis custom", async () => {
    let finish: ((value: boolean) => void) | null = null;
    const select = vi.fn(() => new Promise<boolean>((resolve) => { finish = resolve; }));
    render(
      <ChatPlusMenu
        onFileImport={vi.fn()}
        agentic={false}
        planModeEnabled={false}
        onPlanModeChange={vi.fn()}
        showCompression
        compressionProfiles={[profile("beaver", "Beaver"), profile("custom", "Custom")]}
        selectedCompressionId="beaver"
        onCompressionSelect={select}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Plus" }));
    fireEvent.click(screen.getByRole("button", { name: "Compression" }));
    const choices = screen.getAllByRole("button").filter((button) => (
      button.textContent === "Beaver" || button.textContent === "Custom"
    ));
    expect(choices.map((button) => button.textContent)).toEqual(["Beaver", "Custom"]);
    fireEvent.click(screen.getByRole("button", { name: "Custom" }));
    expect(screen.getByText("Compression")).toBeInTheDocument();

    act(() => { finish?.(true); });
    expect(select).toHaveBeenCalledWith("custom");
    await waitFor(() => expect(screen.queryByText("Compression")).toBeNull());
  });
});
