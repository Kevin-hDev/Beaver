import { compareOccupants } from "./slot-registry";
import { compareMutations, mutationId } from "./slot-resolution-validation";
import type { SlotMutation, SlotOccupant } from "./slot-types";

export interface RelativePlacement {
  occupantId: string;
  operation: "before" | "after";
  targetId: string;
}

export function orderMutationsByDependencies(
  mutations: readonly SlotMutation[],
  declaredCandidateIds: ReadonlySet<string>,
): { ordered: SlotMutation[]; cyclicIds: string[] } {
  const byId = new Map(mutations.map((mutation) => [mutationId(mutation), mutation]));
  const outgoing = new Map<string, Set<string>>();
  const indegree = new Map([...byId.keys()].map((id) => [id, 0]));

  for (const mutation of mutations) {
    const dependentId = mutationId(mutation);
    const declarationId = mutation.targetId;
    if (!declarationId || !declaredCandidateIds.has(declarationId)
      || !byId.has(declarationId)) continue;
    const dependents = outgoing.get(declarationId) ?? new Set<string>();
    if (!dependents.has(dependentId)) {
      dependents.add(dependentId);
      outgoing.set(declarationId, dependents);
      indegree.set(dependentId, (indegree.get(dependentId) ?? 0) + 1);
    }
  }

  const ready = mutations.filter((mutation) => indegree.get(mutationId(mutation)) === 0)
    .sort(compareMutations);
  const ordered: SlotMutation[] = [];
  while (ready.length > 0) {
    const mutation = ready.shift();
    if (!mutation) break;
    ordered.push(mutation);
    for (const dependentId of outgoing.get(mutationId(mutation)) ?? []) {
      const next = (indegree.get(dependentId) ?? 0) - 1;
      indegree.set(dependentId, next);
      if (next === 0) {
        const dependent = byId.get(dependentId);
        if (dependent) {
          ready.push(dependent);
          ready.sort(compareMutations);
        }
      }
    }
  }

  const unresolved = mutations.map(mutationId)
    .filter((id) => (indegree.get(id) ?? 0) > 0);
  return {
    ordered,
    cyclicIds: unresolved.filter((id) => reachesItself(id, id, outgoing, new Set())).sort(),
  };
}

function reachesItself(
  start: string,
  current: string,
  outgoing: ReadonlyMap<string, ReadonlySet<string>>,
  visited: Set<string>,
): boolean {
  for (const next of outgoing.get(current) ?? []) {
    if (next === start) return true;
    if (!visited.has(next)) {
      visited.add(next);
      if (reachesItself(start, next, outgoing, visited)) return true;
    }
  }
  return false;
}

export function orderOccupants(
  occupants: readonly SlotOccupant[],
  relative: readonly RelativePlacement[],
): { ordered: SlotOccupant[]; cyclicIds: string[] } {
  const byId = new Map(occupants.map((occupant) => [occupant.id, occupant]));
  const outgoing = new Map<string, Set<string>>();
  const indegree = new Map(occupants.map((occupant) => [occupant.id, 0]));

  for (const relation of relative) {
    if (!byId.has(relation.occupantId) || !byId.has(relation.targetId)) continue;
    const from = relation.operation === "before" ? relation.occupantId : relation.targetId;
    const to = relation.operation === "before" ? relation.targetId : relation.occupantId;
    const targets = outgoing.get(from) ?? new Set<string>();
    if (!targets.has(to)) {
      targets.add(to);
      outgoing.set(from, targets);
      indegree.set(to, (indegree.get(to) ?? 0) + 1);
    }
  }

  const ready = occupants.filter((occupant) => indegree.get(occupant.id) === 0).sort(compareOccupants);
  const ordered: SlotOccupant[] = [];
  while (ready.length > 0) {
    const occupant = ready.shift();
    if (!occupant) break;
    ordered.push(occupant);
    for (const targetId of outgoing.get(occupant.id) ?? []) {
      const next = (indegree.get(targetId) ?? 0) - 1;
      indegree.set(targetId, next);
      if (next === 0) {
        const target = byId.get(targetId);
        if (target) {
          ready.push(target);
          ready.sort(compareOccupants);
        }
      }
    }
  }

  const emitted = new Set(ordered.map((occupant) => occupant.id));
  const cyclicIds = occupants
    .filter((occupant) => !emitted.has(occupant.id))
    .map((occupant) => occupant.id)
    .sort();
  return { ordered, cyclicIds };
}
