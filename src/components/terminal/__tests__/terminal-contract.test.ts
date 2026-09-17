import { describe, expect, it } from "vitest";
import { MAX_WRITE_BYTES as queueWriteBytes } from "../terminal-input-queue";
import {
  MAX_GROUPS as uiGroups,
  MAX_LABEL_BYTES as uiLabelBytes,
  MAX_LIVE_TERMINALS as uiLiveTerminals,
  MAX_TABS_PER_GROUP as uiTabsPerGroup,
  MAX_TOTAL_TABS as uiTotalTabs,
} from "@/hooks/terminal-types";
import {
  MAX_GROUPS,
  MAX_LABEL_BYTES,
  MAX_LIVE_TERMINALS,
  MAX_TABS_PER_GROUP,
  MAX_TOTAL_TABS,
  MAX_WRITE_BYTES,
} from "@/types/terminal-contract.generated";

describe("terminal generated contract", () => {
  it("alimente les limites visibles de l'interface", () => {
    expect([uiLiveTerminals, uiGroups, uiTabsPerGroup, uiTotalTabs, uiLabelBytes]).toEqual([
      MAX_LIVE_TERMINALS,
      MAX_GROUPS,
      MAX_TABS_PER_GROUP,
      MAX_TOTAL_TABS,
      MAX_LABEL_BYTES,
    ]);
  });

  it("alimente la taille maximale d'une écriture PTY", () => {
    expect(queueWriteBytes).toBe(MAX_WRITE_BYTES);
  });
});
