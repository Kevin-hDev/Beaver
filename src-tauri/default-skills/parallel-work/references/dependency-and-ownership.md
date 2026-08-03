# Dependency and Ownership

You use this guide to decide whether two tasks may run at the same time.

## Dependency test

You add an ordering edge from `A` to `B` when `B` needs any output, decision, schema, generated artifact, migration, or verified behavior from `A`. You also add an edge when running `B` first could make `A` unsafe or invalidate its evidence.

You do not confuse a shared read dependency with a write conflict. Two tasks may read the same stable file concurrently when neither can rewrite it or a derived shared artifact.

## Ownership test

You assign each task an exclusive set of possible writes, not only the files you expect it to change. You include formatter reach, lockfiles, snapshots, generated indexes, migration registries, caches committed to the project, and configuration rewritten by tools.

You serialize tasks when either task may:

- You rewrite the same file or directory.
- You update a shared manifest, lockfile, schema, migration order, generated index, or snapshot.
- You use a mutually exclusive port, device, account, environment, or other stateful resource.
- You run a broad formatter or generator across the other task's scope.
- You depend on a product or architecture decision the other task establishes.

## Graph validation

You give every task a stable ID. You keep edges directional and record one concrete reason per edge. You detect cycles before launch and merge tasks only when their work is inherently inseparable. Otherwise, you expose the cycle as blocked and preserve the unaffected graph.

## Drift handling

You recalculate ownership and readiness after every wave. You treat any executor request for an unowned path as new scheduling information. You pause that task, update the graph, and either expand exclusive ownership safely or serialize it behind the conflicting work.
