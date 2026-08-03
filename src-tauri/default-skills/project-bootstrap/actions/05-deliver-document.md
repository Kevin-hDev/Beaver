# 05 - Deliver Document

Return the confirmed technical vision in conversation or write one approved Markdown artifact.

## Input

- Use the confirmed checklist, candidate comparison, audit rationales, selected design, mitigations, folder tree, and validated Mermaid diagram.
- Accept an explicit Markdown destination or an existing project documentation convention.

## Output

- Return a complete technical-vision or INSTALL-style document and, only when requested, its validated written path.

## Process

1. **Resolve delivery.** Return the full document in conversation when no file is requested. When a file is requested, use the user's destination first or an already documented project convention that the user accepts. Ask when neither determines one.
2. **Validate the destination.** Require a project-contained Markdown path with no traversal and an existing parent directory. Canonicalize the parent and confirm it stays within the project. Never create a directory.
3. **Protect existing content.** Read an existing destination safely and require explicit overwrite or scoped-update approval. Do not merge silently.
4. **Load the template conditionally.** Use [technical-vision-template.md](../assets/technical-vision-template.md) for a complete technical-vision or INSTALL-style artifact. Preserve any established project format when it covers the same required content.
5. **Fill completely.** Include Vision, Needs, Decisions, Candidate comparison, Stack summary, Architecture, Folder structure, Install steps, Audit summary, Risks and mitigations, and Open questions. Preserve evidence dates and links.
6. **Write safe setup guidance.** Provide three to seven imperative manual setup steps describing future repository, runtime, dependency, account, environment, and infrastructure work. Keep them as documentation; execute none of them.
7. **Verify content.** Check heading order, complete tables, no placeholders, valid Markdown, the already validated Mermaid block, a folder tree of at least five lines, and no secrets or unsupported claims.
8. **Write conditionally.** Write only the approved Markdown file. Re-read it and compare its content with the confirmed document.
9. **Report.** Return the document or path, delivery mode, included sections, and any unresolved open questions.

## Stop conditions

- Stop when the destination is ambiguous, outside the project, has a missing parent, is not Markdown, or would overwrite content without approval.
- Stop when any required section, evidence, mitigation, tree entry, or diagram validation is incomplete.
- Never create directories, code, dependencies, accounts, repositories, configuration, services, or infrastructure.

## Test

- Confirm that the delivered document contains every required section in order and no placeholder remains.
- Confirm that file output occurred only at an approved destination whose parent already existed.
- Confirm that the skill changed no state beyond the one explicitly requested Markdown file.
