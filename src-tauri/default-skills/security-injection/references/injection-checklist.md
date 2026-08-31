# Injection checklist — what to verify per surface

## A. Raw-HTML sinks

- [ ] Is the HTML passed through a sanitizer with a known schema (allowed
      tags, attributes, protocols)?
- [ ] Does the schema strip `script`, event handlers (`on*`), `iframe`,
      `object`, `embed`, `form`, `meta`, `base`, `style` when not needed?
- [ ] Are URL attributes restricted to safe protocols (`http`, `https`,
      `mailto`) — including entity-encoded and whitespace-split bypasses?
- [ ] After sanitization, which tags survive with attributes, and is each
      surviving profile inert (e.g. a disabled checkbox, a boolean `open`)?
- [ ] If no sanitizer: is the input guaranteed build-time trusted (static
      bundled assets)? If not guaranteed → finding.

## B. Markdown pipelines

- [ ] Raw HTML enabled (`rehypeRaw`-like)? Then a sanitize stage MUST follow
      it in the plugin order — verify the order, not just the presence.
- [ ] Markdown link/image destinations: hostile schemes (`javascript:`,
      `data:text/html`, `vbscript:`) rejected by the sanitizer AND by the
      click handler?
- [ ] Reference-style links (`[x][1]` definitions) covered too?

## C. Syntax highlighters / code renderers

- [ ] Does the highlighter escape text content by construction (tree
      serializer) or by manual escaping? Manual escaping: is `&` replaced
      FIRST (double-escaping bug otherwise)?
- [ ] Unregistered-language fallback: does it still escape?
- [ ] Can any payload produce a raw tag other than the highlighter's own
      span wrappers? (Lock with a test: strip the spans, assert no `<` left.)

## D. Document / file previews

- [ ] Renderers that convert files to HTML (docx, pdf, xlsx, ebooks): does
      the converted output pass a sanitizer before hitting the DOM?
- [ ] If the platform CSP is the only layer for this surface → single-layer
      report.
- [ ] Archive extraction: paths confined (no `..`, no absolute overwrite),
      sizes bounded, symlinks rejected?

## E. Navigation sinks

- [ ] Every open-in-browser / redirect goes through ONE central guard with a
      scheme allowlist — or are there scattered raw calls? (Scattered = each
      one must be verified separately.)
- [ ] Length caps on URLs; invalid/relative URLs inert.

## F. Execution sinks

- [ ] `eval`, `new Function`, string-built workers/imports: none should
      exist on production paths fed by external content. Any hit = HIGH.

## G. Platform layer

- [ ] CSP or equivalent present and strict on scripts (`script-src 'self'`
      or nonces) — inline handlers and `javascript:` URLs blocked even if
      the sanitizer regresses.
- [ ] Dev-mode relaxations never shipped to production builds.
