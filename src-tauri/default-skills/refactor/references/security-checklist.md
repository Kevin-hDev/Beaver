# Security Checklist

Use the current OWASP Top 10 categories as a review frame, then follow the project's actual trust boundaries and technology-specific risks. A checklist item is not a finding until current code makes it reachable.

## External boundaries

- You validate type, length, range, format, encoding, and allowed characters before processing.
- You confine file paths through canonical resolution and an allowed root; you reject traversal and unsafe links.
- You bound every externally fed collection and expensive parse, upload, decompression, recursion, or retry path.
- You reject malformed or ambiguous serialization and fail closed on validation errors.

## Identity and access

- You authenticate at the trusted boundary and authorize the exact resource and operation.
- You preserve tenant, role, and ownership context across calls and asynchronous work.
- You deny on lookup, policy, parsing, or dependency failure.
- You protect sessions, recovery paths, rate limits, and privileged state transitions.

## Injection and outbound access

- You use prepared database statements and structural APIs instead of string construction.
- You pass system arguments separately without a shell and validate allowed executable arguments.
- You escape output for its exact HTML, template, header, or log context.
- You restrict outbound schemes, hosts, redirects, ports, and resolved destinations where untrusted input can influence a request.

## Secrets and cryptography

- You keep secrets outside source code and user-visible errors and filter them from all logs.
- You compare secrets in constant time and zeroize mutable secret buffers after use when the runtime permits it.
- You use a cryptographically secure random generator for tokens, identifiers, nonces, and reset links.
- You use maintained authenticated encryption and established password hashing APIs; you do not design custom cryptography.

## Fix evidence

- You create one focused regression per fix and prove it fails on the pre-fix state.
- You test bypasses, malformed inputs, authorization denial, and failure paths relevant to the fix.
- You record intentional behavior changes and the durable protection in the project's established documentation location.
