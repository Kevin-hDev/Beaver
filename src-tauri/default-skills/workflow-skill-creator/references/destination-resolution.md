# Destination Resolution

You resolve where a skill belongs without moving or copying it unnecessarily.

- You use the destination explicitly supplied by the user when it is safe and writable.
- You inspect project documentation and neighboring skill bundles only to discover a binding local convention.
- You ask the user to choose an exact destination root when evidence is absent, conflicting, or describes several valid locations.
- You keep an existing skill in its current directory during a refactor unless the user explicitly requests a move.
- You never derive a destination from the source directory of a loaded or imported skill.
- You reject `..`, unresolved symbolic links, non-directory roots, and any resolved target outside the confirmed root.
- You preserve a skill in another CLI's directory when that directory is the confirmed owner; you do not migrate it into a central copy.
- You state the final absolute or project-relative target before writing and require confirmation for a new bundle or move.
