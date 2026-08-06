# Contributing to Beaver

Thanks for taking the time to contribute.

## License and CLA

Beaver is licensed under the **GNU Affero General Public License v3.0**
([LICENSE](LICENSE)).

Before your first pull request is merged, you must sign the
**[Contributor License Agreement](CLA.md)**. You keep ownership of your work;
the CLA grants the project the rights it needs to keep publishing Beaver and to
offer commercial licenses alongside the AGPL.

To sign, comment on your first pull request with:

```
I have read the Beaver CLA v1.0 and I agree to it. Signed: <Full Name> <email>
```

## Getting started

```bash
npm install
cd src-tauri && bash scripts/download-ollama.sh   # first run only
npm run tauri dev
```

## Before opening a pull request

Run the full check suite and make sure everything passes:

```bash
npm run lint                                      # frontend lint + React boundaries
npx tsc --noEmit                                  # TypeScript
npm test                                          # frontend, CEF browser, extension host
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo test
```

## Code conventions

The full ruleset lives in [CLAUDE.md](CLAUDE.md). The essentials:

- **One responsibility per file.** Past 230 lines, stop and check — the file
  almost certainly carries several. Comfortable target: 50 to 150 lines.
- **Paths** always go through `crate::services::paths::data_dir()`, never
  hardcoded.
- **API keys** never reach JavaScript. Rust loads them at call time and zeroizes
  them afterwards.
- **Colors** always use a `var(--token)` from `src/styles/themes/`. If the token
  is missing, create it in both themes.
- **User-facing text** ships in all seven languages (fr, en, es, de, it, zh, ja)
  through `src/i18n/`. No hardcoded strings.
- **Bounded collections**: every externally fed collection needs a maximum size
  and an eviction policy.

## Reporting a security issue

Do not open a public issue. Follow the process in
[SECURITY.md](SECURITY.md).
