import { describe, expect, it } from "vitest";
import { appScopedPath } from "./forecast-notes-utils";

describe("appScopedPath", () => {
  it("shows the Beaver name for the compatible storage directory", () => {
    expect(appScopedPath("/Users/test/.local/share/cl-go-dash/forecast-notes/note.md"))
      .toBe("/Beaver/forecast-notes/note.md");
  });

  it("supports Windows paths without exposing the legacy display name", () => {
    expect(appScopedPath("C:\\Users\\test\\.local\\share\\cl-go-dash\\notes\\one.md"))
      .toBe("/Beaver/notes/one.md");
  });

  it("keeps paths outside the application storage unchanged", () => {
    expect(appScopedPath("/Users/test/Documents/forecast.csv"))
      .toBe("/Users/test/Documents/forecast.csv");
  });
});
