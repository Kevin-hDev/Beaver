# Setup Decisions

Read this reference when you resolve branch, bootstrap, visibility, remote, provider, or repository boundaries.

- Prefer an explicit valid branch supplied by the user or project scaffolder.
- Use `init.defaultBranch` when it exists and is valid.
- Use `main` only when no explicit or configured choice exists.
- Treat a parent work tree as a boundary. Do not initialize a child repository without explicit nested-repository intent.
- Treat a nested repository as separate user-owned data. Do not absorb it.
- Keep initialization, contribution guidance, bootstrap commit, remote attachment, remote creation, and push independently selectable.
- Treat an explicit `initialize and publish` request as authorization for the minimum empty bootstrap commit needed to create `HEAD`, but never as authorization to stage existing project content.
- Create `CONTRIBUTING.md` only when explicitly requested. Never overwrite an existing contribution guide.
- Use any available authenticated provider mechanism. Do not require one fixed tool when another configured mechanism can fulfill the same provider operation safely.
- Default a newly created remote to private.
- Never infer permission to replace `origin`, publish all branches, publish tags, transfer ownership, or create ordinary project commits.
