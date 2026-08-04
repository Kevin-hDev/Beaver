import { describe, expect, it } from "vitest";

import { updateErrorKey } from "../update-error";

describe("updateErrorKey", () => {
  it("distingue les étapes connues de la mise à jour", () => {
    expect(updateErrorKey("update-download-error")).toBe("errors.downloadFailed");
    expect(updateErrorKey("update-write-error")).toBe("errors.updatePrepareFailed");
    expect(updateErrorKey("update-install-error")).toBe("errors.updateInstallFailed");
  });

  it("masque les erreurs internes inconnues", () => {
    expect(updateErrorKey("C:\\Users\\name\\secret")).toBe("errors.updateFailed");
    expect(updateErrorKey({ path: "/private/secret" })).toBe("errors.updateFailed");
  });
});
