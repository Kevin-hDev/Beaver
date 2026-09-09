# Remplacer un outil de Beaver par le sien

**Emplacement site** — Intégrations › Extensions › Remplacer un outil
**Répond à** — « Comment je remplace un outil de Beaver par le mien, qu'est-ce qui est garanti, et qu'est-ce qui peut casser ? »
**Sources** — `EXTENSIONS.md` (sections « Remplacer un outil natif », « Déclarer l'effet d'un outil », « Ce que Beaver isole — et ce qu'il n'isole pas », « Limites actuelles »), `src-tauri/resources/extension-host/extension-api.mjs`, `src-tauri/resources/extension-host/contract.json`, `src-tauri/src/services/extensions/` (`tool_bridge.rs`, `registry_index.rs`, `runtime_sync.rs`, `runtime_sync_contributions.rs`, `runtime_sync_apply.rs`, `types.rs`, `discovery_inspection.rs`), `src-tauri/src/services/agent_local/` (`tool_availability.rs`, `tool_dispatcher_route.rs`, `tool_dispatcher_entry.rs`, `tool_catalog.rs`, `tool_catalog_filter.rs`, `permission_gate.rs`, `permission_policy.rs`, `tool_plan_guard.rs`, `extension_tool_set_apply.rs`, `extension_session_plugins.rs`, `subagent_tool_profile.rs`), `src-tauri/src/commands/agent_chat_task/tool_policy.rs`, `src/components/extensions/extension-detail.tsx`, `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, sauf les points listés en fin de fichier

---

## Plan de page proposé

1. Ce que « remplacer » veut dire exactement
2. Le prérequis : le niveau avancé
3. Il n'existe aucune liste des outils remplaçables
4. Quand le remplacement s'applique — et quand l'outil d'origine revient
5. Activation, confirmations et mode Plan
6. Ce que cela engage vraiment
7. Diagnostiquer un remplacement
8. Revenir en arrière

---

## Contenu

### Ce que « remplacer » veut dire exactement

Une extension peut prendre la place d'un outil de Beaver. À partir de ce moment, **le modèle ne voit plus qu'un seul outil sous ce nom : celui de l'extension.** Il ne sait pas qu'un remplacement a eu lieu, et il ne peut pas demander l'outil d'origine.

Le mécanisme est une substitution par le nom, pas un ajout :

- Beaver construit la liste des outils du cœur, puis y injecte ceux des extensions. Quand un nom existe déjà, l'outil de l'extension **prend sa place dans la liste**, il n'est pas ajouté à côté (`tool_bridge.rs:25-41`, fonction `merge`). *Vérifié dans le code.*
- L'outil d'origine n'est pas perdu : Beaver le range dans la définition de remplacement, sous une clé interne `_beaverCoreFallback` (`tool_bridge.rs:3` et `:34`). Il sert à revenir en arrière automatiquement dans les cas décrits plus bas. *Vérifié dans le code.*
- Le test qui verrouille ce comportement est explicite : après remplacement, la liste contient **une seule entrée**, celle de l'extension, et l'ancienne est conservée en repli (`tool_bridge.rs:72-103`, test `an_explicit_replacement_wins_without_duplicate_names`). *Vérifié dans le code.*

Un outil ordinaire d'extension, lui, est toujours préfixé par l'identifiant de l'extension — `mon-extension.mon-outil`. Un remplacement est **le seul cas où une extension publie un outil sans préfixe** (`extension-api.mjs:43-48`). *Vérifié dans le code.*

Le code du remplacement s'écrit exactement comme un outil ordinaire : un nom, une description, un schéma de paramètres, une classe d'effet et une fonction d'exécution. Seul le point d'entrée change — `beaver.unstable.registerReplacement(...)` au lieu de `beaver.registerTool(...)` (`EXTENSIONS.md`, section « Remplacer un outil natif » ; `extension-api.mjs:174-179`). *Vérifié dans le code.*

### Le prérequis : le niveau avancé

Le remplacement n'est accessible qu'aux extensions dont le manifeste déclare `"apiLevel": "advanced"`. Le contrôle a lieu **deux fois**, et il faut le dire sur le site parce que les deux échecs ne se ressemblent pas :

| Où | Ce qui se passe si le niveau n'est pas `advanced` | Source |
|---|---|---|
| Dans l'extension, à l'enregistrement | L'appel échoue immédiatement avec l'erreur `advanced_api_required` | `extension-api.mjs:174-178` |
| Dans Beaver, à la réception | **Toutes** les contributions de l'extension sont refusées, pas seulement le remplacement, et Beaver enregistre le diagnostic `advanced_required` | `runtime_sync.rs:132-138`, `runtime_sync_contributions.rs:24-26`, `runtime_sync_apply.rs:86-89` |

Le message affiché à l'utilisateur pour ce diagnostic est : **« Le mode avancé est requis pour remplacer un outil intégré »** (`src/i18n/fr.json`, clé `extensions.diagnostics.codes.advanced_required`). *Vérifié dans le code.*

Le second contrôle est celui qui compte : le premier vit dans le code de l'extension, donc dans un processus que Beaver ne contrôle pas. Le second est en Rust, il s'applique quoi qu'ait fait l'extension, et il rejette le lot entier de ses contributions.

**Le niveau avancé est déclaré instable par le contrat lui-même** : « L'API avancée est instable et peut changer entre deux versions de Beaver » (`EXTENSIONS.md`, section « Limites actuelles »). Cette phrase doit apparaître telle quelle sur le site.

Le niveau avancé n'ouvre par ailleurs aucune porte supplémentaire vers le cœur de Beaver. `beaver.unstable.call(...)` existe pour de futures méthodes déclarées avancées, mais **le contrat actuel n'en publie aucune** : les 12 méthodes exposées sont toutes de niveau `stable` (`contract.json`, section `methods.hostToCore` ; vérifié par lecture du fichier — zéro méthode de niveau autre que `stable`). Tout appel à `unstable.call` est donc rejeté aujourd'hui avec `core_method_unavailable` (`extension-api.mjs:219-225`). *Vérifié dans le code.*

### Il n'existe aucune liste des outils remplaçables

**C'est le point le plus important de la page.**

Beaver **ne tient aucune liste blanche** de points de remplacement. La règle est mécanique :

- si le nom déclaré correspond à un outil du catalogue du moment, la définition de l'extension **le recouvre** ;
- si le nom ne correspond à rien, la définition **devient un nouvel outil, publié sans préfixe** (`tool_bridge.rs:30-38` : quand aucune position ne correspond, la définition est simplement ajoutée à la liste).

`EXTENSIONS.md` qualifie ce comportement en une phrase qu'il faut reprendre : **« Ce comportement est instable et peut changer entre versions. »**

Deux conséquences directes :

1. **Une faute de frappe ne produit aucune erreur.** Elle produit un outil. C'est le piège majeur de cette page, développé plus bas.
2. **Le catalogue de référence est celui de la version installée.** Il est défini dans `tool_catalog.rs:29-85` : 14 outils verrouillés (dont `bash`, `bash_control`, `read_file`, `write_file`, `edit_file`, `list_dir`, `grep`, `glob`, `web_search`, `web_fetch`, `search_mcp_tools`) et 32 outils optionnels. Un outil qui disparaît ou change de nom dans une version suivante transforme silencieusement un remplacement en outil supplémentaire. *Vérifié dans le code.*

Une phrase de `EXTENSIONS.md` va dans l'autre sens et parle d'« un outil natif prévu pour être remplaçable » (section « Ce qu'une extension peut faire », et tableau « Choisir rapidement » : « Remplacer un outil natif explicitement remplaçable »). **Le code ne connaît pas cette notion** : rien dans `runtime_sync.rs`, `registry_index.rs` ou `tool_bridge.rs` ne distingue un outil remplaçable d'un autre. À signaler en « Points à confirmer » ; ne pas reprendre le mot « explicitement remplaçable » sur le site.

### Quand le remplacement s'applique — et quand l'outil d'origine revient

Un remplacement enregistré n'est pas un remplacement permanent. Beaver revient à l'outil d'origine dans plusieurs situations, sans le dire au modèle. Le tableau des situations est en section « Tableaux ».

Les deux conditions à retenir :

- **L'extension doit être activée et approuvée.** L'index des outils d'extension ne retient que les fiches à la fois activées et approuvées (`registry_index.rs:84-99`, filtre `record.enabled && record.trusted`). Une extension désactivée disparaît de l'index, et l'outil d'origine reprend sa place. *Vérifié dans le code.*
- **L'extension doit être active dans la conversation en cours.** Beaver ne charge pas toutes les extensions dans toutes les conversations : il choisit celles qui tiennent dans le budget d'outils du modèle (`extension_session_plugins.rs:48-57`, `extension_tool_selection.rs:48-90`). Quand l'extension n'est pas retenue, c'est l'outil d'origine qui est envoyé au modèle (`extension_tool_set_apply.rs:63-77`, branche `Some(_) => core_fallback(tool)`). *Vérifié dans le code.*

Un détail favorable au remplacement mérite d'être dit : **un remplacement ne consomme pas de place dans le budget d'outils**. Le décompte exclut les définitions qui portent un repli (`extension_tool_set_apply.rs:30-36`). Une extension dont tous les outils sont des remplacements pèse donc zéro dans le budget. *Vérifié dans le code.*

### Activation, confirmations et mode Plan

Trois règles, indépendantes les unes des autres.

**1. Le remplacement obéit au réglage d'activation de l'outil d'origine.**

Si l'utilisateur a désactivé l'outil dans les réglages, le remplacement est désactivé avec lui. La règle tient en une ligne de code : un outil est disponible s'il est activé dans les réglages, **ou** s'il s'agit d'un outil d'extension qui n'est pas un remplacement (`tool_availability.rs:1-7`). Deux tests l'énoncent explicitement : `replacement_respects_a_disabled_core_tool_setting` et `a_new_extension_tool_does_not_need_a_core_setting` (`tool_availability.rs:13-21`). *Vérifié dans le code.*

C'est cohérent : désactiver `web_search` doit couper la recherche Web, pas la déléguer à quelqu'un d'autre.

**2. Les confirmations viennent de la classe d'effet déclarée par l'extension, pas de l'outil remplacé.**

Dès qu'un outil est connu de l'index des extensions, Beaver décide de demander ou non une confirmation **uniquement** d'après sa classe d'effet, et court-circuite entièrement la logique de l'outil d'origine (`permission_gate.rs:83-89` : le test sur l'index précède le `match` sur les noms d'outils natifs). *Vérifié dans le code.*

La correspondance effet → comportement est dans `permission_policy.rs:13-41`. Elle est reprise en section « Tableaux ».

**3. En mode Plan, c'est aussi la classe d'effet qui décide.**

Le mode Plan autorise une liste fermée d'outils de lecture. Pour un outil d'extension, cette liste est ignorée : Beaver regarde d'abord l'index des extensions et applique la politique de l'effet (`tool_plan_guard.rs:34-48`). Seule la classe `read-only` est autorisée en mode Plan ; un test balaie toutes les classes pour le vérifier (`tool_plan_guard.rs:89-98`, test `only_read_only_extensions_are_allowed_in_plan_mode`). *Vérifié dans le code.*

Conséquence dans les deux sens :

- remplacer un outil autorisé en mode Plan (`read_file`, `grep`, `web_search`…) par un outil déclaré `local-write` **le retire du mode Plan** ;
- remplacer un outil interdit en mode Plan par un outil déclaré `read-only` **l'y fait entrer**.

La page dédiée au mode Plan est actuellement gelée (`_geles/plan-mode.md.gele`) ; ces deux phrases doivent donc vivre ici.

### Ce que cela engage vraiment

Cette section n'est pas une précaution de rédaction. Elle décrit ce que le code fait.

Une extension est **du code local de confiance, exécuté avec les droits du compte utilisateur. Ce n'est pas une sandbox** (`EXTENSIONS.md`, section « À retenir avant de commencer »). Le tableau d'isolation du même fichier le formule ainsi : « Accès complet à Node.js — l'extension peut toujours lire des fichiers, lancer des processus et utiliser le réseau ».

Remplacer `bash` ou la recherche Web ajoute deux choses à ce constat :

- **L'outil remplaçant voit tout ce que l'agent lui passe.** Il reçoit les arguments que le modèle a construits — une commande shell complète, une requête de recherche, un chemin de fichier — ainsi que le dossier de travail de la conversation. Ce qu'il en fait, et ce qu'il renvoie au modèle à la place du vrai résultat, dépend entièrement de son code.
- **Les gardes propres à l'outil d'origine ne s'appliquent plus.** Le code le dit à deux endroits :
  - la garde `bash` de Beaver, qui distingue une commande sûre d'une commande à confirmer, n'est jamais atteinte pour un outil d'extension : le test sur l'index passe avant (`permission_gate.rs:83-89`, la branche `"bash" => !permission_bash::is_safe(cmd)` est en aval) ;
  - un commentaire du même fichier acte le cas réseau : « `external-read` réutilise la décision et le dialogue de `web_fetch`, jamais son filtre anti-SSRF : le code Node approuvé garde son accès réseau direct » (`permission_gate.rs:85-86`). *Vérifié dans le code.*

Ces deux faits vont dans des encadrés, pas dans des notes de bas de page.

### Diagnostiquer un remplacement

**Dans l'application.** La fiche de l'extension (Réglages → Extensions → la fiche → « Contributions enregistrées ») liste chaque outil avec son nom, sa classe d'effet traduite, et — quand c'est un remplacement — l'étiquette **« Remplace un outil intégré »** (`src/components/extensions/extension-detail.tsx:145-160` ; libellé `extensions.detail.replacesCore` dans `src/i18n/fr.json`). C'est le seul endroit où l'information est affichée telle quelle. *Vérifié dans le code ; apparence non vérifiée à l'écran.*

**Depuis l'agent.** Deux outils existent, tous deux verrouillés et donc toujours présents (`tool_catalog.rs:41-48`) :

- `list_extensions` — liste les extensions connues ;
- `inspect_extensions` — détaille une ou plusieurs extensions par identifiant, avec leurs outils, skills et ressources, et un état par extension.

Ils sont autorisés en mode Plan (`tool_plan_guard.rs:3-25`). *Vérifié dans le code.*

**Limite à annoncer** : `inspect_extensions` renvoie pour chaque outil son identifiant, son nom et un résumé — **il n'indique ni la classe d'effet, ni le fait qu'il remplace un outil de Beaver** (`discovery_inspection.rs:28-33` et `:47-55`). Le seul indice disponible pour l'agent est le nom lui-même : un outil d'extension dont le nom ne porte pas le préfixe de son extension est un remplacement. *Vérifié dans le code.*

### Revenir en arrière

Le retour en arrière est immédiat et ne demande aucune manipulation de fichier :

1. **Désactiver l'extension** dans sa fiche. Elle sort de l'index, son processus et ses permissions de session sont révoqués (`registry_index.rs:84-99` ; `EXTENSIONS.md`, section « En cas de problème », étape 5). L'outil d'origine reprend sa place à la conversation suivante.
2. **Retirer l'extension** si la confiance est rompue. Pour une source Git ou npm, Beaver ne retire que sa propre copie ; les fichiers locaux de l'utilisateur restent intacts (`EXTENSIONS.md`, tableau « Ajouter l'extension à Beaver »).
3. **Désactiver l'outil d'origine dans les réglages** coupe aussi le remplacement, par la règle 1 de la section précédente. C'est utile quand on veut suspendre l'outil sans toucher à l'extension.
4. Si l'interface elle-même est cassée par le module avancé de l'extension, Beaver se lance sans interface tierce en maintenant **Maj** au démarrage, ou avec l'argument exact `--safe-mode` (`EXTENSIONS.md`, section « En cas de problème »). *Issu d'EXTENSIONS.md, non recoupé avec le code.*

Un point de sécurité qui ne se règle pas en désactivant : si l'extension a demandé un secret, « désactiver ou supprimer une extension ne retire pas une copie qu'elle aurait déjà obtenue : pour invalider cette copie, révoquez l'accès auprès du fournisseur concerné » (`EXTENSIONS.md`, section « Isolation et confiance »).

---

## Tableaux

### Ce que le modèle voit selon la situation

| Situation | Ce que le modèle reçoit | Source |
|---|---|---|
| Extension activée, approuvée et retenue dans la conversation | L'outil de l'extension, sous le nom d'origine | `extension_tool_set_apply.rs:63-77` |
| Extension désactivée ou non approuvée | L'outil d'origine de Beaver | `registry_index.rs:84-99` |
| Extension non retenue dans la conversation (budget d'outils du modèle) | L'outil d'origine, restauré depuis le repli | `extension_tool_set_apply.rs:71` |
| L'outil d'origine est désactivé dans les réglages | Aucun des deux | `tool_availability.rs:1-7`, `tool_catalog_filter.rs:9-22` |
| Mode Chat | L'outil d'origine ; le mode Chat n'expose que `web_search` et `web_fetch` et ignore les extensions | `tool_definitions_chat.rs:3-4`, `tool_dispatcher_entry.rs:80`, `tool_dispatcher_route.rs:1-13` |
| Route qui refuse les extensions (profils OpenRouter/Groq) | L'outil d'origine, restauré depuis le repli, avec un avis affiché | `tool_policy.rs:42-58` et `:120-133` |
| Registre d'extensions indisponible | Uniquement les outils natifs encore admissibles, avec un avertissement traduit | `extension_tool_set_degraded.rs:13-47` |

### Classes d'effet : ce que chacune déclenche

Valeurs vérifiées dans `permission_policy.rs:13-41` ; libellés français relevés dans `src/i18n/fr.json`, clés `extensions.effects.*`.

| Valeur à déclarer | Libellé affiché | Confirmation demandée | Autorisé en mode Plan | Décision mémorisable pour la conversation |
|---|---|---|---|---|
| `read-only` | Lecture seule | Non | **Oui** | Non |
| `external-read` | Lecture externe | Oui | Non | Oui |
| `local-write` | Écriture locale | Oui | Non | Oui |
| `external-write` | Écriture externe | Oui | Non | Oui |
| `process` | Processus | Oui | Non | Non |
| `secret` | Accès aux secrets | Oui | Non | Non |
| `unknown` | Effet inconnu | Oui | Non | Non |

Un effet absent, mal orthographié ou inconnu n'est pas refusé : il devient `unknown`, la classe la plus restrictive. Le processus qui héberge l'extension normalise déjà la valeur (`extension-api.mjs:54`), et Rust la revalide de son côté (`extension_contract_effect.rs:30-46`, toute valeur non reconnue tombe sur `Unknown`). *Vérifié dans le code.*

### Les deux contrôles du niveau avancé

| Contrôle | Portée du refus | Message ou code |
|---|---|---|
| Dans l'extension (`extension-api.mjs:174-178`) | L'appel `registerReplacement` échoue | Erreur `advanced_api_required` |
| Dans Beaver (`runtime_sync.rs:132-138`) | **Toutes** les contributions de l'extension sont refusées | Diagnostic `advanced_required` → « Le mode avancé est requis pour remplacer un outil intégré » |

---

## Encadrés

**Encadré 1 — Une fonctionnalité déclarée instable.**
Le remplacement repose sur l'API avancée, que le contrat de Beaver décrit lui-même comme instable : « L'API avancée est instable et peut changer entre deux versions de Beaver. » Le comportement en l'absence de correspondance de nom porte la même mention : « Ce comportement est instable et peut changer entre versions. » Une extension qui remplace un outil peut donc cesser de fonctionner, ou changer de comportement, à la mise à jour suivante de Beaver — sans que cela constitue une régression. Écrivez le mot « instable » sur la page ; ne le remplacez pas par « avancé » ou « expérimental ».

**Encadré 2 — Remplacer `bash` ou la recherche Web est l'acte le plus engageant de l'application.**
Une extension n'est pas isolée : c'est du code exécuté avec les droits du compte utilisateur, avec l'accès complet à Node.js — fichiers, réseau, processus. Quand cette extension remplace `bash`, elle reçoit chaque commande que l'agent voulait exécuter. Quand elle remplace `web_search`, elle reçoit chaque requête et décide de ce que le modèle lira en retour. Elle peut exécuter autre chose, envoyer ailleurs, ou renvoyer un résultat fabriqué : le modèle n'a aucun moyen de s'en apercevoir, et l'utilisateur non plus. N'installez un remplacement que si vous avez lu le code de l'extension, ses dépendances, et que vous acceptez de refaire cette lecture à chaque mise à jour.

**Encadré 3 — Les gardes de l'outil d'origine ne suivent pas.**
Un remplacement n'hérite d'aucune des protections écrites pour l'outil qu'il recouvre. Deux exemples vérifiés dans le code : un remplacement de `bash` ne passe plus par l'analyse qui distingue une commande sûre d'une commande à confirmer ; un remplacement de la recherche Web déclaré `external-read` réutilise le dialogue de confirmation de `web_fetch` **mais pas son filtre anti-SSRF** — le code de l'extension conserve un accès réseau direct. Ce qui protège l'utilisateur après un remplacement, c'est uniquement la classe d'effet que l'extension a déclarée elle-même.

**Encadré 4 — Un remplacement déclaré `read-only` ne demande jamais rien.**
La classe `read-only` est la seule sans confirmation, et la seule autorisée en mode Plan. Rien n'empêche une extension de déclarer `read-only` sur un outil qui remplace `bash`. Le résultat : un outil tiers qui exécute ce qu'il veut, sans dialogue, y compris pendant une phase de planification censée ne rien modifier. La classe d'effet est une déclaration de l'auteur, **pas une vérification de Beaver**. C'est la raison pour laquelle la lecture du code de l'extension n'est pas facultative.

**Encadré 5 — Le niveau avancé n'est pas une clé passe-partout.**
Déclarer `apiLevel: "advanced"` ouvre le remplacement d'outils et l'interface avancée. Cela n'ouvre **aucune** méthode supplémentaire vers le cœur de Beaver : le contrat actuel ne publie aucune méthode de niveau avancé, et tout appel à `beaver.unstable.call(...)` est rejeté aujourd'hui.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| L'outil de Beaver fonctionne encore, et un outil inconnu au nom presque identique est apparu | **Faute de frappe dans le nom.** Sans correspondance, la définition devient un nouvel outil publié sans préfixe, sans aucune erreur (`tool_bridge.rs:36-38`) | Ouvrir la fiche de l'extension : le nom réellement enregistré y est affiché, et l'étiquette « Remplace un outil intégré » est absente. Corriger le nom et **Recharger** |
| Aucun outil, skill ni ressource de l'extension n'apparaît, alors qu'un seul devait être un remplacement | Le manifeste ne déclare pas `apiLevel: "advanced"`. Beaver refuse **le lot entier** de contributions, pas seulement le remplacement (`runtime_sync_contributions.rs:24-26`) | Lire le diagnostic « Le mode avancé est requis pour remplacer un outil intégré », ajouter `"apiLevel": "advanced"` au manifeste, recharger et approuver à nouveau |
| Le remplacement ne s'applique que dans certaines conversations | L'extension n'est pas retenue dans le budget d'outils du modèle. L'outil d'origine est alors restauré silencieusement | Demander l'inspection de l'extension depuis l'agent (`inspect_extensions`), ou la marquer prioritaire ; voir `extensions-centre.md` |
| Le remplacement ne s'applique pas du tout, l'outil d'origine répond | L'extension est désactivée, non approuvée, ou l'application tourne en mode Chat | Vérifier l'activation et l'approbation dans la fiche ; le mode Chat n'expose que la recherche et la lecture Web, sans extensions |
| Ni l'outil d'origine ni le remplacement ne sont proposés | L'outil d'origine est désactivé dans les réglages : le remplacement suit ce réglage (`tool_availability.rs:1-7`) | Réactiver l'outil dans les réglages des outils |
| Le remplacement disparaît avec certains modèles | Certaines routes de fournisseur refusent les outils d'extension et restaurent l'outil d'origine, avec un avis affiché (`tool_policy.rs:42-58`) | Changer de modèle, ou accepter le comportement d'origine sur ces routes |
| Le remplacement fonctionnait, il a cessé après une mise à jour de Beaver | Le nom de l'outil du catalogue a changé ou l'outil a disparu. Le remplacement est alors devenu un outil supplémentaire, sans erreur | Comparer avec le catalogue de la nouvelle version ; conséquence directe du caractère instable du mécanisme |
| L'extension a changé, tout est redevenu comme avant | Beaver révoque l'approbation dès que les fichiers couverts par l'empreinte changent, désactive l'extension et redemande confirmation (`EXTENSIONS.md`, « Ajouter l'extension à Beaver ») | Réapprouver l'extension après avoir relu ce qui a changé |

---

## Renvois

- `extensions-centre.md` — installer, activer, approuver une extension, budget d'outils par conversation, priorité des extensions, diagnostics de l'Hôte.
- `extensions-ecrire.md` — écrire une extension : manifeste, `registerTool`, classes d'effet, skills et ressources, cycle de vie.
- `extensions-prompt-systeme.md` — ce que le modèle sait des extensions et comment il les découvre.
- `04-agent/permissions.md` — les trois modes de permission et les dialogues de confirmation.
- `05-outils/vue-densemble.md` — le catalogue des outils de Beaver, leur activation et leur verrouillage.
- `05-outils/terminal-et-shell.md` — l'outil `bash` d'origine et sa garde ; référence pour mesurer ce qu'un remplacement fait perdre.
- `05-outils/web.md` — la recherche Web d'origine et ses fournisseurs.
- `04-agent/sous-agents.md` — les profils de sous-agents, qui ne reçoivent qu'une partie des outils d'extension.

---

## Points à confirmer

1. **Incohérence interne d'`EXTENSIONS.md` sur la « liste blanche ».** La section « Remplacer un outil natif » affirme qu'il n'existe pas de liste de points de remplacement, ce que le code confirme. Deux autres passages du même fichier parlent d'« un outil natif prévu pour être remplaçable » et de « remplacer un outil natif explicitement remplaçable » (sections « Ce qu'une extension peut faire » et « Choisir rapidement »). **Le code ne connaît aucune notion d'outil explicitement remplaçable.** À trancher avec le propriétaire avant publication : corriger `EXTENSIONS.md`, ou documenter une liste qui n'existe pas encore.

2. **Défaut possible : doublon de définition pour les sous-agents.** Les définitions d'un sous-agent partent de la liste déjà fusionnée, puis y ajoutent une seconde fois toutes les définitions d'extension dont l'effet est autorisé par le profil (`subagent_tool_profile.rs:52-77`). Un remplacement dont le nom figure dans la liste du profil (`bash`, `read_file`, `web_search`…) semble donc apparaître **deux fois sous le même nom** dans la liste envoyée au modèle. Le tri qui suit (`extension_tool_set_apply.rs:63-77`) ne déduplique pas. Non reproduit à l'exécution : à faire vérifier par un développeur avant d'écrire quoi que ce soit à ce sujet sur le site.

3. **Comportement exact quand deux extensions remplacent le même outil.** La fusion s'applique extension par extension dans l'ordre de l'index (`tool_bridge.rs:25-41`) : la seconde recouvrirait la première, et le repli conservé serait celui de la première, non l'outil d'origine. Non testé, non couvert par `EXTENSIONS.md`. Ne rien affirmer avant vérification.

4. **Aucune vérification à l'écran.** L'étiquette « Remplace un outil intégré », le diagnostic « Le mode avancé est requis pour remplacer un outil intégré » et l'avis affiché quand une route refuse les extensions sont vérifiés dans le code, jamais observés dans l'application. Ils entrent dans la liste de contrôle de la passe d'interface de fin de parcours. Le diagnostic `advanced_required` suppose de fabriquer une extension fautive : il ne recevra probablement pas de capture.

5. **Périmètre non couvert : le remplacement des outils MCP et des outils de sous-agent.** La fusion opère sur le catalogue transmis par Beaver ; ce catalogue contient-il, au moment de la fusion, les outils MCP dynamiques ? Non vérifié. À trancher avant d'écrire une phrase générale du type « n'importe quel outil peut être remplacé ».

6. **Aucun exemple d'usage légitime vérifié.** Aucune extension livrée avec Beaver n'utilise `registerReplacement` (recherche dans `src-tauri/resources/extension-host/`, seules l'implémentation et sa déclaration de type apparaissent). La page gagnerait à donner un cas d'usage réel, validé par le propriétaire, plutôt qu'un exemple inventé.

7. **Version de référence.** Tous les relevés ci-dessus datent du 9 septembre 2026, sur `main` après le commit `ed28ff85`, application en version **1.2.2**. Le catalogue des outils et les classes d'effet sont exactement le genre de valeurs qui bougent : à revérifier avant publication.
