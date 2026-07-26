import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MissingDirectoryPrompt } from "../missing-directory-prompt";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: { path?: string }) => values?.path ?? key,
  }),
}));

describe("MissingDirectoryPrompt", () => {
  it("affiche les chemins et transmet les deux actions", () => {
    const onResolve = vi.fn();
    render(
      <MissingDirectoryPrompt
        directory={{ missing_path: "/project/gone", nearest_parent: "/project" }}
        resolving={false}
        onResolve={onResolve}
      />,
    );

    expect(screen.getByText("/project/gone")).toBeTruthy();
    expect(screen.getByText("/project")).toBeTruthy();
    fireEvent.click(screen.getByText("missingDirectory.switch"));
    fireEvent.click(screen.getByText("missingDirectory.create"));
    expect(onResolve).toHaveBeenNthCalledWith(1, "switch");
    expect(onResolve).toHaveBeenNthCalledWith(2, "create");
  });
});
