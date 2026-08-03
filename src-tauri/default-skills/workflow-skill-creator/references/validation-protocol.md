# Validation Protocol

You validate the complete bundle rather than only its manifest.

1. **Inventory files.** You list a bounded set of regular files, reject symbolic links, and confirm every path resolves inside the bundle.
2. **Validate identity.** You compare the folder name, frontmatter name, evaluation skill name, UI metadata prompt, and every internal identifier.
3. **Validate description.** You check the 250-character limit, trigger examples, exclusions, and overlap with neighboring descriptions.
4. **Validate instructions.** You check direct imperatives, action contracts, stop conditions, observable tests, resource consumers, and diagram usefulness.
5. **Validate references.** You resolve every local Markdown link and confirm every referenced file exists without orphaned resources.
6. **Validate evaluations.** You parse JSON, bound cases and messages, require unique identifiers, and verify both trigger discrimination and behavioral judges.
7. **Run tooling.** You use the destination's validator, linter, and representative execution checks when available.
8. **Report evidence.** You give every file and check a closed status and retain failures or unavailable checks explicitly.
