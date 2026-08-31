# Boundaries checklist

## A. IPC / command surfaces

- [ ] Every handler callable from the less-trusted layer is enumerated —
      the list itself is part of the report
- [ ] Each handler validates its inputs at the boundary (not "somewhere
      later")
- [ ] Capability tokens (when used) are CSPRNG-generated, compared in
      constant time, scoped to one resource
- [ ] Permission scopes are minimal: flag any scope wider than the actual
      need (e.g. whole-disk read where a data dir suffices)
- [ ] Error paths fail closed and return generic messages

## B. Paths and files

- [ ] External paths are canonicalized and confined to an allowed root;
      `..` and symlink escapes rejected
- [ ] Writes are atomic where corruption matters
- [ ] Archive extraction: entry paths confined, sizes bounded, links rejected
- [ ] File dialogs/drag-drop: paths registered and validated before use

## C. Outbound requests (SSRF)

- [ ] Scheme allowlist (http/https only)
- [ ] Internal ranges blocked: loopback, RFC1918, link-local/metadata
      (169.254.0.0/16), and the user's own LAN
- [ ] Redirects re-validated at each hop; redirect count bounded
- [ ] DNS pinned: the connection goes to the validated address (rebinding
      resistance)
- [ ] Response size and time bounded

## D. Protocols and external opens

- [ ] One central guard with a scheme allowlist for every open-in-system
      action; scattered raw calls each need individual review
- [ ] URL length caps; invalid/relative URLs inert

## E. Sandboxes, plugins, embedded runtimes

- [ ] What the embedded runtime may do: permission handler (camera/mic/
      downloads), cookie scope, script access to the host
- [ ] The embedded engine's update cadence is someone's explicit duty
- [ ] Plugins/extensions: who can install, what they can reach, how revoked
- [ ] Sidecar processes: bound to loopback only, authenticated if they serve
      a socket

## F. Process spawning

- [ ] Executable + argument array, never a shell string
- [ ] Executable resolved/validated (absolute path, exists, expected)
- [ ] Arguments validated against an allowlist or strict pattern
- [ ] Child count and lifetime bounded; trees killed on close

## G. Defaults and failure modes

- [ ] Empty allowlist/config = closed, not open
- [ ] Validation errors block the operation
- [ ] Timeouts and dependency failures do not degrade to "allowed"
