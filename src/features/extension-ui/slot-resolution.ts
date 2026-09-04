import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import {
  orderMutationsByDependencies,
  type RelativePlacement,
} from "./slot-resolution-order";
import {
  applyMutation,
  cloneCore,
  occupantIndex,
  orderAndBound,
  materializeCandidates,
  rejectBrokenRelations,
  removeRejected,
} from "./slot-resolution-state";
import {
  extensionOccupantId,
  type SlotDiagnostic,
  type SlotMutation,
  type SlotOccupant,
  type SlotRegistry,
  type SlotResolution,
} from "./slot-types";
import {
  compareDiagnostics,
  compareMutations,
  mutationId,
  needsNewOccupant,
  rejectPreflightConflicts,
  rejectUnavailableDependencies,
  targetDiagnostic,
  validOrder,
} from "./slot-resolution-validation";

export function resolveSlots(
  registry: SlotRegistry,
  inputMutations: readonly SlotMutation[],
): SlotResolution {
  if (inputMutations.length > UI_LIMITS.maxGlobalStandardContributions) {
    throw new Error("Slot mutation limit exceeded.");
  }
  const mutations = [...inputMutations].sort(compareMutations);
  const working = cloneCore(registry);
  const baseById = occupantIndex(working);
  const rejected = new Set<string>();
  const diagnostics = new Map<string, SlotDiagnostic[]>();
  const relations: RelativePlacement[] = [];
  const candidates = new Map<string, SlotOccupant>();

  const reject = (mutation: SlotMutation, code: SlotDiagnostic["code"]) => {
    const id = mutationId(mutation);
    if (rejected.has(id)) return;
    rejected.add(id);
    const current = diagnostics.get(mutation.extensionId) ?? [];
    current.push({
      extensionId: mutation.extensionId,
      contributionId: mutation.contributionId,
      code,
    });
    diagnostics.set(mutation.extensionId, current);
  };

  for (const mutation of mutations) {
    const definition = registry.definitions[mutation.placement];
    const id = extensionOccupantId(mutation.extensionId, mutation.contributionId);
    if (!id || !definition
      || definition.contributionType !== mutation.contributionType
      || !validOrder(mutation.order)) {
      reject(mutation, "ui_contribution_invalid");
      continue;
    }
    if (baseById.has(id)) {
      reject(mutation, "ui_mutation_conflict");
      continue;
    }
  }

  const structurallyValid = mutations.filter(
    (mutation) => !rejected.has(mutationId(mutation)),
  );
  for (const mutation of structurallyValid) {
    const id = mutationId(mutation);
    if (needsNewOccupant(mutation.operation)) {
      candidates.set(id, {
        id,
        placement: mutation.placement,
        contributionType: mutation.contributionType,
        order: mutation.order,
        source: {
          kind: "extension",
          extensionId: mutation.extensionId,
          contributionId: mutation.contributionId,
        },
        target: id,
      });
    }
  }

  const references = new Map([...baseById, ...candidates]);
  for (const mutation of structurallyValid) {
    const diagnostic = targetDiagnostic(mutation, references);
    if (diagnostic) reject(mutation, diagnostic);
  }

  const declaredCandidateIds = new Set(candidates.keys());
  rejectUnavailableDependencies(mutations, declaredCandidateIds, rejected, reject);
  rejectPreflightConflicts(
    mutations.filter((mutation) => !rejected.has(mutationId(mutation))),
    reject,
  );
  rejectUnavailableDependencies(mutations, declaredCandidateIds, rejected, reject);

  let executable = mutations.filter((mutation) => !rejected.has(mutationId(mutation)));
  let execution = orderMutationsByDependencies(executable, declaredCandidateIds);
  for (const cyclicId of execution.cyclicIds) {
    const mutation = mutations.find((item) => mutationId(item) === cyclicId);
    if (mutation) reject(mutation, "ui_mutation_conflict");
  }
  rejectUnavailableDependencies(mutations, declaredCandidateIds, rejected, reject);
  executable = mutations.filter((mutation) => !rejected.has(mutationId(mutation)));
  execution = orderMutationsByDependencies(executable, declaredCandidateIds);

  /* Les candidats survivants sont présents avant exécution, puis les effets
     suivent leurs dépendances réelles plutôt que leurs identifiants. */
  materializeCandidates(
    working,
    [...candidates.values()].filter(({ id }) => !rejected.has(id)),
  );
  for (const mutation of execution.ordered) {
    const id = mutationId(mutation);
    applyMutation(mutation, candidates.get(id), working, relations, reject);
  }

  rejectBrokenRelations(relations, working, mutations, rejected, reject);
  removeRejected(working, rejected);
  const occupantsByPlacement = orderAndBound(
    working,
    relations,
    mutations,
    rejected,
    reject,
  );
  const diagnosticsByExtension = Object.fromEntries(
    [...diagnostics.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([extensionId, values]) => [extensionId, values.sort(compareDiagnostics)]),
  );
  return {
    occupantsByPlacement,
    diagnosticsByExtension,
    rejectedContributionIds: [...rejected].sort(),
  };
}
