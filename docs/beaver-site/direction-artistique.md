# Beaver — Direction Artistique du site

> Document de travail v1 — 2026-07-20. À itérer avant toute ligne de code.

---

## 1. Identité & positionnement

**Beaver** — le castor bâtisseur. L'agent qui construit, branche par branche.

L'app n'est PAS "local-first" : c'est un atelier tout-en-un. LLM locaux via Ollama,
10 providers cloud, comptes web (OpenAI/Grok/Kimi), forecasting, terminal intégré,
Git, MCP, wakeups. Le local est une force parmi d'autres (privacy quand tu veux,
cloud quand tu veux).

### Le positionnement clé : un harnais, pas une app fermée

Beaver est un **harnais personnalisable** (harness). Avec le système d'extensions
et l'hôte, l'utilisateur peut créer et connecter n'importe quelle extension et
**retirer ou remplacer ce que l'app contient déjà** : outils, system prompt,
features, et tout le frontend — panneaux, boutons, vues, thèmes.
Ce n'est pas un fork du projet : c'est le même Beaver, remodulé par l'utilisateur.
Le site doit vendre ça comme un pilier à part entière (« hackable », pas
« extensible ») — c'est ce qui distingue Beaver des apps agentiques fermées.

### Pitch

- **Accroche principale** : « Tout est déjà là — et tu peux tout reprendre. »
  (depuis août 2026 : le harnais est la thèse du site ; la métaphore castor
  devient la colonne vertébrale visuelle — barrage, sciure, mascotte)
- **Sous-titre** : « Local, cloud, ou les deux. Un seul endroit, toutes tes machines à penser. »
- **Accroche historique** (conservée pour la métaphore) : « L'agent qui
  construit, branche par branche. »
- **Version EN** : "Beaver builds while you sleep." / "The agent that builds, branch by branch."

### La métaphore castor = les vraies features

| Métaphore | Feature réelle |
|---|---|
| Branche par branche | Session branching, todos, Plan mode |
| Le barrage s'empile | L'agent enchaîne les tâches et les outils |
| Nocturne, bosse en continu | Wakeups programmés, gateway Telegram/Slack/Discord |
| Transforme son environnement | Fichiers, Git, terminal, MCP |
| Le castor voit loin (la rivière) | **Forecasting** — le différenciateur qu'aucun concurrent n'a |

### Ton

Outil sérieux et pro (l'app est sobre), mais le site se lâche :
rétro-terminal très moderne, jamais corporate, jamais "landing SaaS IA".

### Anti-références (à ne JAMAIS faire)

- Dégradé violet-bleu + glassmorphism + hero générique (le "flop IA")
- vibe-coding.fr : landing SaaS interchangeable
- Grille de 3 features avec icônes génériques
- Texte en gradient, cards arrondies uniformes

---

## 2. Références digérées

| Site | Ce qu'on prend |
|---|---|
| Active Theory | Base sombre atmosphérique, mouvement organique (particules), scroll qui construit la scène, nav pilule minimale |
| Glenn Catteeuw | Typographie display condensée géante en all-caps, mono uppercase pour le courant, infos en blocs, CTA typographiques |
| OpenClaw | Un seul accent couleur, dot-matrix en texture, italique serif de tension dans les titres, bloc terminal d'install à onglets, mascotte |
| wodniack.dev | Esprit mono/binaire, loader à compteur, animations GSAP |
| microsoft.ai | Respiration éditoriale, espace, ton posé (pour les sections texte) |

---

## 3. Palette

Fond quasi-noir, un seul accent. Pas de dégradé criard.

| Token | Rôle | Valeur indicative |
|---|---|---|
| `--bg` | Fond principal | `#0A0908` (noir chaud, presque bois) |
| `--bg-raise` | Surfaces | `#141210` |
| `--ink` | Texte principal | `#F2EDE6` (blanc cassé chaud) |
| `--ink-dim` | Texte secondaire | `#8A8378` |
| `--accent` | **L'unique accent** | `#E8862E` ambre boisé (écorce, sciure, rivière au couchant) |
| `--accent-dim` | Accent atténué | `#E8862E` à 40% |
| `--line` | Bordures / grille | `#26221D` |
| `--term-green` | Détails terminal uniquement | `#7DBB7D` (usage rare, clin d'œil) |

Règles : l'accent sert pour les CTA, les mots-clés, l'état actif. Jamais en fond de section.
Textures autorisées : dot-matrix, grille blueprint, grain subtil, scanlines très légères.

### Thème light (obligatoire — l'app a déjà dark + light)

| Token | Rôle | Valeur indicative |
|---|---|---|
| `--bg` | Fond principal | `#F4EFE7` (papier chaud) |
| `--bg-raise` | Surfaces | `#FFFDF9` |
| `--ink` | Texte principal | `#1A1713` |
| `--ink-dim` | Texte secondaire | `#6E6558` |
| `--accent` | Accent (assombri pour le contraste) | `#C2651B` |
| `--line` | Bordures / grille | `#DCD2C2` |
| `--term-green` | Détails terminal | `#3E7D3E` |

Le toggle vit dans la nav-pill, persiste en localStorage, respecte
`prefers-color-scheme` au premier chargement. Mêmes typos, mêmes animations :
seuls les tokens changent (une seule couche de variables CSS, comme dans l'app).

## 4. Typographie

Pas de polices "classiques" (Inter, Roboto, SF…).

| Rôle | Police retenue (mockups v2) | Fallback |
|---|---|---|
| Display (titres géants, all-caps condensé) | **Anton** | Archivo Black |
| Tension (1 mot en italique dans les titres) | **Instrument Serif Italic** | Georgia italic |
| Corps de texte | **Instrument Sans** (400/500/600) | system-ui |
| Mono / terminal / labels | **Martian Mono** (300/400/600) | Geist Mono, JetBrains Mono |

Signature typographique : titre display condensé géant avec **un mot en italique serif**
(la tension OpenClaw), labels en mono uppercase espacé (`letter-spacing` large).

## 5. Concept du scroll : « The Dam »

Le site se construit comme un barrage. Le scroll ASSEMBLE la page —
chaque section ajoute une couche / une branche. Rien ne "défile" platement.

- Des **particules de sciure/points** (Canvas 2D ou WebGL léger) dérivent en fond
  et s'agglomèrent progressivement en structure au fil du scroll
- La **mascotte webp animée** guide le visiteur : elle travaille dans le hero,
  réapparaît entre les sections (elle pousse une branche, elle dort, elle pointe)
- Transitions de section : wipe/morph, jamais de simple fade-in en cascade

## 6. Structure du site

> **Depuis août 2026, trois pages au lieu d'une one-page :**
> - `index.html` — loader, hero + démo « l'app se remodule », manifeste,
>   harnais (la branche 06 promue en section dédiée), import (« tu viens
>   d'ailleurs »), footer
> - `barrage.html` — « ce qu'il embarque » : le barrage 01–07, le forecast
>   (section `#forecast`), l'installation (`#install`), footer
> - `docs.html` — la documentation (voir section 10)
>
> Le code est passé de HTML monolithiques à des modules partagés :
> `css/{base,nav,hero,dam,harness,forecast,docs,footer,motion}.css` et
> `js/{theme,dust,demo,dam,chart,install}.js`.

### 6.0 Loader
Compteur 0→100 en mono, petite animation mascotte, puis reveal du hero (clip-path).

### 6.1 Hero
- Titre display géant : « TOUT EST DÉJÀ LÀ — *et tu peux tout* REPRENDRE »
- Sous-titre sobre, 2 CTA : `Télécharger` (accent plein) / `Voir comment →` (outline)
- **Démo « l'app se remodule »** : wireframe de l'UI qui se réorganise tout
  seul en 3 états (installation → tu ajoutes ton panneau → tu remplaces ce que
  tu veux) — la thèse du site jouée avant d'être écrite
- Fond : particules de sciure (Canvas 2D) qui s'assemblent en barrage au scroll

### 6.2 Manifeste (éditorial, microsoft.ai)
2-3 phrases grandes, posées. « Tes LLM locaux. Tes providers cloud. Tes clés
dans un vault chiffré. Un seul atelier. » Beaucoup d'espace, aucune card.

### 6.3 Le barrage (features)
Pas une grille de cards : une **construction verticale numérotée** (01, 02, 03…
façon Aker) où chaque feature est une branche ajoutée au barrage au scroll :
- 01 — Tous tes modèles (Ollama géré + 10 providers API + comptes web OpenAI/Grok/Kimi)
- 02 — L'agent au travail (outils, sous-agents, Plan mode, mémoire persistante)
- 03 — Il bosse pendant que tu dors (wakeups, gateway Telegram/Slack/Discord)
- 04 — Ton code, ton Git (branches, worktrees, commits, terminal PTY intégré)
- 05 — Un vrai navigateur embarqué (Chromium CEF, macOS/Windows, pilotable par l'agent)
- 06 — Un harnais à ta main (hôte Node séparé, SDK `@beaver/sdk` : remplace outils,
  system prompt, features et UI — panneaux, boutons, vues — sans forker Beaver)
- 07 — Tes secrets restent secrets (vault XChaCha20, clés jamais exposées)
- Argument d'adoption bonus : **import depuis Claude Code, Codex, OpenClaw, Kimi Code…**
  (« tu migres en 2 minutes » — peut vivre dans la section install)

La branche 06 mérite probablement **sa propre section dédiée** après le barrage :
« Le harnais » — un avant/après visuel montrant une UI Beaver standard vs une UI
remodulée par extensions, avec un mini-bloc de code d'exemple d'extension.

### 6.4 Forecast — la signature
« Ce que les autres agents n'ont pas. » Un vrai graphique de séries temporelles
animé (tracé qui se dessine, intervalle de confiance), modèles listés en mono
(Chronos-Bolt, TimesFM, MOIRAI 2.0, Toto 2.0, FlowState, TabPFN-TS, TiRex,
Sundial, TimeGPT-2 ☁…). C'est LA section mémorable.
Forecast V2 : backtests glissants, métriques MASE/sMAPE/MAE/couverture, anomalies,
décomposition, ensembles pondérés, 7 vues, exports CSV→PDF.
Vit dans `barrage.html#forecast` (la page dédiée a été fusionnée).

### 6.5 Install
Bloc terminal unique : `curl -fsSL https://beaver.dev/install.sh | bash` qui
se tape tout seul à l'arrivée dans le viewport + bouton Copier.
~~Onglets macOS/Linux | Windows~~ → simplifié en une ligne (août 2026) ;
à revalider pour Windows (PowerShell ?). Vit dans `barrage.html#install`.

### 6.6 Stack & chiffres
Bandeau mono : `Rust · Tauri 2 · React 19 · 3 OS · AGPL-3.0`.
Éventuellement stars GitHub plus tard.

### 6.7 Footer
CTA typographique géant « CONSTRUIS *ton* BARRAGE. PUIS DÉMONTE LE NÔTRE. »
façon Glenn Catteeuw. Liens : GitHub, docs, changelog. Mascotte qui dort.

## 7. Inventaire des animations

| Élément | Technique | Priorité |
|---|---|---|
| Loader compteur + reveal | GSAP + clip-path | P0 |
| Particules sciure/barrage au scroll | Canvas 2D ou three.js léger + ScrollTrigger | P0 |
| Titres qui se révèlent (split text, masques) | GSAP SplitText / custom | P0 |
| Terminal qui se tape tout seul | JS typing + curseur clignotant | P0 |
| Mascotte entre sections (scroll-driven) | webp + ScrollTrigger scrub | P1 |
| Graphique forecast qui se dessine | SVG path animation | P1 |
| Transitions de sections (wipe/morph) | GSAP ScrollTrigger pin | P1 |
| Hover magnétique sur CTA | JS léger | P2 |
| Grain / scanlines | CSS overlay | P2 |

Contraintes : 60fps, `prefers-reduced-motion` respecté, pas de lib WebGL lourde
si Canvas 2D suffit. Le site doit rester rapide (SEO/GEO).

## 8. Mascotte

- Asset existant : webp animé réagissant à l'agent (dans l'app)
- Sur le site : même langage — elle travaille, attend, dort, pointe
- Elle ne parle pas (pas de bulles), elle AGIT
- Déclinaisons nécessaires : idle, build, sleep, point — à produire si manquantes

## 9. SEO / GEO (cadrage rapide)

- Site statique ou SSG, HTML sémantique complet (pas de contenu 100% JS)
- Meta/OG propres, sitemap, `llms.txt` pour le GEO
- Contenu textuel réel dans le HTML malgré les animations (accessibilité + crawlers)
- Performance : Core Web Vitals verts dès la v1

## 10. Documentation

Style : la DA Beaver en plus sobre. Layout 3 colonnes (sidebar nav sticky /
contenu ~720px / table des matières de page). Anton réduite pour les titres,
mono pour le code, blocs terminal comme le hero, callouts info/warning/security
à liseré ambré, mascotte en mode `explore-book`. Dark + light, recherche Ctrl+K,
contenu 100% dans le HTML (SSG) pour le SEO/GEO.

Arborescence (alignée sur l'état du 28 juillet 2026) :
- **Démarrage** : Installation · Premier lancement (onboarding + Ollama) ·
  Providers, comptes web & OAuth · **Importer depuis Claude Code / Codex / OpenClaw…**
- **Agent** : Modes de permission · Plan mode · Session branching · Sous-agents ·
  Mémoire & personnalité · Skills & commandes `/`
- **Outils** : Fichiers & terminal · Recherche web (Brave/Exa/Firecrawl/SearXNG) ·
  Navigateur intégré (macOS/Windows) · MCP & Gateway (Telegram/Slack/Discord)
- **Forecast** : Workspace · Données & préparation · Modèles locaux ·
  Évaluation & backtests · Scénarios & exports
- **Extensions** : Socle & centre d'extensions · Plugins Office officiels ·
  Écrire une extension (SDK `@beaver/sdk`) · Confiance & sécurité
- **Référence** : Wakeups · Sécurité & vault XChaCha20 · Raccourcis clavier ·
  Stockage local · Thèmes & langues
- **Guides** (SEO) : Premier forecast · Automatiser avec les wakeups ·
  Connecter Telegram · Générer un DOCX/PDF avec l'agent

Stack recommandée : Astro (statique, rapide, îles JS, i18n possible).

## 11. Questions ouvertes

1. Nom de domaine ? → `beaver.dev` est utilisé dans les mockups
   (commande d'install) — **à confirmer et réserver** avant publication
2. Langue du site : EN uniquement, ou FR + EN ?
3. ~~Screenshots réels de l'app~~ → **décision 2026-08 : images uniquement, pas de vidéo pour le moment** (section 6.3 montrera des captures fixes)
4. Déclinaisons de la mascotte : qui les produit ?
5. Stack du site : Astro (reco : rapide, SEO-friendly, îles JS) ou React/Vite classique ?

### Notes de session (2026-08)
- Le chart forecast de l'app est passé en v2 : courbes lissées (monotone), aire en
  dégradé sous l'historique, zone de forecast teintée, bande de confiance subtile,
  fan chart q10/q90 + saisonnalité, drag pan / wheel zoom. Le mockup forecast
  a été aligné sur ce langage visuel (dans `barrage.html#forecast`).

### Notes de session (2026-08-08 — refonte v2)
- **Pitch pivoté** : le harnais (« tout reprendre, pas un fork ») devient la
  thèse du site et le titre du hero. La métaphore castor reste visuelle.
- **Trois pages** : `index.html` (hero + démo remodulage, manifeste, harnais,
  import), `barrage.html` (barrage 01–07, forecast, install), `docs.html`
  (vraie page « Modes de permission » avec tableau, callouts, sidebar 6 groupes).
- **Code modularisé** : `css/` × 9, `js/` × 6. `forecast.html` monolithe
  supprimé (section déplacée dans `barrage.html`).
- **Nouveautés visuelles** : loader compteur (une fois par session), sciure
  Canvas 2D qui s'assemble en barrage au scroll (`dust.js`), démo hero qui
  remodule l'UI en 3 états (`demo.js`), footer « PUIS DÉMONTE LE NÔTRE. ».
- **`prefers-reduced-motion` respecté partout** ; params de debug conservés :
  `?theme=light`, `?state=all`, `?skip=loader`.
- Modèles forecast listés : Chronos(-Bolt), TimesFM, MOIRAI 2.0, Toto 2.0,
  FlowState, TabPFN-TS, TiRex, Sundial, TimeGPT-2 ☁.
- Install simplifiée en une ligne curl (`beaver.dev`) — onglets par OS abandonnés.
- Previews v2 archivées dans `mockup/preview-site-beaver/` (suffixe `-v2`).
- L'app a maintenant plusieurs thèmes (dark, light, astral-mist, emerald-night,
  cobalt-frost) — le site reste sur dark + light pour la v1.
- Source de vérité produit lue : `Beaver/docs/produit/fonctionnalites-embarquees.md`
  (28 juillet 2026), copiée dans `docs/beaver-site/ref/`. Nouveautés intégrées au
  site : navigateur Chromium intégré (macOS/Windows), système d'extensions +
  4 plugins Office officiels, mémoire persistante, comptes web (OpenAI/Grok/Kimi),
  import depuis d'autres agents, Forecast V2 (backtests, métriques, exports).
  Le barrage passe de 5 à 7 branches.
