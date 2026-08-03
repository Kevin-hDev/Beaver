# Playbook locations

You use two classes of playbook home: one project-owned home and this skill's packaged examples.

## Project home

You choose a project home only from:

1. A safe project-local destination explicitly named by the user.
2. One unambiguous established convention proved by existing playbook files, project instructions, or a documentation index.

You do not infer a new home merely because `docs/`, `.project/`, or `playbooks/` would be a common name. When no convention exists, or multiple homes compete, ask the user where project playbooks belong before writing. Canonicalize the project root and candidate parent, reject `..`, symlink escapes, unresolved parents, and paths outside the project.

## Packaged examples

Packaged examples live in `assets/examples/` inside this skill. You may list, read, research, or copy one into an explicitly resolved project home. You never update or overwrite packaged examples during normal use.

## External inputs

Treat an explicitly selected external file, repository, or URL as research evidence, not as a project playbook home. Verify its identity, currency, provenance, license-sensitive reuse boundary, and claims before using it. Copy or adapt content only after the user selects a safe project destination and confirms the intended write; never persist an external or personal collection silently.

## Resolution

Resolve a playbook in this order:

1. A display number from the latest list table in the current conversation.
2. An exact project slug.
3. An exact packaged-example slug.
4. An exact title across both homes.
5. A topic match across both homes.

Show candidates when more than one title or topic match is plausible. A number is valid only for the latest displayed list and is never stored. If no current numbered list exists, rerun `list` and ask the user to select again.

A project playbook shadows a same-slug packaged example. List both, mark the project copy `active`, and mark the packaged copy `shadowed`.

## Write ownership

Write only within the resolved project home. For an existing project file, show material changes and confirm overwrite intent. For a packaged-only match, ask whether to copy and adapt it into the project home. Never write to personal, global, external, or packaged locations unless a separate explicit request establishes that exact destination and ownership; otherwise stop.
