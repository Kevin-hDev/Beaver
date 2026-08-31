# Security Map — {project name}

> Produced by the `security-map` skill on {date}. Read-only cartography:
> this map says WHAT to audit, not audit verdicts. Status of every zone is
> CONFIRMED (code read) or SUSPECTED (inferred — verify before trusting).

## 1. Project frame

- **Stack**: {languages, frameworks}
- **App shape**: {web / desktop / CLI / library}
- **Declared security rules**: {from AGENTS.md / project docs — or "none found"}
- **Prior security work**: {existing audit docs, test batteries, their dates}

## 2. Zone inventory

### Zone {n} — {name} `{path}`
- **Evidence**: `{file:line}` (entry), `{file:line}` (capability)
- **Data flow**: {what untrusted data} → {what privileged capability}
- **Protections observed**: {sanitize, validation, allowlist… or "none seen"}
- **Status**: CONFIRMED | SUSPECTED
- **Score**: exposure {1-3} × blast radius {1-3} = **{score}**

{repeat per zone}

## 3. Secret flow summary

- **At rest**: {where secrets live}
- **In motion**: {how they cross layers}
- **Weak points**: {places a secret could escape, with file:line — or "none found"}

## 4. Ranked audit plan

| Rank | Zone | Score | Skill to run |
|---|---|---|---|
| 1 | {zone} | {score} | `security-injection` / `security-secrets` / `security-boundaries` / `security-dependencies` |

## 5. Open questions

- {everything that could not be confirmed read-only}

---
*Next step: run the focused skill at the top of the plan.*
