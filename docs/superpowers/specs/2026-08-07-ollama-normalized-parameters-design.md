# Ollama Normalized Parameters Design

## Goal

Make the parameter editor operate on the semantic parameter values stored by Ollama, including values with quotes, edge whitespace, or line breaks. Clearing a field must remove the corresponding override, including an existing multiline parameter. Provider routing, tool selection, chat streaming, model capabilities, and Ollama lifecycle behavior remain unchanged.

## Source Contract

Beaver does not reread the Modelfile it submitted. It recreates the model, then reads the normalized Modelfile returned by Ollama `/api/show`.

Ollama 0.32.5 emits a raw value unless it contains a newline or leading/trailing whitespace. In those cases it adds simple quotes, or triple quotes when the value itself contains a quote. Therefore a quote at the start of a normalized single-line value is not sufficient evidence that a multiline block has started.

## Architecture

Rust becomes the single source of truth for parameter extraction and rewriting. A bounded scanner parses the normalized Modelfile into semantic parameter entries and directive spans. The Tauri read command returns the Modelfile and extracted parameters together, so the frontend no longer implements a second Modelfile parser.

A quoted block is recognized only when a matching closing delimiter is found. An unmatched quote-looking parameter value is treated as literal normalized data, not as an open block. Non-parameter directives such as `SYSTEM`, `TEMPLATE`, and `MESSAGE` are still skipped as complete spans.

When saving, Rust removes every top-level `PARAMETER` span, including multiline spans, and inserts the complete semantic payload supplied by the editor. This restores the invariant that an empty editor field means no override.

## Value Rules

- Parameter keys are trimmed and validated as before.
- Numeric official parameters are trimmed and validated numerically.
- Text and custom values preserve leading/trailing whitespace exactly.
- An empty string is omitted by the frontend; a whitespace-only string remains a real value.
- NUL and carriage return remain forbidden.
- Line feeds are accepted for text values and rendered safely as quoted blocks.
- Three consecutive quotes are rejected with a dedicated translated validation message because Ollama has no reliable escaping syntax for that delimiter.
- Values with a quote, newline, or edge whitespace are rendered using the same quoting contract understood by Ollama's parser.

## Interface

Stop values and custom parameter values use bounded text areas so multiline values are visible, editable, and removable. Existing single-line values keep the same compact appearance. The seven supported languages receive the validation message for unsupported triple-quote values.

## Error Handling

Extraction is bounded by the existing maximum entry and value sizes. A structurally invalid normalized response fails closed with a generic safe application error. User validation errors identify only the unsupported character combination and never expose paths, provider bodies, or internal stack details.

## Tests and Review

Tests must reproduce the complete normalized cycle rather than checking only Beaver's immediate output:

1. semantic value to Beaver rendering;
2. simulated Ollama parse and normalized emission;
3. Beaver extraction;
4. second save without value drift.

Coverage includes isolated quotes, quoted text, edge whitespace, whitespace-only stops, multiline stops, multiline values containing simple quotes, removal of multiline overrides, unmatched quote-looking normalized values, triple-quote rejection, CRLF preservation, and following-parameter replacement. The final review checks all affected callers and confirms that no chat, provider, tool, model-capability, or lifecycle path changed.
