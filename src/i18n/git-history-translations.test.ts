import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itJson from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("git history translations", () => {
  it("contient les libellés dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as Array<{
      filePreview: {
        listModes: { uncommitted: string };
        diffUnavailable: string;
        diffTruncated: string;
        gitStatus: Record<string, string>;
      };
      agentLocal: {
        sessionSummary: {
          modificationsUnavailable: string;
          modificationsPartial: string;
          commits: { title: string; error: string };
          git: { remoteStatusUnavailable: string };
        };
      };
    }>;
    for (const locale of locales) {
      expect(locale.filePreview.listModes.uncommitted).toBeTruthy();
      expect(locale.filePreview.diffUnavailable).toBeTruthy();
      expect(locale.filePreview.diffTruncated).toBeTruthy();
      expect(Object.keys(locale.filePreview.gitStatus)).toHaveLength(6);
      expect(locale.filePreview.gitStatus.added).toBeTruthy();
      expect(locale.filePreview.gitStatus.modified).toBeTruthy();
      expect(locale.filePreview.gitStatus.deleted).toBeTruthy();
      expect(locale.filePreview.gitStatus.renamed).toBeTruthy();
      expect(locale.agentLocal.sessionSummary.commits.title).toBeTruthy();
      expect(locale.agentLocal.sessionSummary.commits.error).toBeTruthy();
      expect(locale.agentLocal.sessionSummary.modificationsUnavailable).toBeTruthy();
      expect(locale.agentLocal.sessionSummary.modificationsPartial).toContain("{{count}}");
      expect(locale.agentLocal.sessionSummary.git.remoteStatusUnavailable).toBeTruthy();
    }
    expect(fr.filePreview.listModes.uncommitted).toBe("Non commit");
  });
});
