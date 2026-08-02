import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ProjectSelector } from "../project-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ProjectSelector directory access", () => {
  it("affiche encore le blocage pour une session dont le projet est verrouillé", () => {
    render(
      <ProjectSelector
        projects={[{
          id: "project-1",
          name: "Project",
          path: "/project",
          order: 0,
          created_at: "2026-08-02T00:00:00Z",
        }]}
        selectedProjectId="project-1"
        locked
        hidden={false}
        onSelect={vi.fn()}
        onAddProject={vi.fn()}
        directoryAccessPrompt={{
          allowedPaths: ["/project/allowed"],
          onCancel: vi.fn(),
          onSettings: vi.fn(),
        }}
      />,
    );

    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("/project/allowed")).toBeInTheDocument();
  });
});
