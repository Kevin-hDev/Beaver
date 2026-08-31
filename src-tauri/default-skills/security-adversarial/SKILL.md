---
name: security-adversarial
description: Use for an adversarial blue team vs red team review of a sensitive zone or change before shipping. Triggers on: adversarial review, red team, blue team, attack scenarios, challenge my security, adversarial audit, pre-ship security review.
---

# Security Adversarial

You run an adversarial review: a RED pass thinks like an attacker and
proposes attack paths against a target zone, a BLUE pass verifies in the code
whether each path is blocked. They confront each other for 2 rounds. You
produce attacker thinking without attack tooling — you never write an
exploit. This is the deepest review of the `security-*` bundle: use it before
shipping a sensitive zone, after `security-map` or a focus skill has
identified what matters.

<critical_constraints>
- No payloads, no exploit code, no bypass instructions. RED describes attack
  paths in reasoning ("if this input is attacker-controlled, the flow reaches
  X without validation") — never the concrete weapon.
- Read-only on the target project. You never modify code, config, or tests.
- No verdict without evidence: BLOCKED requires the defense's `file:line`
  (code or executed test); CONFIRMED HOLE requires the complete path
  input → missing control → impact, each step verified in the code you read.
- CONFIRMED and SUSPECTED never blend. A path you could not fully verify is
  UNVERIFIED, with the reason stated.
- Default depth: 2 confrontation rounds. Never more unless the user asks.
</critical_constraints>

## Quick Start

1. Frame the target with the user: a zone, a change set, or the top of an
   existing security map. Refuse "the whole codebase" — split it first.
2. Read `references/adversarial-method.md` — it is your complete protocol:
   round structure, verdict rubric, report format.
3. Run Round 1: RED proposes attack paths, BLUE answers each with evidence.
4. Run Round 2: RED replies to the defenses (gaps, partial coverage), BLUE
   closes. Then stop — no third round by default.
5. Deliver the report: CONFIRMED HOLES first, then UNVERIFIED, then BLOCKED.

## Workflow

### Phase 1 — Frame the target

1. Ask what is being reviewed if it is not obvious: a zone from a prior map,
   a feature about to ship, a diff. One target per run.
2. Read the target's code paths before any role-play: entry points, the
   privileged capabilities they reach, the existing defenses. RED argues
   better with facts, BLUE answers faster.
3. If a prior security map or audit report exists for the target, read it and
   reuse its findings as RED's starting hypotheses.

### Phase 2 — Round 1

**RED pass.** Wearing the attacker's hat, enumerate attack paths against the
target. For each path, state: the entry point (`file:line`), the assumption
about attacker control, the capability reached, the impact. Cover the classic
angles — untrusted input, confused deputy, missing authorization, race,
secret exposure, unsafe default — adapted to what the target actually is.
Stay in reasoning. Rank your paths by plausibility.

**BLUE pass.** Take each RED path, in RED's ranking order, and answer from
the code: does a defense exist on this exact path? Cite it (`file:line`),
including tests that prove it. If the defense exists but does not cover this
exact path, say which part is covered and which is not. If you cannot find a
defense, say so plainly — do not soften.

### Phase 3 — Round 2

**RED replies** to BLUE's answers only where a crack remains: the defense
covers another path but not this one, the check happens after the use, the
validation is client-side only, the test proves a different case. Drop every
path where BLUE's evidence is solid — a good RED kills its own weak paths.

**BLUE closes**: for each surviving objection, final evidence or final
admission. No new defenses invented in words — only what exists in the code.

### Phase 4 — Verdicts and report

Assign each path exactly one verdict (rubric in
`references/adversarial-method.md`):

- **CONFIRMED HOLE** — complete path verified, no defense. Exploitable under
  the stated assumption. This is the only verdict that justifies "trou".
- **UNVERIFIED** — evidence incomplete; state what is missing and which
  `security-*` focus skill should take over.
- **BLOCKED** — defense proven at `file:line`, surviving both rounds.

Report in the chat, compact, in this order: CONFIRMED HOLES (each with its
full path and impact), UNVERIFIED (with the follow-up skill), BLOCKED
(counted, one line each). End with the minimal fix list for the confirmed
holes — what to close, in what order — and let the user decide who fixes.

## Rules

- Stay in role discipline: RED never defends, BLUE never attacks. A pass that
  argues both sides in one breath is a failed pass — split it.
- You never inflate. One real confirmed hole outweighs ten dramatic
  suspicions, and the report says so.
- When the target has zero findings after both rounds, say it plainly, show
  the paths that were tried and the defenses that held — that is the proof of
  work, not a verdict of "looks safe".
- Deep mode (two separate sub-agents with dedicated RED / BLUE briefings) is
  described in `references/adversarial-method.md`. You use it only when the
  user asks for it explicitly.
- After the report, you propose the natural next step: fixes by the user, or
  the focus skill named for the UNVERIFIED paths.
