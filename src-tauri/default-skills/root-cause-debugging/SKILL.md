---
name: root-cause-debugging
description: Systematic debugging with differential diagnosis — 3 mandatory hypotheses, root cause tracing, learnings log
argument-hint: "<bug description>"
---

# Systematic Debugging

Random fixes waste time and create new bugs. Quick patches mask underlying issues.

**Core principle:** ALWAYS find root cause before attempting fixes. Symptom fixes are failure.

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

If you haven't completed Phase 1, you cannot propose fixes.

## When to Use

Use for ANY technical issue: test failures, bugs, unexpected behavior, performance problems, build failures, integration issues.

**Use this ESPECIALLY when:**
- Under time pressure (emergencies make guessing tempting)
- "Just one quick fix" seems obvious
- You've already tried multiple fixes
- Previous fix didn't work

**Don't skip when:**
- Issue seems simple (simple bugs have root causes too)
- You're in a hurry (systematic debugging is FASTER than guess-and-check)

## The Five Phases

Complete each phase before proceeding to the next.

---

### Phase 1: Root Cause Investigation

**BEFORE attempting ANY fix:**

**1.1 Read Error Messages Carefully**
- Don't skip past errors or warnings — they often contain the exact solution
- Read stack traces completely
- Note line numbers, file paths, error codes

**1.2 Reproduce Consistently**
- Can you trigger it reliably? What are the exact steps?
- Does it happen every time?
- If not reproducible → gather more data, don't guess

**1.3 Check Recent Changes**
- What changed? Git diff, recent commits
- New dependencies, config changes
- Environmental differences

**1.4 Gather Evidence in Multi-Component Systems**

When system has multiple components (CI → build → signing, API → service → database):

```
For EACH component boundary:
  - Log what data enters component
  - Log what data exits component
  - Verify environment/config propagation
  - Check state at each layer

Run once to gather evidence showing WHERE it breaks
THEN analyze evidence to identify failing component
THEN investigate that specific component
```

**1.5 Trace Data Flow (Root Cause Tracing)**

When error is deep in call stack:
- Where does bad value originate?
- What called this with bad value?
- Keep tracing UP until you find the source
- Fix at source, not at symptom

```
Found immediate cause
  → Can trace one level up? → Trace backwards
  → Is this the source? → No → Keep tracing
  → Yes → Fix at source + add validation at each layer
```

**NEVER fix just where the error appears.** Trace back to find the original trigger.

---

### Phase 2: Pattern Analysis

**Find the pattern before fixing:**

1. **Find Working Examples** — Locate similar working code in same codebase
2. **Compare Against References** — Read reference implementation COMPLETELY, don't skim
3. **Identify Differences** — List every difference, however small. Don't assume "that can't matter"
4. **Understand Dependencies** — What settings, config, environment does it need?

---

### Phase 3: Three Hypotheses (mandatory)

**Form exactly 3 hypotheses before any correction.**

```
Hypothesis 1 (primary)    : [most probable cause]
  Justification           : [1 sentence]
  Quick test              : [command or check to confirm/refute]

Hypothesis 2 (alternative): [other plausible explanation]
  Justification           : [1 sentence]
  Quick test              : [command or check]

Hypothesis 3 (non-obvious): [less evident but possible]
  Justification           : [1 sentence]
  Quick test              : [command or check]
```

**Strict rule**: if you can't formulate 3 hypotheses → force deeper thinking. Do NOT proceed to Phase 4.

Why 3 instead of 1: a single hypothesis creates confirmation bias. 3 forces you to consider alternatives and prevents tunnel vision.

---

### Phase 4: Differential Diagnosis

**Test ALL 3 hypotheses BEFORE fixing anything.**

For each hypothesis:
1. Execute the quick test defined in Phase 3
2. Record the exact result
3. Classify: `confirmed` / `refuted` / `inconclusive`

```
Hypothesis 1 (primary)    : [confirmed | refuted | inconclusive]
  Test result             : [exact output]

Hypothesis 2 (alternative): [confirmed | refuted | inconclusive]
  Test result             : [exact output]

Hypothesis 3 (non-obvious): [confirmed | refuted | inconclusive]
  Test result             : [exact output]
```

**If no hypothesis confirmed** → return to Phase 3 with 3 NEW hypotheses.
Never fix based on an inconclusive hypothesis.

**If multiple confirmed** → fix the primary, note the others as contributing factors.

---

### Phase 5: Implementation

**Fix the root cause, not the symptom.**

**5.1 Create Failing Test Case**
- Simplest possible reproduction
- Automated test if possible
- MUST have before fixing

**5.2 Implement Single Fix**
- Address the confirmed root cause
- ONE change at a time
- No "while I'm here" improvements
- No bundled refactoring

**5.3 Verify Fix**
- Test passes now?
- No other tests broken?
- Issue actually resolved?
- Run: lint + build + targeted tests

**5.4 If Fix Doesn't Work**
- STOP. Count: How many fixes have you tried?
- If < 3: Return to Phase 1, re-analyze with new information
- **If >= 3: STOP and question the architecture (see below)**

**5.5 If 3+ Fixes Failed: Question Architecture**

Pattern indicating architectural problem:
- Each fix reveals new shared state/coupling in different place
- Fixes require massive refactoring
- Each fix creates new symptoms elsewhere

**STOP and question fundamentals:**
- Is this pattern fundamentally sound?
- Should we refactor architecture vs. continue fixing symptoms?
- Discuss with user before attempting more fixes

This is NOT a failed hypothesis — this is a wrong architecture.

**5.6 Defense-in-Depth**

After fixing, add validation at EVERY layer data passes through:
- **Layer 1 — Entry point**: reject invalid input at API boundary
- **Layer 2 — Business logic**: ensure data makes sense for this operation
- **Layer 3 — Environment guards**: prevent dangerous operations in specific contexts
- **Layer 4 — Debug instrumentation**: capture context for forensics

Single validation: "We fixed the bug." Four layers: "We made the bug impossible."

---

### Phase 6: Record Learning

Document the root cause (not the symptom) in `{project}-output/learnings.md`:

```markdown
---
date: {ISO 8601}
task: {task_id or description}
category: bug
---
Bug: {symptom observed}
Root cause: {confirmed hypothesis}
Fix: {what was changed}
Defense: {layers added, if any}
Pattern to avoid: {if applicable}
```

If the bug reveals a recurring pattern → note it explicitly for future reference.

---

## Red Flags — STOP and Follow Process

If you catch yourself thinking:
- "Quick fix for now, investigate later"
- "Just try changing X and see if it works"
- "Add multiple changes, run tests"
- "Skip the test, I'll manually verify"
- "It's probably X, let me fix that"
- "I don't fully understand but this might work"
- "Here are the main problems: [lists fixes without investigation]"
- Proposing solutions before tracing data flow
- **"One more fix attempt" (when already tried 2+)**
- **Each fix reveals new problem in different place**

**ALL of these mean: STOP. Return to Phase 1.**

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Issue is simple, don't need process" | Simple issues have root causes too. Process is fast for simple bugs. |
| "Emergency, no time for process" | Systematic debugging is FASTER than guess-and-check thrashing. |
| "Just try this first, then investigate" | First fix sets the pattern. Do it right from the start. |
| "I'll write test after confirming fix works" | Untested fixes don't stick. Test first proves it. |
| "Multiple fixes at once saves time" | Can't isolate what worked. Causes new bugs. |
| "I see the problem, let me fix it" | Seeing symptoms ≠ understanding root cause. |
| "One more fix attempt" (after 2+ failures) | 3+ failures = architectural problem. Question pattern, don't fix again. |

## H.K Workflow Integration

This skill is called automatically in two situations:

**From step-03-execute (Ascent loop):**
If 3 iterations fail → before asking the user for help, load this skill and apply all 6 phases. If debug resolves the problem → resume the wave.

**From step-04-validate (SelfCheck):**
If SelfCheck = FAIL (< 3/4) → before returning to Execute, apply root-cause-debugging on the failed questions.

## Quick Reference

| Phase | Key Activities | Success Criteria |
|-------|---------------|------------------|
| **1. Root Cause** | Read errors, reproduce, check changes, trace data flow | Understand WHAT and WHY |
| **2. Pattern** | Find working examples, compare differences | Identify what's different |
| **3. Hypotheses** | Form exactly 3 theories with justification | 3 testable hypotheses ready |
| **4. Differential** | Test all 3 hypotheses, classify results | At least 1 confirmed |
| **5. Implementation** | Create test, fix root cause, verify, defense-in-depth | Bug resolved, tests pass |
| **6. Learning** | Document root cause, fix, pattern to avoid | `{project}-output/learnings.md` updated |

User: $ARGUMENTS
