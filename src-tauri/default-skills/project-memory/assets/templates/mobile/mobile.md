# Mobile

## Platform

- Targets and framework: {iOS, Android, cross-platform, or native stack.}
- Application entry points: `{paths}`

## Navigation and state

- Navigation: {verified router and macro flow source.}
- State and local storage: {verified libraries and persistence boundaries.}

## Native access

| Capability | Platform boundary | Permission handling | Evidence |
| --- | --- | --- | --- |
| {camera/location/push/etc.} | {native module} | {request and denial behavior} | `{path}` |

## Build and release

- {Build tooling, signing boundary, store distribution, and update flow without secrets.}

<!-- Capture macro platform behavior, not every screen. Remove placeholders and this comment. -->
