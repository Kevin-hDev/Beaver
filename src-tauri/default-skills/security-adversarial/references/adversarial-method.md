# Adversarial Method — protocol complet

This reference is the full protocol for `security-adversarial`. Read it before
Round 1. It defines the round structure, the verdict rubric, the report
format, and the optional deep mode with two sub-agents.

## Round structure

### Round 1

1. **RED proposes** attack paths. Each path is a record:
   - Entry point — `file:line` where attacker-influenced data or control
     enters
   - Assumption — what the attacker controls, in one line
   - Flow — the chain from entry to the privileged capability, each hop
     verified in the code
   - Impact — what breaks if the path works (data read, code execution,
     privilege gained, denial of service)
   - Plausibility — high / medium / low, with the reason
2. RED covers the angles that fit the target. Do not force all of them:
   - Untrusted input reaching a privileged sink
   - Confused deputy (a trusted component acting for an untrusted caller)
   - Missing or misplaced authorization
   - Race conditions (check-then-use, shared state)
   - Secret exposure (logs, errors, responses, storage)
   - Unsafe defaults and downgrade paths
3. **BLUE answers** every path, in RED's plausibility order:
   - Defense found → cite `file:line`, quote the control, name the test that
     proves it if one exists (run it when cheap)
   - Partial defense → state exactly which segment of the path is covered and
     which is open
   - No defense found → say "no defense found on this path", never soften

### Round 2

1. **RED replies** only where a crack remains. Legitimate replies:
   - The defense covers a different path, not this one
   - The check runs after the sensitive operation (order bug)
   - The validation lives only in a layer the attacker can skip
   - The test proves a case different from the path proposed
   - The defense has a known weakness class (pattern-matching instead of
     parsing, denylist instead of allowlist, non-constant-time comparison)
2. RED **drops** every path where BLUE's evidence is solid. Killing its own
   weak paths is what makes RED credible — a RED that never concedes is
  theatre, not review.

3. **BLUE closes**: for each surviving objection, produce final evidence from
   the code, or record the final admission. No hypothetical defenses, no "this
   could be fixed by" — only what exists.

### Stop rule

Two rounds, then verdicts. A path still disputed after Round 2 is UNVERIFIED
with the dispute summarized — never force a third round unless the user asks.

## Verdict rubric

| Verdict | Requirements — all mandatory |
|---|---|
| CONFIRMED HOLE | Entry verified `file:line`; flow verified hop by hop; no defense found on this exact path after both rounds; impact stated. The assumption about attacker control must be realistic (not "if the attacker is root"). |
| UNVERIFIED | Evidence incomplete: state what is missing (unreadable generated code, dynamic behavior that needs runtime, code outside the allowed scope) and name the follow-up `security-*` focus skill. |
| BLOCKED | Defense cited `file:line`, covering this exact path, surviving RED's Round 2 reply (or unchallenged because RED conceded). |

A defense "proven" only by intention, comment, or naming is not evidence.
A test that exists but was not run still counts as evidence, one level below
an executed test — say which one you cite.

## Report format

```
ADVERSARIAL REVIEW — {target} — {date}

CONFIRMED HOLES: {n} | UNVERIFIED: {n} | BLOCKED: {n}

### Confirmed holes
1. {path name}
   Entry: {file:line} — Assumption: {attacker controls X}
   Flow: {entry} → {hop} → {capability}
   Impact: {what breaks}
   Round 2 note: {why no defense held}

### Unverified
- {path} — missing: {what} — follow-up: security-{focus}

### Blocked (defenses that held)
- {path} — blocked by {defense} at {file:line}{, test: {name} (run | cited)}

### Minimal fix list
1. {the fix that closes the most / the most severe confirmed hole first}
```

Order of the report never changes: holes first, then unknowns, then what
held. The fix list is minimal on purpose — closing the holes, not redesigning.

## Deep mode — two sub-agents (only on explicit user request)

Default mode is one agent alternating roles. Deep mode deploys two sub-agents:

1. Spawn **RED agent** with the RED briefing: the target frame, the code
   excerpts you gathered in Phase 1, and the instruction to produce Round 1
   paths only (no defenses, no verdicts).
2. Spawn **BLUE agent** with the BLUE briefing: the target frame and RED's
   paths, with the instruction to answer each from the code with `file:line`
   evidence.
3. Relay Round 2: send BLUE's answers back to RED for replies, then RED's
   objections back to BLUE for closing evidence.
4. You remain the referee: you verify the citations both sides produce
   (sub-agents can cite wrong lines), you assign the verdicts, you write the
   report.

Referee duties are not optional: open every cited `file:line` before trusting
it. A citation that does not match demotes the path to UNVERIFIED with
"miscited evidence" as the reason.
