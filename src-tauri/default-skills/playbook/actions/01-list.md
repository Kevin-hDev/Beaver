# 01 - List playbooks

You inventory every available project playbook and packaged example without changing project state.

## Input

- Accept an optional project root and optional explicit playbook location.
- Use the latest explicit destination or established project convention when one is already known.

## Output

- Return a table `| # | Playbook | Source | Status | Description |` sorted by source and file name.
- Return `No playbooks yet.` when both resolved homes are absent or empty.

## Process

1. **Resolve homes.** You read [locations.md](../references/locations.md). You resolve the project home without inventing one and use [packaged examples](../assets/examples/) as the read-only packaged home.
2. **Discover files.** You enumerate Markdown playbooks in both homes, excluding indexes and general readme files. You inspect at most 100 entries per ordered batch and continue until all entries are covered.
3. **Parse identity.** You read each H1 title and the single-sentence description immediately below it. You mark malformed files instead of inventing missing metadata.
4. **Apply precedence.** You mark a project playbook `active` and a same-slug packaged example `shadowed`. You mark all other valid entries `available`.
5. **Sort and number.** You sort project entries before packaged entries, then by file name, and assign contiguous display numbers from 1 after sorting.
6. **Render.** You link every title to its actual path and retain malformed entries with a clear status so they can be repaired deliberately.

## Stop conditions

- You stop and ask when multiple plausible project homes exist or an explicit location is unsafe.
- You stop with `blocked` when the canonical project root cannot be established safely.
- You do not create a directory, repair a file, or store the numbered mapping outside the current conversation.

## Test

- Confirm that every discovered project playbook and packaged example appears exactly once.
- Confirm that numbering is contiguous and matches the displayed order.
- Confirm that same-slug project entries are active and packaged entries are shadowed.
- Confirm that an empty or absent home produces `No playbooks yet.` without an error or write.
