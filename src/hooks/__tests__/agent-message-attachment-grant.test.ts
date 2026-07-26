import { describe, expect, it } from "vitest";
import { pendingFilesToAttachments } from "../agent-message-builders";

describe("pendingFilesToAttachments", () => {
  it("conserve la preuve d'accès persistante", () => {
    const attachments = pendingFilesToAttachments([{
      name: "notes.txt",
      path: "/tmp/notes.txt",
      type: "text/plain",
      size: 12,
      accessGrant: "v1.valid",
    }]);

    expect(attachments[0]).toMatchObject({
      mime_type: "text/plain",
      size: 12,
      access_grant: "v1.valid",
    });
  });
});
