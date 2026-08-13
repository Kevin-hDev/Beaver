# Native CEF CI Deadline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rendre le parcours CEF natif déterministe et faire apparaître l'étape exacte de tout blocage macOS.

**Architecture:** Une politique Node indépendante possède l'unique échéance monotone du parcours. Le scénario WebDriver lui délègue chaque opération asynchrone et la configuration Mocha dérive sa marge de nettoyage de cette même politique.

**Tech Stack:** Node.js 24, `node:test`, TypeScript, WebdriverIO/Mocha.

## Global Constraints

- Tu ne modifies pas le comportement de production de Beaver sans preuve d'un blocage interne.
- Tu dérives chaque budget local de l'échéance globale partagée.
- Tu limites et valides chaque nom d'étape avant de l'écrire dans les traces.
- Tu conserves chaque fichier de code sous 230 lignes.

---

### Task 1: Autorité temporelle du parcours natif

**Files:**
- Create: `scripts/e2e/native-journey-deadline.mjs`
- Create: `scripts/e2e/native-journey-deadline.d.mts`
- Create: `scripts/e2e/native-journey-deadline.test.mjs`

**Interfaces:**
- Produces: `createNativeJourney({ now?, report? })`
- Produces: `journey.run(stage, ceilingMs, operation)`
- Produces: `NATIVE_JOURNEY_TIMEOUT_MS` and `NATIVE_JOURNEY_MOCHA_TIMEOUT_MS`

- [ ] **Step 1: Write the failing deadline tests**

  Tu testes avec une horloge injectée qu'une seconde étape ne reçoit que le
  temps global restant, qu'une opération suspendue échoue avec son nom, et que
  les événements `started/completed/failed` sont produits sans donnée sensible.

- [ ] **Step 2: Run the tests and verify RED**

  Run: `node --test scripts/e2e/native-journey-deadline.test.mjs`

  Expected: FAIL because `native-journey-deadline.mjs` does not exist.

- [ ] **Step 3: Implement the minimal authority**

  Tu utilises `performance.now()` par défaut, un `AbortController` local pour
  le minuteur, `Promise.race` pour la frontière asynchrone et une validation
  stricte du nom d'étape. Tu annules toujours le minuteur dans `finally`.

- [ ] **Step 4: Run the tests and verify GREEN**

  Run: `node --test scripts/e2e/native-journey-deadline.test.mjs`

  Expected: all tests pass.

### Task 2: Adoption complète par le parcours CEF

**Files:**
- Modify: `tests/e2e/native-cef-shutdown.spec.ts`
- Modify: `wdio.conf.ts`
- Test: `scripts/e2e/e2e-process.test.mjs`

**Interfaces:**
- Consumes: `createNativeJourney`, `NATIVE_JOURNEY_MOCHA_TIMEOUT_MS`
- Preserves: page title `Beaver CEF smoke`, helper observation, coordinated exit

- [ ] **Step 1: Write the failing adoption test**

  Tu exécutes un contrôle comportemental qui importe la configuration et
  vérifie que le délai Mocha est celui de la politique. Tu ajoutes au contrôle
  E2E existant la preuve que le scénario passe ses frontières asynchrones par
  l'autorité du parcours.

- [ ] **Step 2: Run the adoption test and verify RED**

  Run: `node --test scripts/e2e/e2e-process.test.mjs`

  Expected: FAIL because the current scenario and configuration do not consume
  the shared policy.

- [ ] **Step 3: Adopt the authority**

  Tu enveloppes l'accueil, l'observation native, la capacité, l'ouverture, la
  surface, le helper, le chargement, la demande de sortie, la suppression de
  session et les observations de fin. Tu gardes la fermeture du serveur dans
  un `finally` borné.

- [ ] **Step 4: Run targeted verification**

  Run: `node --test scripts/e2e/native-journey-deadline.test.mjs scripts/e2e/e2e-process.test.mjs scripts/e2e/native-cef-observer.test.mjs`

  Expected: all tests pass.

### Task 3: Vérification et livraison

**Files:**
- Modify: `graphify-out/` through `graphify update .`
- Add: git note on the implementation commit

- [ ] **Step 1: Run static and frontend verification**

  Run: `npx tsc --noEmit`

  Run: `npm test`

  Expected: all checks pass.

- [ ] **Step 2: Check file sizes and diff**

  Run: `git diff --check`

  Run: `Get-Content <changed-code-file> | Measure-Object -Line`

  Expected: no whitespace error and every code file remains under 230 lines.

- [ ] **Step 3: Refresh the project graph**

  Run: `graphify update .`

  Expected: successful incremental update.

- [ ] **Step 4: Commit, annotate and push**

  Tu crées un commit ciblé, ajoutes une git note qui décrit le comportement,
  la raison et les preuves réellement exécutées, puis pousses le commit et les
  notes sur la branche de la PR.
