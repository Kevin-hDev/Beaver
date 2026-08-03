# 01 - Inspect

You reconstruct the release contract and requested endpoint.

## Input

- Accept a repository, optional target version, note overrides, and intent to prepare, tag, push, publish, or complete the full release.

## Output

- Return current and target versions, change range, branch, artifacts, notes format, checks, tag pattern, provider mechanism, and endpoint.

## Process

1. **Read rules.** Load release instructions, manifests, automation, changelog policy, notes data, and recent release history.
2. **Inspect state.** Read branch, status, remotes, tags, provider releases, current versions, and the committed change range in bounded passes of at most 200 commits.
3. **Resolve current version.** Use the documented version source. When none exists, use the latest valid release tag. When neither exists and the fallback applies, start from `1.0.0` and state that no prior release exists.
4. **Resolve target.** Validate an explicit version. Otherwise apply documented policy; when incomplete, read [release-fallback.md](../references/release-fallback.md) and compute major for verified `BREAKING CHANGE`, minor for any verified `feat`, otherwise patch.
5. **Check uniqueness.** Verify that version, tag, and provider release do not exist locally or remotely.
6. **Map artifacts.** List every required version file, lockfile, changelog, localized note, generated artifact, commit, tag, and check.
7. **Resolve endpoint.** Distinguish prepare-only, local tag, remote branch and tag, provider release, and deployment. Never expand scope silently.
8. **Resolve provider.** Select an available authenticated connector, CLI, MCP capability, or API only when remote or provider publication is requested.

## Stop conditions

- Stop on conflicting versions, unrelated dirty changes, detached `HEAD`, ambiguous branch, unsafe content, or existing target.
- Continue the history scan in another bounded pass when more than 200 commits exist; do not discard older release changes.
- Ask for an explicit target only when neither project policy nor SemVer fallback can determine one defensibly.

## Test

- Confirm that one target version and endpoint follow explicit input, project rules, or the documented fallback.
- Confirm that every expected mutation and check is listed before preparation.
