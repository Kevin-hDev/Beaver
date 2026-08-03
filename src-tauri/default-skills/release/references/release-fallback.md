# Release Fallback

Read this reference only when the repository has no complete release convention.

- Use strict `major.minor.patch` SemVer and allow a suffix only when explicitly requested.
- Read current version from a version-manager file, then the latest valid tag, then `1.0.0` when neither exists.
- Compute major for a verified `BREAKING CHANGE`, minor for any verified `feat`, otherwise patch.
- Use the repository's existing tag prefix and annotated or signed style; use an annotated `v<version>` tag when history provides no style.
- Require release notes and include only evidenced summary, features, fixes, breaking changes, migrations, verification, and limitations.
- Limit the release commit to required version and release-note artifacts.
- Never infer deployment, asset building, signing, publication, or latest-release marking from tag creation alone.
