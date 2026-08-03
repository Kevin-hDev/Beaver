# 04 - Publish

You attach or create the explicitly requested remote and push one verified local branch.

## Input

- Accept an initialized repository, explicit publication endpoint, provider mechanism, repository identity, visibility, remote name or URL, and a valid local `HEAD`.

## Output

- Return provider, remote URL, visibility, pushed branch, local and remote SHA, or a precise partial outcome.

## Process

1. **Ensure a pushable HEAD.** Require an existing commit. When the original request explicitly asked for end-to-end initialization and publication, complete `03-bootstrap` first if needed.
2. **Preflight content.** Require a clean publication scope, no suspected secrets in tracked history, and no conflicting remote.
3. **Resolve provider.** Use the available authenticated connector, CLI, MCP capability, or API selected during inspection. Validate owner, repository name, and explicit or private-default visibility.
4. **Check existence.** Search for the target remote identity and stop before overwriting or repurposing an existing repository.
5. **Create conditionally.** Create one remote repository only when requested, then verify identity and visibility.
6. **Attach conditionally.** Add the exact supplied or returned URL only when attachment was requested and the remote name is free.
7. **Push normally.** Push the current verified branch once without force and establish upstream only for that branch.
8. **Verify remote.** Confirm that the remote branch SHA matches local `HEAD`. Report remote creation, attachment, and push separately.

## Stop conditions

- Stop when no commit exists outside an explicitly authorized end-to-end bootstrap, a secret may be present, the remote identity conflicts, or no provider mechanism is available.
- Never create a public repository unless visibility was explicit.
- Preserve a remotely created repository when attachment or push later fails and report the partial state.

## Test

- Confirm provider identity and visibility when a remote was created.
- Confirm that the remote branch matches local `HEAD` when pushed.
- Confirm that no other branch, remote, setting, or project file changed.
