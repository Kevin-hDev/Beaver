# The Ten Reflexes — checkup version

Apply each reflex to the changed code. For each: search the pattern, then
judge CONFORM / VIOLATION (`file:line`) / UNDETERMINED. These are universal
secure-coding rules; adapt the examples to the project's language.

## 1. Constant-time secret comparison

- **Search**: equality operators (`==`, `===`, `equals`) on tokens, hashes,
  API keys, passwords, HMACs.
- **Violation**: a secret compared with a plain equality operator — the
  timing difference leaks the secret byte by byte.
- **Conform**: a constant-time comparison (`subtle`, `ct_eq`, `crypto.timingSafeEqual`,
  XOR byte-by-byte loops).

## 2. Bounded externally-fed collections

- **Search**: new `Map`/`Set`/`List`/array/cache fields in the diff that store
  anything derived from requests, messages, files, or model output.
- **Violation**: insertion with no maximum size and no eviction.
- **Conform**: an explicit cap + eviction, or a bounded structure.

## 3. Secrets in a secure store only

- **Search**: string literals that look like keys/tokens/passwords; secrets
  written to config files, session stores, or the frontend layer.
- **Violation**: a secret in source code, or a secret made readable by a
  less-trusted layer (a `get` exposed over IPC, plaintext storage).
- **Conform**: OS keystore / vault / secure env var; fixtures with obviously
  fake values are not findings.

## 4. Input validation before processing

- **Search**: new parsing/handling of external input (params, bodies, paths,
  URLs, files, messages).
- **Violation**: input used without checking type, length, format, characters;
  SQL built by concatenation; HTML rendered unescaped; paths without
  traversal protection (`..`, symlink escape, confinement to an allowed root).
- **Conform**: validation first, parameterized queries, escaped output,
  canonicalized + confined paths.

## 5. Generic user-facing error messages

- **Search**: new error construction reaching the user/UI.
- **Violation**: file paths, stack traces, table/column names, library
  versions, or raw backend errors in a visible message.
- **Conform**: a generic stable message; details go to logs after redaction.

## 6. CSPRNG for tokens, IDs, nonces

- **Search**: random generation in the diff (`random`, `Math.random`, UUIDs,
  custom ID builders).
- **Violation**: a predictable generator (e.g. `Math.random`, timestamps,
  counters) used for a token, session ID, nonce, or reset link.
- **Conform**: `crypto.randomBytes`, `OsRng`, `SecureRandom`, `crypto.randomUUID`.

## 7. System arguments as a separate list

- **Search**: process spawning (`spawn`, `exec`, `system`, `ProcessBuilder`).
- **Violation**: a shell string built from variables; no regex/allowlist
  validation of the arguments; `shell: true`-style indirection.
- **Conform**: executable + argument array, no shell, arguments validated.

## 8. Fail CLOSED on errors

- **Search**: new `catch`/`except`/`Result` handling in the diff.
- **Violation**: a catch that swallows the error and continues the privileged
  flow (empty catch, fallback to "allowed", default-open policy).
- **Conform**: the error is handled or propagated so the operation blocks.

## 9. Sensitive data filtered from every log

- **Search**: new log/console/debug calls; new log sinks.
- **Violation**: tokens, passwords, keys, raw HTTP bodies, or message contents
  reaching a log, console, or debug channel.
- **Conform**: redaction before write; stable generic error categories.

## 10. Zeroize secret buffers after use

- **Search**: buffers/variables holding secrets in the diff.
- **Violation**: secrets left to the garbage collector when the language
  offers explicit clearing.
- **Conform**: zeroizing types or explicit clearing after use, where the
  runtime permits it.

## Verdict discipline

- CONFORM requires that you actually found the protective code and can cite it.
- If the language makes a reflex inapplicable (e.g. no manual memory control),
  mark it NOT APPLICABLE with the reason — never silently skip.
