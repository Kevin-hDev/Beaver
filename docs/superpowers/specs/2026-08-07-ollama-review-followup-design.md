# Ollama Review Follow-up Design

## Goal

Close the remaining review defects without changing provider routing, tool selection, model capabilities, or Ollama lifecycle behavior.

## Parameter value contract

The settings UI and backend must exchange semantic parameter values, not Modelfile syntax. The frontend decodes complete single-line quoted values when reading a Modelfile. The backend never strips quotes supplied by the user and renders any text containing a quote inside triple quotes. The required invariant is that parsing a rendered value returns the exact submitted value, including `"`, `""`, embedded quotes, and Windows backslashes.

## Multiline directives

The line scanner tracks whether a quoted value is complete. Multiline `SYSTEM`, `TEMPLATE`, and other non-parameter directives are preserved. Multiline `PARAMETER` spans are also preserved byte-for-byte because the current single-line settings editor cannot safely edit them. The frontend omits these unsupported spans from the editable parameter list, so their opening delimiter is never submitted as a value.

## Unknown context diagnostics

A zero context window remains the existing internal sentinel for an unknown model capacity. Structured details accept that sentinel, and the frontend interprets it only as “unknown”; it never displays zero as a real window size. Known counters remain validated and displayed, while a dedicated translated message states that the real model window is unknown. No synthetic window value is reported.

## Error handling and scope

- User parameter values remain bounded and line breaks remain rejected.
- The unreachable newline-rendering branch is removed.
- Existing valid Modelfile directives and CRLF line endings are preserved.
- No raw provider body, internal path, or encoded diagnostic reaches the UI.

## Verification

Regression tests cover literal one- and two-quote stop values, embedded quotes, semantic frontend decoding, multiline parameter preservation for simple and triple quotes, following parameter removal, and unknown-window diagnostics in Rust and the UI. Targeted tests run red before production edits and green afterward, followed by the broader Ollama, frontend, Rust, lint, and type-check suites.
