import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { GitDirtyFileList } from "../git-dirty-file-list";

describe("GitDirtyFileList", () => {
  it("n'écrit jamais +0 ni -0 à côté d'un fichier", () => {
    const { container } = render(
      <GitDirtyFileList
        files={[
          { path: "src/added.ts", status: "added", additions: 4, deletions: 0 },
          { path: "src/removed.ts", status: "deleted", additions: 0, deletions: 2 },
          { path: "src/changed.ts", status: "modified", additions: 1, deletions: 1 },
        ]}
      />,
    );

    const stats = [...container.querySelectorAll(".bcd-file-stat")].map((stat) => stat.textContent);
    expect(stats).toEqual(["+4", "-2", "+1 -1"]);
  });
});
