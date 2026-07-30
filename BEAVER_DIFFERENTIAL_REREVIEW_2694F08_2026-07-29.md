# Contre-revue différentielle — commit 2694f08

## Verdict

Périmètre examiné : `2694f08^..2694f08`, son message de commit et
`git notes show 2694f08`.

Le commit corrige bien les échecs automatiques et cinq défauts de la première
revue. Les deux décisions de conception restantes sont maintenant expliquées
dans la note Git et reflétées dans les fichiers locaux ignorés par Git.

Il reste toutefois un conflit d'autorité de sévérité moyenne dans la nouvelle
règle sur les résultats d'outils. La branche ne devrait pas être fusionnée avant
de l'avoir clarifié.

| Sévérité | Nombre |
|---|---:|
| Critique | 0 |
| Haute | 0 |
| Moyenne | 1 |
| Faible | 1 |

**Risque résiduel : moyen**

## État des sept constats précédents

| Constat précédent | État |
|---|---|
| Clippy : code de production après le module de tests | Corrigé |
| Confirmation des actions locales destructrices | Corrigé et testé |
| Une skill chargée était neutralisée | Corrigé pour `load_skill` |
| Quatre titres au lieu d'un unique `# Rules` | Décision documentée et spécification locale mise à jour |
| Règle `set_formula` dupliquée | Corrigé |
| Ligne vide finale | Corrigé |
| Fichier de tests au-dessus de 200 lignes | Politique locale modifiée et décision enregistrée dans la note |

## Nouveaux constats

### Moyenne — Les réponses directes de l'utilisateur sont classées comme sans autorité

**Fichier principal :**
`src-tauri/src/services/agent_local/prompt_external_content.rs:13`

La nouvelle règle autorise correctement `load_skill`, puis affirme que rien
d'autre arrivant dans un résultat d'outil ne porte d'instructions.

Ce classement est trop large. `ask_user_choice` attend une réponse directe de
l'utilisateur, puis l'insère dans un `ToolResult`
(`tool_interactive.rs:22-30`, `tool_interactive.rs:52-75`). Tous les
`ToolResult` sont ensuite renvoyés au modèle avec le rôle `tool`
(`tool_executor_results.rs:41-55`).

Le même canal transporte les décisions de Plan mode et les consignes internes
qui en découlent (`tool_plan_messages.rs:8-35`).

**Scénario concret :**

1. L'agent appelle `ask_user_choice` car le choix doit changer la suite.
2. L'utilisateur sélectionne « Other » et écrit une consigne personnalisée.
3. Cette consigne revient uniquement dans un message de rôle `tool`.
4. Le prompt dit qu'elle ne porte aucune instruction.

Le résultat dépend alors de la manière dont le modèle arbitre ce conflit : il
peut ignorer la consigne personnalisée, ou traiter de façon incohérente une
approbation de plan. Cela contredit directement la définition
d'`ask_user_choice`, selon laquelle la réponse change l'étape suivante.

**Correction recommandée :** classer selon l'origine réelle, pas seulement
selon le rôle technique du message. Les réponses obtenues par
`ask_user_choice` et les décisions de `planmode` doivent rester des instructions
utilisateur ou des signaux de contrôle fiables. Les résultats de lecture,
commande, web et MCP restent des données non fiables.

Ajouter un test statique couvrant explicitement ces exceptions, puis idéalement
une évaluation avec un vrai modèle pour une réponse personnalisée et les trois
décisions de Plan mode.

### Faible — La section External content de la spécification gelée reste obsolète

**Fichier :** `docs/plans/agent-prompt-restructure-spec.md:346-361`

La partie consacrée à la structure a bien été mise à jour pour les quatre
titres. En revanche, la section 7.6 contient toujours l'ancien texte qui classe
tous les résultats d'outils comme des données et limite les instructions au
prompt système et aux messages utilisateur.

Elle ne décrit donc ni l'exception `load_skill` ajoutée par `2694f08`, ni le
traitement attendu des réponses interactives. Comme ce document sert de
spécification gelée, une future modification peut réintroduire le défaut.

**Correction recommandée :** mettre la section 7.6 en accord avec la règle
d'autorité finale après correction du constat moyen.

## Vérifications exécutées

| Commande | Résultat |
|---|---|
| `cargo clippy --all-targets -- -D warnings` | Réussi |
| `cargo fmt --all -- --check` | Réussi |
| `cargo test` | Réussi — 2 452 tests, 0 échec |
| `git diff --check 2694f08^..2694f08` | Réussi |

Le nombre de tests et les validations annoncés dans le message du commit sont
donc exacts.

## Décision

Le commit `2694f08` améliore nettement la branche et résout les deux anciens
blocages majeurs. Il reste une correction ciblée à faire avant fusion :
préserver explicitement l'autorité des réponses utilisateur et des décisions de
Plan mode qui transitent par un résultat d'outil. La divergence documentaire
peut être corrigée dans le même passage.

## Addendum — validation du commit a8b9198

### Verdict

La correction de `ask_user_choice` est complète. La correction du mode Plan
reste ambiguë : le prompt fait confiance à la décision et au statut, mais pas
explicitement à la prochaine action renvoyée par les outils.

| Sévérité | Nombre |
|---|---:|
| Moyenne | 1 |
| Faible | 2 |

**Recommandation : correction conditionnelle avant fusion.**

### Moyenne — L'exception du mode Plan n'englobe pas explicitement ses consignes

`prompt_external_content.rs:15-17` rend fiables les « decisions and status » de
`planmode` et `exitplanmode`. Or `tool_plan_messages.rs:8-35` renvoie aussi la
prochaine action à exécuter : quitter le mode Plan, continuer à planifier,
republier, ou commencer immédiatement l'implémentation.

Les fiches des outils répètent une partie de ces consignes, ce qui réduit
l'impact. Elles ne décrivent toutefois pas aussi précisément toutes les
bifurcations, notamment la différence entre `continue_planning` et `quit_plan`.
Le contrôleur ajoute ensuite une correction de rôle système si le modèle ne
réagit pas (`agent_loop_plan.rs:27-35`), mais cette réparation consomme un tour
supplémentaire et masque l'ambiguïté.

**Correction :** rends fiable tout ce que les deux outils internes renvoient,
y compris la décision, le statut et la prochaine action explicitement demandée.

### Faible — Le filtrage réel de la règle interactive n'est pas testé

`tool_prompt_filter.rs:7-21` retire les mentions d'un outil désactivé ligne par
ligne. La règle `ask_user_choice` disparaît aujourd'hui parce que sa chaîne
assemblée tient sur une seule ligne. Une reformulation qui introduit un saut de
ligne peut laisser un fragment orphelin sans faire échouer les tests actuels.

**Correction :** teste `filter_system_prompt` avec le vrai
`EXTERNAL_CONTENT`, `ask_user_choice` désactivé, puis vérifie que toute la règle
interactive disparaît tandis que les règles Plan restent présentes.

### Faible — Le test du fallback verrouille une phrase entière

`prompt_external_content.rs:53-57` compare une phrase complète sans vérifier un
comportement supplémentaire. Une reformulation équivalente casserait le test.

**Correction :** vérifie deux fragments courts portant le contrat sémantique,
par exemple « Every other tool result » et « not an instruction », et laisse le
test de filtrage couvrir le comportement réel.

### Confiance et limites

Confiance élevée sur l'analyse statique du prompt, des fiches d'outils, des
messages Plan et du filtre. Aucun test avec un vrai modèle n'a été exécuté pour
mesurer la fréquence concrète du tour de réparation.
