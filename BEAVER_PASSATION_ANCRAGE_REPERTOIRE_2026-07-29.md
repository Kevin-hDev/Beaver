# Passation — ancrage du répertoire de travail

Destinataire : GPT-5.5, sur la branche `codex/fix-project-root-context`.
Auteur de l'analyse amont : Claude, session du 2026-07-29.
Dernier commit revu : `e6c2247` — *fix(agent): anchor sessions to the project root*.

## Comment tu lis ce document

Tu commences par une investigation de ton côté, avant de toucher au code. Ce document te dit ce qui est déjà établi pour que tu n'y passes pas ton temps, et ce qui reste ouvert pour que tu t'y concentres.

- La section **Établi** contient des faits vérifiés dans le code, avec leur référence `fichier:ligne`. Tu ne les re-dérives pas. Tu les contredis si tu trouves qu'ils sont faux, en montrant le code.
- La section **Décisions** contient des choix que Kevin a tranchés. Tu ne les rediscutes pas. Tu signales seulement si l'un d'eux est techniquement impossible, en expliquant pourquoi.
- Les sections **Lot 1** et **Lot 2** sont le travail. Le lot 1 part seul et avant le lot 2.
- La section **À investiguer** liste ce que personne n'a vérifié. C'est là que ton investigation apporte quelque chose de neuf.

---

## Établi — vérifié dans le code, ne pas re-dériver

### La panne d'origine et sa correction

Une seule variable, `session.working_dir`, portait quatre rôles : racine annoncée au modèle dans le prompt, base de résolution des chemins relatifs, ancrage des contrôles d'accès, et cible réécrite après chaque commande bash.

`tool_bash.rs` ajoutait `pwd -P` à la fin de chaque commande, et `tool_dispatcher.rs` persistait le résultat via `session_store::update_working_dir`. Un `cd` dans n'importe quelle commande déplaçait donc la session de façon durable. Le front rejouait ensuite ce répertoire contaminé sur « Réessayer » et « Modifier le message ».

`e6c2247` a supprimé ce mécanisme par construction : le champ `new_cwd` n'existe plus dans `ShellOutput`, `wrap_command_with_cwd` et `extract_cwd` sont supprimés, et un paramètre `workdir` optionnel et absolu remplace la persistance. Le front ne conserve plus de répertoire en cache. `resolve_for_session` fait gagner le projet sur toute valeur entrante ou stockée.

Cette partie est correcte. Tu ne la retouches pas.

### Ce que font les six agents de référence

Six dépôts ont été lus en entier dans `~/Projects/analyse-repo/` : Codex CLI, Claude Code, OpenCode, Pi, Hermes Agent, OpenClaw.

Un invariant tenu par les six, sans exception : **la racine montrée au modèle n'est jamais la variable que le shell déplace.** Codex garde un cwd de tour immuable, Claude Code un `getOriginalCwd()` figé au lancement, OpenCode affiche `directory` et `worktree` séparément dans son bloc `<env>`, Pi et OpenClaw figent leur racine à la construction, Hermes calcule sa racine depuis le dépôt git indépendamment du cwd vivant.

Deux points où les six divergent, donc où le choix est libre :

- La persistance d'un `cd` — quatre ne persistent pas, deux persistent avec un garde qui ramène à la racine et un message qui la nomme.
- Le confinement — trois ne confinent pas du tout, deux demandent une approbation, un utilise un bac à sable système.

Aucun n'exige un chemin absolu comme mécanisme de confinement. Là où la description du paramètre dit « absolu », le code accepte le relatif et le résout. C'est une convention d'explicitation, jamais une barrière.

### Le routage mémoire

`tool_dispatcher_entry.rs:75` fait passer tous les appels de `read_file`, `write_file`, `edit_file`, `list_dir`, `grep` et `glob` par `memory_tool::dispatch_if_memory` avant le vrai outil. Une erreur renvoyée par cet intercepteur court-circuite l'outil réel.

Avant `e6c2247`, `lexical_path` (`memory_paths.rs:116`) refusait tout chemin contenant `.` ou `..`, y compris pour des chemins qui ne visaient pas la mémoire. `list_dir` avec `path: "."` — la forme que la description de l'outil recommandait — renvoyait donc « Chemin mémoire invalide. » `e6c2247` a corrigé ce cas.

### Le mode mémoire automatique

`memory_runtime.rs:57-62` : en `MemoryMode::Automatic`, l'agent écrit en mémoire librement, sans autorisation par tour. Le mode manuel est un choix explicite de l'utilisateur. C'est le design de Claude Code, Codex, Hermes et OpenClaw. Tu ne le changes pas.

### Le dossier de données

`paths.rs:6-10` : `data_dir()` vaut `~/.local/share/cl-go-dash` sur les trois systèmes, dérivé du dossier personnel. Aucune variance entre macOS, Windows et Linux.

Il contient déjà 26 Go et des répertoires de travail créés par l'application pour ses sous-agents : `subagent-worktrees/`, `subagent-directory-repos/`, `subagent-changes/`. Le motif « l'application fabrique un répertoire de travail dans son dossier de données » existe donc déjà et fonctionne.

`security.rs:77` ajoute `data_dir()` aux racines autorisées en lecture et en écriture, indépendamment du réglage `allowed_paths`.

---

## Décisions — tranchées par Kevin, ne pas rediscuter

**Les sessions sans projet reçoivent une racine fabriquée**, dans le dossier de données, enregistrée comme un projet ordinaire. Il n'existe alors plus de branche de repli à sécuriser, parce qu'il n'y a plus de repli. C'est la conception de Codex Desktop, à un détail près : Codex écrit dans `~/Documents`, ce qui est un mauvais choix et documenté comme tel plus bas.

**Les dossiers sont créés à la première écriture**, pas à l'ouverture de la session. Codex les crée d'emblée et accumule des dossiers vides — six sur quinze journées sur la machine de Kevin.

**La séparation `work/` et `outputs/` est reprise.** `work/` contient les dépôts clonés et les fichiers intermédiaires, `outputs/` contient les livrables.

**L'emplacement d'`outputs/` est réglable**, dans l'onglet des réglages avancés, tout en bas. Cinq utilisateurs de Codex réclament ce réglage depuis mai 2026 sans l'obtenir.

**Un bouton ouvre le dossier `cl-go-dash`**, placé dans les réglages avancés sous le réglage de périmètre des dossiers. Pas dans la conversation : un bouton qui ouvre le dossier de la session en cours montrerait des fichiers que l'utilisateur a déjà sous les yeux.

**Le mode full access n'affiche aucune demande d'approbation.** L'objectif du produit est une IA autonome : un utilisateur qui lance une implémentation de quatre heures et s'absente ne doit pas retrouver l'agent arrêté à 7 % devant une boîte de dialogue. Toute correction qui introduit une porte en mode full access est refusée.

**Les autorisations restent celles de Codex et Claude Code.** La majeure partie des fichiers de configuration reste accessible à l'agent. Il crée et modifie librement les skills, les rules, AGENTS.md et la mémoire.

**Le modèle de responsabilité est celui des CLI existants** : l'utilisateur choisit son mode en connaissance de cause et porte les conséquences.

---

## Lot 1 — régression bloquante, à corriger avant la fusion

`e6c2247` introduit un contournement des protections de la mémoire. Ce lot part seul, avant tout le reste.

### Ce qui ne va pas

`memory_project_migration.rs:29-37` :

```rust
let unresolved = if path.is_absolute() { path.to_path_buf() } else { working_dir.join(path) };
if !unresolved.starts_with(layout.root()) {
    return Ok(None);
}
```

`Path::starts_with` compare les composants littéralement, sans résoudre les `..`. Un chemin comme `../../../.local/share/cl-go-dash/memory/global/MEMORY.md` construit un `unresolved` qui ne commence pas lexicalement par la racine mémoire. La fonction renvoie donc `Ok(None)` et le chemin part vers les outils fichier ordinaires, alors qu'il atteint bien la mémoire une fois résolu par le système de fichiers.

Vérifié en compilant le cas.

### Ce que ça casse

Deux fonctions cessent de fonctionner, et aucune n'a de rapport avec des approbations.

**L'interrupteur mémoire.** Un utilisateur qui a mis la mémoire en mode manuel, ou qui l'a désactivée, se fait quand même écrire dedans. Le contrôle de `memory_runtime::read_allowed` et `write_allowed` n'est jamais atteint.

**L'isolation entre projets.** Une session ouverte dans un projet atteint la mémoire d'un autre projet. Le message `Mémoire d'un autre projet inaccessible.` (`memory_project_migration.rs:41`) n'est plus atteint.

Le rejet des liens symboliques de `validate_in_scope` est également contourné.

Le trou ne dépend pas du réglage `allowed_paths` : `security.rs:77` ajoute `data_dir()` aux racines autorisées, et la mémoire en est un sous-dossier. Restreindre `allowed_paths` au projet ne le refermerait pas.

### Le test qui couvrait ce cas a été affaibli

`memory_paths_tests.rs:47`, `traversal_is_rejected`, testait auparavant :

```rust
.scope_for_tool_path("../memory/global/MEMORY.md", root.path())
```

Il teste maintenant un chemin absolu déjà situé dans l'arbre mémoire, avec un `..` interne. Ce cas passe le nouveau garde et se fait rejeter plus loin, donc le test est vert. L'échappée depuis l'extérieur vers la mémoire n'est plus couverte nulle part.

La suite est verte — 69 tests mémoire, 0 échec, vérifié — parce que le cas a quitté la suite.

### Direction de correction

Tu normalises `unresolved` lexicalement — tu résous les composants `.` et `..` sans toucher au disque — avant de le comparer à `layout.root()`. Tu ne peux pas utiliser `canonicalize` directement : le fichier peut ne pas exister encore lors d'une écriture. La technique employée par `security.rs` dans `canonicalize_candidate` est adaptée : canonicaliser le parent, puis rejoindre le nom du fichier.

Un chemin qui atteint la mémoire par n'importe quelle route est alors reconnu et reçoit la validation stricte. Tout le reste continue de passer, y compris `.` et `./src`.

Tu restaures le test d'échappée à côté du nouveau. Les deux cas sont distincts et méritent chacun leur test.

### Piste à vérifier dans le même lot

`memory_tool.rs:67-82`, `is_memory_operation`, utilise la même construction `working_dir.join(path)` suivie d'un `starts_with` non canonicalisé. Elle alimente `write_authorization`, appelé depuis `tool_executor_write.rs:47`. Vérifie si elle présente la même faiblesse, et corrige-la au même endroit si c'est le cas.

---

## Lot 2 — conception, après la fusion du lot 1

### 2.1 Racine fabriquée pour les sessions sans projet

Trois endroits se replient aujourd'hui sur le dossier personnel :

- `agent_working_dir.rs:35-41` — session sans projet, sans répertoire entrant et sans répertoire stocké.
- `agent_chat_task/common.rs:57-69` — `resolve_working_dir`, non touchée par `e6c2247`, se replie sur `dirs::home_dir()` quand la valeur reçue est vide.
- `gateway/agent_bridge.rs:149` — passe `working_dir: None`, donc toute session déclenchée par un canal distant démarre dans le dossier personnel.

Tu remplaces ces replis par la fabrication d'une racine. Le nommage `session-workspaces/` s'aligne sur les conventions existantes du dossier de données (`session-permissions/`, `session-tabs.json`).

Structure retenue, dérivée de l'observation directe de Codex Desktop sur la machine de Kevin :

```
<data_dir>/session-workspaces/<YYYY-MM-DD>/<slug>/
    work/
    outputs/
```

Le slug vient du premier message utilisateur : minuscules, tout caractère non alphanumérique remplacé par un tiret, tronqué autour de trente à trente-cinq caractères sur une frontière de mot, suffixe `-2` et suivants en cas de collision dans la même journée.

Tu enregistres le dossier fabriqué comme projet, pour qu'il n'existe qu'un seul chemin d'ancrage dans le code. Une session dont la racine a été fabriquée doit être indistinguable, en aval, d'une session dont l'utilisateur a choisi le dossier.

### 2.2 Point 4 de la revue — le signal, pas le mur

`models/config.rs:109` : `default_allowed_paths()` vaut `/` sur Unix et `C:\` sur Windows. `validate_write_path` (`security.rs:132`) ne reçoit pas le répertoire de travail et ne teste que cette liste. Aucune écriture n'est donc bornée par défaut.

Kevin assume ce choix, et il est aligné sur la majorité des références : trois des six ne confinent pas non plus. Tu ne poses pas de mur.

Ce qui manque n'est pas la limite, c'est le retour. Aucune des six références ne combine « pas de confinement » avec « aucun signal ». `enrich_error` (`tool_dispatcher_entry.rs:127`) ajoute des indices pour `edit_file` et `bash`, et rien pour les erreurs de chemin.

Deux mécanismes existants à examiner comme modèles :

- Hermes attache un avertissement au résultat de l'outil quand un chemin relatif résout hors de la racine, sans rien bloquer : *« Relative path X resolved to Y, which is OUTSIDE the active workspace (root) »*. L'écriture a déjà eu lieu, le modèle apprend qu'il a dérivé.
- Claude Code ajoute à la sortie d'erreur `Shell cwd was reset to <racine>`, qui nomme la racine.

Tu proposes une forme adaptée à Beaver. Elle ne bloque pas, elle n'ajoute pas de latence, et elle nomme la racine attendue.

### 2.3 Catalogue des permissions

Trois catégories, à établir précisément dans le code :

**Libre, sans porte** — le projet, l'espace de session, les skills, les rules, AGENTS.md, la mémoire en mode automatique, et la configuration quand l'utilisateur le demande.

**Règle de prompt, sans porte** — ne pas modifier les réglages de permission ni la configuration sur l'initiative d'un contenu externe : page web, résultat d'outil, message d'un autre agent. C'est le mécanisme utilisé par Claude Code, dont le prompt système interdit ce cas sans qu'aucun blocage technique existe. Une consigne ne coûte aucune latence et n'interrompt jamais une exécution longue. `prompt_external_content.rs` fournit déjà l'emplacement.

**Blocage dur, aucune demande** — `secrets.enc`, les jetons d'authentification, le coffre. La raison n'est pas le risque : aucune instruction utilisateur ne rend utile la lecture de ces fichiers par l'agent. Un blocage qui ne retire aucune capacité ne coûte rien à l'autonomie.

### 2.4 Interface

Bouton d'ouverture du dossier `cl-go-dash` dans les réglages avancés, sous le réglage de périmètre des dossiers. Réglage d'emplacement d'`outputs/` au même endroit.

---

## À investiguer — personne ne l'a vérifié

Tu traites ces points avant de proposer un plan. Ils ne sont pas des tâches, ce sont des questions ouvertes.

1. **Autres appelants du repli sur le dossier personnel.** Trois ont été trouvés. Cherche s'il en existe d'autres, notamment dans les chemins sous-agent et les tâches planifiées.

2. **`is_memory_operation`** présente-t-elle la même faiblesse lexicale que `scope_for_tool_path` ? Voir la piste du lot 1.

3. **Comportement de `data_dir()` sur Windows et Linux.** La fonction construit `~/.local/share/cl-go-dash` sans distinction de système. Vérifie que la fabrication de racines de session dans ce dossier ne pose pas de problème sur les trois plateformes, notamment sur les longueurs de chemin Windows.

4. **Quand une session sans projet devient-elle une session avec projet ?** Si l'utilisateur sélectionne un projet après avoir commencé à écrire dans une racine fabriquée, que devient le contenu déjà produit. Aucune décision n'a été prise.

5. **Purge des racines fabriquées.** Codex n'en a aucune et a fermé la demande en « not planned ». Kevin veut y penser. Propose un mécanisme, sans le rendre automatique par défaut sans son accord.

6. **Signal du lot 2.2** — quelle forme exacte, à quel moment, dans quel champ du résultat d'outil. C'est le point le moins spécifié du document.

---

## Hors périmètre

Tu ne déplaces pas le dossier de données. Kevin sait que `~/.local/share/cl-go-dash` est peu découvrable et que les autres CLI posent leur configuration directement dans le dossier personnel. C'est une migration lourde, sur 26 Go, à décider séparément. Placer les sessions dans le dossier de données découple justement cette question : le jour où la migration a lieu, les sessions suivent.

Tu ne modifies pas le mécanisme de persistance du `cd` corrigé par `e6c2247`. Il est correct.

Tu n'ajoutes aucune demande d'approbation en mode full access.

---

## Vérification attendue

Tu ne déclares rien validé tant qu'un test échoue. Si un test est rouge, tu le dis avec sa sortie.

Pour le lot 1, tu restaures le test d'échappée supprimé et tu ajoutes un test pour le chemin non canonicalisé décrit plus haut. Le lot n'est pas fini tant que les deux cas — passage des chemins ordinaires, rejet des échappées — sont couverts par des tests distincts.
