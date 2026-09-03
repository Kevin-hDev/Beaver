import { describe, expect, it } from "vitest";
import { UI_LIMITS, UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { createSlotRegistry } from "../slot-registry";
import { resolveSlots } from "../slot-resolution";
import type { SlotMutation, SlotOccupant } from "../slot-types";

const BASE_OCCUPANTS: readonly SlotOccupant[] = [
  core("beaver.agent-local", "app.navigation.primary", "tab", 0),
  core("beaver.heartbeat", "app.navigation.primary", "tab", 10),
  core("beaver.settings", "app.navigation.primary", "tab", 30),
  core("beaver.general", "settings.navigation.preferences", "settingsTab", 0),
  core("beaver.extensions", "settings.navigation.integrations", "settingsTab", 30),
];

function core(
  id: string,
  placement: SlotOccupant["placement"],
  contributionType: SlotOccupant["contributionType"],
  order: number,
): SlotOccupant {
  return { id, placement, contributionType, order, source: { kind: "core" }, target: id };
}

function mutation(
  extensionId: string,
  contributionId: string,
  operation: SlotMutation["operation"],
  options: Partial<Pick<SlotMutation, "placement" | "contributionType" | "order" | "targetId">> = {},
): SlotMutation {
  return {
    extensionId,
    contributionId,
    operation,
    placement: options.placement ?? "app.navigation.primary",
    contributionType: options.contributionType ?? "tab",
    order: options.order ?? 10,
    ...(options.targetId ? { targetId: options.targetId } : {}),
  };
}

function ids(result: ReturnType<typeof resolveSlots>, placement = "app.navigation.primary") {
  return result.occupantsByPlacement[placement as keyof typeof result.occupantsByPlacement]
    .map((occupant) => occupant.id);
}

function rejected(result: ReturnType<typeof resolveSlots>) {
  return result.rejectedContributionIds;
}

function codes(result: ReturnType<typeof resolveSlots>, extensionId: string) {
  return (result.diagnosticsByExtension[extensionId] ?? []).map((diagnostic) => diagnostic.code);
}

describe("slot resolution", () => {
  const registry = createSlotRegistry(UI_PLACEMENTS, BASE_OCCUPANTS);

  it("orders additions independently from arrival order", () => {
    const first = mutation("zeta", "late", "add", { order: 20 });
    const second = mutation("alpha", "early", "add", { order: 20 });

    const forward = resolveSlots(registry, [first, second]);
    const reverse = resolveSlots(registry, [second, first]);

    expect(ids(forward)).toEqual([
      "beaver.agent-local",
      "beaver.heartbeat",
      "extension:alpha:early",
      "extension:zeta:late",
      "beaver.settings",
    ]);
    expect(ids(reverse)).toEqual(ids(forward));
  });

  it("places before and after the referenced compatible occupant", () => {
    const result = resolveSlots(registry, [
      mutation("acme", "before", "before", { targetId: "beaver.heartbeat" }),
      mutation("acme", "after", "after", { targetId: "beaver.heartbeat" }),
    ]);

    expect(ids(result)).toEqual([
      "beaver.agent-local",
      "extension:acme:before",
      "beaver.heartbeat",
      "extension:acme:after",
      "beaver.settings",
    ]);
  });

  it("replaces, moves and removes only the selected occupants", () => {
    const result = resolveSlots(registry, [
      mutation("acme", "replacement", "replace", { targetId: "beaver.heartbeat" }),
      mutation("acme", "move-general", "move", {
        targetId: "beaver.general",
        placement: "settings.navigation.integrations",
        contributionType: "settingsTab",
      }),
      mutation("acme", "remove-agent", "remove", { targetId: "beaver.agent-local" }),
    ]);

    expect(ids(result)).toEqual([
      "extension:acme:replacement",
      "beaver.settings",
    ]);
    expect(ids(result, "settings.navigation.preferences")).toEqual([]);
    expect(ids(result, "settings.navigation.integrations")).toEqual([
      "beaver.general",
      "beaver.extensions",
    ]);
  });

  it("rejects a missing target without changing healthy occupants", () => {
    const result = resolveSlots(registry, [
      mutation("acme", "missing", "before", { targetId: "beaver.absent" }),
      mutation("healthy", "tab", "add", { order: 5 }),
    ]);

    expect(rejected(result)).toEqual(["extension:acme:missing"]);
    expect(codes(result, "acme")).toEqual(["ui_reference_missing"]);
    expect(ids(result)).toContain("extension:healthy:tab");
  });

  it("rejects a target from an incompatible placement type", () => {
    const result = resolveSlots(registry, [
      mutation("acme", "wrong-type", "before", { targetId: "beaver.general" }),
    ]);

    expect(rejected(result)).toEqual(["extension:acme:wrong-type"]);
    expect(codes(result, "acme")).toEqual(["ui_reference_incompatible"]);
    expect(ids(result)).toEqual([
      "beaver.agent-local", "beaver.heartbeat", "beaver.settings",
    ]);
  });

  it("rejects every side of a relative cycle", () => {
    const result = resolveSlots(registry, [
      mutation("one", "first", "before", { targetId: "extension:two:second" }),
      mutation("two", "second", "before", { targetId: "extension:one:first" }),
    ]);

    expect(rejected(result)).toEqual([
      "extension:one:first", "extension:two:second",
    ]);
    expect(codes(result, "one")).toEqual(["ui_mutation_conflict"]);
    expect(codes(result, "two")).toEqual(["ui_mutation_conflict"]);
    expect(ids(result)).toEqual([
      "beaver.agent-local", "beaver.heartbeat", "beaver.settings",
    ]);
  });

  it("keeps healthy relative ordering when another relation cycles", () => {
    const result = resolveSlots(registry, [
      mutation("healthy", "before-heartbeat", "before", {
        targetId: "beaver.heartbeat",
        order: 100,
      }),
      mutation("one", "first", "before", { targetId: "extension:two:second" }),
      mutation("two", "second", "before", { targetId: "extension:one:first" }),
    ]);

    const ordered = ids(result);
    expect(ordered.indexOf("extension:healthy:before-heartbeat"))
      .toBeLessThan(ordered.indexOf("beaver.heartbeat"));
    expect(ordered).not.toContain("extension:one:first");
    expect(ordered).not.toContain("extension:two:second");
  });

  it("rejects a relative contribution when its target moves to another placement", () => {
    const relative = mutation("alpha", "before-general", "before", {
      placement: "settings.navigation.preferences",
      contributionType: "settingsTab",
      targetId: "beaver.general",
    });
    const move = mutation("zeta", "move-general", "move", {
      placement: "settings.navigation.integrations",
      contributionType: "settingsTab",
      targetId: "beaver.general",
    });

    for (const input of [[relative, move], [move, relative]]) {
      const result = resolveSlots(registry, input);
      expect(codes(result, "alpha")).toEqual(["ui_reference_incompatible"]);
      expect(codes(result, "zeta")).toEqual([]);
      expect(ids(result, "settings.navigation.preferences"))
        .not.toContain("extension:alpha:before-general");
      expect(ids(result, "settings.navigation.integrations")).toEqual([
        "beaver.general",
        "beaver.extensions",
      ]);
    }
  });

  it("rejects all destructive conflicts and retains the original occupant", () => {
    const result = resolveSlots(registry, [
      mutation("one", "replace", "replace", { targetId: "beaver.heartbeat" }),
      mutation("two", "remove", "remove", { targetId: "beaver.heartbeat" }),
    ]);

    expect(rejected(result)).toEqual([
      "extension:one:replace", "extension:two:remove",
    ]);
    expect(codes(result, "one")).toEqual(["ui_mutation_conflict"]);
    expect(codes(result, "two")).toEqual(["ui_mutation_conflict"]);
    expect(ids(result)).toContain("beaver.heartbeat");
  });

  it("does not let an invalid destructive mutation conflict with a valid one", () => {
    const result = resolveSlots(registry, [
      mutation("broken", "wrong-type", "remove", {
        targetId: "beaver.heartbeat",
        contributionType: "settingsTab",
      }),
      mutation("healthy", "remove-heartbeat", "remove", {
        targetId: "beaver.heartbeat",
      }),
    ]);

    expect(codes(result, "broken")).toEqual(["ui_contribution_invalid"]);
    expect(codes(result, "healthy")).toEqual([]);
    expect(ids(result)).not.toContain("beaver.heartbeat");
  });

  it("does not let an incompatible destructive target conflict with a valid one", () => {
    const result = resolveSlots(registry, [
      mutation("broken", "wrong-target", "remove", {
        placement: "settings.navigation.preferences",
        contributionType: "settingsTab",
        targetId: "beaver.heartbeat",
      }),
      mutation("healthy", "remove-heartbeat", "remove", {
        targetId: "beaver.heartbeat",
      }),
    ]);

    expect(codes(result, "broken")).toEqual(["ui_reference_incompatible"]);
    expect(codes(result, "healthy")).toEqual([]);
    expect(ids(result)).not.toContain("beaver.heartbeat");
  });

  it("rejects a mutation whose same-batch target declaration is rejected", () => {
    const result = resolveSlots(registry, [
      mutation("broken", "invalid-anchor", "before", {
        targetId: "beaver.missing",
      }),
      mutation("dependent", "replacement", "replace", {
        targetId: "extension:broken:invalid-anchor",
      }),
    ]);

    expect(codes(result, "broken")).toEqual(["ui_reference_missing"]);
    expect(codes(result, "dependent")).toEqual(["ui_reference_missing"]);
    expect(ids(result)).not.toContain("extension:broken:invalid-anchor");
    expect(ids(result)).not.toContain("extension:dependent:replacement");
  });

  it.each([
    ["zeta", "alpha"],
    ["alpha", "zeta"],
  ])("removes a same-batch declaration regardless of lexical ids (%s, %s)",
    (targetExtensionId, removerExtensionId) => {
      const targetId = `extension:${targetExtensionId}:target`;
      const remove = mutation(removerExtensionId, "remove-target", "remove", { targetId });
      const declaration = mutation(targetExtensionId, "target", "add", { order: 5 });

      for (const input of [[remove, declaration], [declaration, remove]]) {
        const result = resolveSlots(registry, input);
        expect(rejected(result)).toEqual([]);
        expect(ids(result)).not.toContain(targetId);
      }
    });

  it.each([
    ["zeta", "alpha"],
    ["alpha", "zeta"],
  ])("resolves replace chains by dependency rather than lexical ids (%s, %s)",
    (middleExtensionId, finalExtensionId) => {
      const middleId = `extension:${middleExtensionId}:middle`;
      const finalId = `extension:${finalExtensionId}:final`;
      const middle = mutation(middleExtensionId, "middle", "replace", {
        targetId: "beaver.heartbeat",
        order: 100,
      });
      const final = mutation(finalExtensionId, "final", "replace", {
        targetId: middleId,
        order: -100,
      });

      for (const input of [[final, middle], [middle, final]]) {
        const result = resolveSlots(registry, input);
        const resolvedFinal = result.occupantsByPlacement["app.navigation.primary"]
          .find(({ id }) => id === finalId);
        expect(rejected(result)).toEqual([]);
        expect(ids(result)).not.toContain("beaver.heartbeat");
        expect(ids(result)).not.toContain(middleId);
        expect(resolvedFinal?.order).toBe(10);
      }
    });

  it.each([
    ["zeta", "alpha"],
    ["alpha", "zeta"],
  ])("replaces a same-batch declaration regardless of lexical ids (%s, %s)",
    (targetExtensionId, replacerExtensionId) => {
      const targetId = `extension:${targetExtensionId}:target`;
      const replacementId = `extension:${replacerExtensionId}:replacement`;
      const replace = mutation(replacerExtensionId, "replacement", "replace", { targetId });
      const declaration = mutation(targetExtensionId, "target", "add", { order: 5 });

      for (const input of [[replace, declaration], [declaration, replace]]) {
        const result = resolveSlots(registry, input);
        expect(rejected(result)).toEqual([]);
        expect(ids(result)).not.toContain(targetId);
        expect(ids(result).filter((id) => id === replacementId)).toHaveLength(1);
      }
    });

  it.each([
    ["zeta", "alpha"],
    ["alpha", "zeta"],
  ])("moves a same-batch declaration regardless of lexical ids (%s, %s)",
    (targetExtensionId, moverExtensionId) => {
      const targetId = `extension:${targetExtensionId}:target`;
      const move = mutation(moverExtensionId, "move-target", "move", { targetId, order: 25 });
      const declaration = mutation(targetExtensionId, "target", "add", { order: 5 });

      for (const input of [[move, declaration], [declaration, move]]) {
        const result = resolveSlots(registry, input);
        const moved = result.occupantsByPlacement["app.navigation.primary"]
          .find(({ id }) => id === targetId);
        expect(rejected(result)).toEqual([]);
        expect(moved?.order).toBe(25);
      }
    });

  it.each([
    ["zeta", "alpha"],
    ["alpha", "zeta"],
  ])("moves a same-batch replacement only after it is resolved (%s, %s)",
    (targetExtensionId, moverExtensionId) => {
      const targetId = `extension:${targetExtensionId}:replacement`;
      const declaration = mutation(targetExtensionId, "replacement", "replace", {
        targetId: "beaver.heartbeat",
        order: 5,
      });
      const move = mutation(moverExtensionId, "move-replacement", "move", {
        targetId,
        order: 25,
      });

      for (const input of [[move, declaration], [declaration, move]]) {
        const result = resolveSlots(registry, input);
        const moved = result.occupantsByPlacement["app.navigation.primary"]
          .find(({ id }) => id === targetId);
        expect(rejected(result)).toEqual([]);
        expect(ids(result)).not.toContain("beaver.heartbeat");
        expect(moved?.order).toBe(25);
      }
    });

  it("keeps protected Settings and Extensions for every forbidden operation", () => {
    const attempts = [
      mutation("one", "settings-replace", "replace", { targetId: "beaver.settings" }),
      mutation("two", "settings-remove", "remove", { targetId: "beaver.settings" }),
      mutation("three", "extensions-replace", "replace", {
        targetId: "beaver.extensions",
        placement: "settings.navigation.integrations",
        contributionType: "settingsTab",
      }),
      mutation("four", "extensions-remove", "remove", {
        targetId: "beaver.extensions",
        placement: "settings.navigation.integrations",
        contributionType: "settingsTab",
      }),
    ];

    for (const attempt of attempts) {
      const result = resolveSlots(registry, [attempt]);
      expect(rejected(result)).toEqual([`extension:${attempt.extensionId}:${attempt.contributionId}`]);
      expect(codes(result, attempt.extensionId)).toEqual(["ui_protected_occupant"]);
      expect(ids(result)).toContain("beaver.settings");
      expect(ids(result, "settings.navigation.integrations")).toContain("beaver.extensions");
    }
  });

  it("rejects additions beyond the generated per-placement bound", () => {
    const occupants = Array.from({ length: UI_LIMITS.maxOccupantsPerPlacement }, (_, index) =>
      core(`beaver.item-${index}`, "app.navigation.primary", "tab", index));
    const fullRegistry = createSlotRegistry(UI_PLACEMENTS, occupants);

    const result = resolveSlots(fullRegistry, [mutation("acme", "overflow", "add")]);

    expect(rejected(result)).toEqual(["extension:acme:overflow"]);
    expect(codes(result, "acme")).toEqual(["ui_limit_exceeded"]);
    expect(ids(result)).toHaveLength(UI_LIMITS.maxOccupantsPerPlacement);
  });
});
