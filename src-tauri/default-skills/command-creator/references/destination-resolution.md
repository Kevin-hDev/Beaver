# Destination Resolution

You place command bundles only in confirmed skill roots.

- You use an explicit user-supplied destination when it is safe and writable.
- You inspect project documentation and neighboring bundles only for a binding local convention.
- You ask the user to choose a destination when evidence is absent, conflicting, or offers several valid roots.
- You keep an existing bundle in place during refactoring unless the user explicitly requests a move.
- You never derive the destination from an imported skill, another CLI's command directory, or a familiar default path.
- You reject `..`, unresolved symbolic links, non-directory roots, and targets that resolve outside the confirmed root.
- You confirm the final target before writing and preserve source-owned skills in their current directories.
