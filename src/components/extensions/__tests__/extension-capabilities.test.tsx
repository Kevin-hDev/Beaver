import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionCapabilities } from "../extension-capabilities";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

function extension(status: ExtensionRecord["status"] = "active"): ExtensionRecord {
  return {
    manifest: {
      id: "com.example.capabilities",
      name: "Capabilities",
      version: "1.0.0",
      beaverApi: "1",
      runtime: "node",
      access: "full",
      apiLevel: "stable",
      essential: false,
    },
    kind: "local",
    source: "/extension",
    enabled: true,
    trusted: true,
    showInChat: true,
    status,
    contributions: {
      tools: [],
      events: [],
      skills: [{
        id: "guide",
        name: "Guide",
        description: "A concise guide.",
        path: "skills/private-guide.md",
      }],
      resources: [{
        id: "preview",
        name: "Preview",
        description: "An image preview.",
        type: "image",
        path: "resources/private-preview.png",
      }],
    },
  };
}

describe("ExtensionCapabilities", () => {
  it("affiche des lignes compactes sans révéler les chemins internes", () => {
    render(<ExtensionCapabilities extension={extension()} />);

    expect(screen.getByText("Guide")).toBeInTheDocument();
    expect(screen.getByText("A concise guide.")).toBeInTheDocument();
    expect(screen.getByText("extensions.detail.resourceTypes.image")).toBeInTheDocument();
    expect(screen.queryByText("skills/private-guide.md")).not.toBeInTheDocument();
    expect(screen.queryByText("resources/private-preview.png")).not.toBeInTheDocument();
  });

  it("associe chaque instance à un titre distinct", () => {
    const { container } = render(<>
      <ExtensionCapabilities extension={extension()} />
      <ExtensionCapabilities extension={extension()} />
    </>);
    const sections = [...container.querySelectorAll("section[aria-labelledby]")];
    const titleIds = sections.map((section) => section.getAttribute("aria-labelledby"));

    expect(new Set(titleIds).size).toBe(2);
    for (const titleId of titleIds) {
      expect(container.querySelector(`#${titleId}`)).not.toBeNull();
    }
  });

  it.each([
    ["loading", "extensions.detail.capabilitiesLoading"],
    ["error", "extensions.detail.capabilitiesError"],
    ["inactive", "extensions.detail.capabilitiesEmpty"],
  ] as const)("rend l'état %s", (status, expected) => {
    const value = extension(status);
    value.contributions.skills = [];
    value.contributions.resources = [];

    render(<ExtensionCapabilities extension={value} />);

    expect(screen.getByText(expected)).toBeInTheDocument();
  });
});
