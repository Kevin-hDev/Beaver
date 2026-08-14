# Branches Git — `create_branch` et `checkout_branch`

**Emplacement site** — Outils › Branches Git
**Répond à** — « L'agent peut-il gérer mes branches Git, et jusqu'où va-t-il ? »
**Sources** — `tool_definitions_git.rs`, `tool_git_error.rs`, `services/git/branch.rs`, `services/git/action_error.rs`, `tool_catalog.rs`, `permission_gate.rs`
**Vérification** — Vérifié dans le code

> Cette page couvre **les deux outils Git de l'agent**. Le parcours Git complet côté interface — commits, envois, fusions, différentiels, historique — est dans `09-automatisation/git-workflow.md`.

---

## Plan de page proposé

1. Deux outils seulement
2. Pourquoi si peu
3. Créer une branche
4. Changer de branche
5. Ce qui bloque
6. Le reste passe par le terminal

---

## Contenu

### Deux outils seulement

Ils forment le groupe **Branches Git**, optionnel et **éteint par défaut**.

| Outil | Rôle | Approbation |
|---|---|---|
| `create_branch` | Crée une branche à partir de l'état courant et bascule dessus | **Oui** |
| `checkout_branch` | Bascule sur une branche existante | **Oui** |

Les deux demandent une approbation en mode Demande d'approbation. Ce sont les deux seules opérations Git que l'agent effectue par un outil dédié.

### Pourquoi si peu

C'est la question que se posera tout lecteur technique, et la réponse est un choix de conception assumé.

Les opérations Git complexes — fusion, rebasage, correction de commit, envoi vers un dépôt distant — **ne passent pas par des outils dédiés**. Elles passent par le terminal, où l'agent tape les commandes comme le ferait l'utilisateur.

L'avantage : rien à réimplémenter, rien à maintenir en parallèle de Git, et l'utilisateur voit **exactement** la commande exécutée plutôt qu'un outil opaque qui fait « quelque chose » avec son dépôt.

Les deux outils dédiés existent parce que ce sont les deux opérations que l'agent fait le plus souvent, et parce qu'elles bénéficient d'un traitement d'erreur soigné — notamment le refus de basculer avec des modifications non enregistrées.

### Créer une branche

- La branche part de **l'état courant** du dépôt, et l'agent bascule dessus immédiatement.
- Le dépôt visé est celui du **répertoire de travail** de la conversation.
- La création **échoue** si une branche du même nom existe déjà. Rien n'est écrasé.

### Changer de branche

- Bascule sur une branche **existante**.
- **Échoue s'il reste des modifications non enregistrées.** Le message indique **combien** de fichiers sont concernés, et demande de les examiner et de les préserver avant de continuer.

Ce refus est le comportement le plus utile des deux outils : il empêche qu'un changement de branche décidé par l'agent fasse perdre du travail en cours.

### Ce qui bloque

| Situation | Message | Que faire |
|---|---|---|
| Nom de branche invalide ou trop long | Nom de branche invalide / trop long | L'agent reformule |
| Branche déjà existante | Cette branche existe déjà | Choisir un autre nom, ou basculer dessus |
| **Dépôt sans aucun commit** | Le dépôt ne contient encore aucun commit | Créer le premier commit d'abord |
| Authentification requise | Authentification GitHub requise | Configurer l'accès au dépôt |
| Branche inexistante | Branche Git introuvable | Vérifier le nom |
| **Modifications non enregistrées** | Le dépôt contient N changement(s) non enregistré(s) | Les enregistrer ou les mettre de côté |
| Branche protégée, branche déjà active, commits non fusionnés | L'état du dépôt empêche ce changement de branche | Examiner l'état du dépôt |
| Identité Git absente ou invalide | Configuration Git invalide | Configurer nom et adresse dans Git |
| Pas de dépôt | Dépôt Git indisponible | Le répertoire de travail n'est pas un dépôt |

Deux cas sont traités à part, et c'est délibéré : quand Beaver **ne peut pas confirmer** qu'une création ou un changement de branche a abouti, le message ne dit pas « échec » mais **« n'a pas pu être confirmé »**. La nuance est importante : l'opération a peut-être réussi. Ces cas ne sont jamais rejoués automatiquement.

### Le reste passe par le terminal

Tout ce qui n'est pas créer ou changer de branche s'écrit en commandes shell :

- lister les branches, consulter l'état, lire un différentiel, consulter l'historique — commandes de lecture, **exécutées sans approbation** parce qu'elles font partie des commandes reconnues comme sûres ;
- enregistrer, fusionner, rebaser, envoyer — commandes qui modifient, donc soumises au mode de permission comme n'importe quelle commande.

**Conséquence à écrire sur le site** : couper le groupe Branches Git n'empêche pas l'agent de toucher à Git. Il le fera par le terminal, qui est verrouillé. Ce groupe ne contrôle que deux raccourcis, pas l'accès à Git.

---

## Encadrés

> **Changer de branche échoue s'il reste du travail non enregistré.**
> C'est le garde-fou le plus utile des deux outils. Le message dit combien de fichiers sont concernés.

> **Désactiver ce groupe ne coupe pas l'accès à Git.**
> L'agent continue de passer par le terminal, qui ne peut pas être désactivé. Pour limiter réellement ce qu'il fait de votre dépôt, le levier est le mode de permission.

> **« N'a pas pu être confirmé » n'est pas « a échoué ».**
> Quand Beaver ne peut pas vérifier le résultat d'une opération Git, il le dit ainsi plutôt que d'affirmer un échec. L'agent vérifie alors l'état du dépôt avant de recommencer.

> **Beaver ne réimplémente pas Git.**
> Les opérations complexes sont des commandes shell visibles dans la conversation. Ce qui s'exécute sur votre dépôt est exactement ce qui est affiché.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent ne crée pas de branche » | Groupe éteint par défaut | L'activer, ou le laisser passer par le terminal |
| « Le dépôt contient N changements non enregistrés » | Modifications en cours | Les enregistrer ou les mettre de côté |
| « Le dépôt ne contient encore aucun commit » | Dépôt fraîchement initialisé | Créer le premier commit |
| « Dépôt Git indisponible » | Le répertoire de travail n'est pas un dépôt Git | Ouvrir un projet versionné |
| « Configuration Git invalide » | Nom ou adresse absents de la configuration Git | Les configurer |
| « L'agent a créé une branche alors que je ne voulais pas » | Mode Accès complet | Passer en mode Demande d'approbation |

---

## Renvois

- `09-automatisation/git-workflow.md` — le parcours Git complet dans l'interface
- `05-outils/terminal-et-shell.md` — les commandes Git de lecture exécutées sans approbation
- `05-outils/sous-agents-outils.md` — les espaces Git isolés des sous-agents
- `04-agent/permissions.md`

---

## Points à confirmer

- Le message **« Authentification GitHub requise »** à la création d'une branche est surprenant : créer une branche est une opération locale. Il vient probablement d'un chemin qui interroge le dépôt distant. **À faire vérifier par l'équipe** — s'il apparaît réellement à l'utilisateur dans ce cas, le message est trompeur.
- Je n'ai **pas lu** le module Git complet, seulement les erreurs remontées par ces deux outils. La section 09 demandera une lecture de `services/git/`.
- Je n'ai **pas vérifié à l'écran** comment se présente une demande d'approbation de changement de branche, ni si l'interface affiche la branche courante pendant la conversation.
- Le comportement quand le répertoire de travail est un **espace de travail Git isolé de sous-agent** n'a pas été vérifié.
