# Les neuf outils de délégation

**Emplacement site** — Outils › Sous-agents
**Répond à** — « Quels sont les outils qui servent à déléguer, suivre et récupérer le travail d'un sous-agent ? »
**Sources** — `tool_definitions_subagent.rs`, `tool_delegate.rs`, `tool_subagent_control.rs`, `tool_subagent_changes.rs`, `subagent_git_actions.rs`, `subagent_registry.rs`, `tool_catalog.rs`, `tool_automation_validation.rs`
**Vérification** — Vérifié dans le code

> Cette page décrit **les outils**. Le fonctionnement des sous-agents — sessions isolées, espaces de travail Git séparés, limites — est expliqué dans `04-agent/sous-agents.md`. Les deux pages se lisent ensemble ; celle-ci est la référence, l'autre l'explication.

---

## Plan de page proposé

1. Un groupe, neuf outils
2. Déléguer une tâche
3. Suivre les sous-agents
4. Récupérer le travail d'un sous-agent codeur
5. Ce qui peut mal tourner

---

## Contenu

### Un groupe, neuf outils

Ils forment le groupe **Sous-agents**, optionnel et **actif par défaut**.

Le groupe est **indivisible** : il n'est pas possible d'autoriser la délégation sans autoriser le suivi, ni l'inverse. Techniquement, tout dépend de l'outil de délégation — s'il est absent, les huit autres sont retirés automatiquement.

| Outil | Rôle | Approbation |
|---|---|---|
| `delegate_task` | Lance un sous-agent | Non |
| `list_subagents` | Liste les sous-agents de la conversation | Non |
| `get_subagent` | Détail et résumé final d'un sous-agent | Non |
| `message_subagent` | Envoie une nouvelle instruction | Non |
| `cancel_subagent` | Arrête un sous-agent en cours | Non |
| `archive_subagent` | Archive un sous-agent terminé | Non |
| `inspect_subagent_changes` | Examine des modifications en attente | Non |
| `apply_subagent_changes` | **Intègre** les modifications | **Oui** |
| `discard_subagent_changes` | **Abandonne** les modifications | Non |

Un seul de ces neuf outils demande une approbation : celui qui intègre du code écrit par un sous-agent dans la branche de l'utilisateur. C'est le seul qui modifie durablement le projet.

### Déléguer une tâche

Deux types de sous-agents, et le choix détermine ce qu'ils peuvent faire :

| Type | Outils dont il dispose | Écrit dans le projet |
|---|---|---|
| **Explorateur** | Lecture de fichiers, listing, recherche par motif, recherche par nom, recherche web, ouverture de page | **Non** |
| **Codeur** | Création et modification de fichiers, dans un espace de travail Git isolé | **Oui**, mais à l'écart |

Points à retenir :

- **Le résultat d'un sous-agent n'est pas montré à l'utilisateur.** Il revient à l'agent principal, qui doit le résumer. C'est écrit noir sur blanc dans la définition de l'outil.
- Un sous-agent travaille **dans sa propre conversation, visible**, pendant que l'agent principal continue.
- Plusieurs sous-agents peuvent travailler **en parallèle** sur des sous-tâches indépendantes.
- L'agent a pour consigne de **ne pas refaire lui-même** le travail délégué, et de ne pas conclure avant d'avoir reçu les rapports.
- La délégation est déconseillée pour les tâches courtes : lire un fichier connu, chercher une fonction précise, une opération en une ou deux étapes.
- **Un sous-agent ne peut pas déléguer.** La récursion est interdite : il est la seule et unique génération d'enfants de sa conversation.

L'instruction donnée au sous-agent doit être **structurée** — contexte, tâche, contraintes, format attendu. Un sous-agent ne voit rien de la conversation parente : ce que l'agent oublie de lui écrire, il ne peut pas le deviner. La définition de l'outil le dit sans détour : une instruction laconique produit un résultat superficiel.

Une variante existe : réutiliser un sous-agent déjà créé plutôt que d'en lancer un neuf, ou pointer vers une **définition d'agent spécialisé** écrite d'avance dans le projet. Les chemins absolus et les remontées d'arborescence y sont refusés.

### Suivre les sous-agents

- **Lister** donne pour chaque sous-agent son état, son nom, sa description et son identifiant.
- **Consulter** donne le détail d'un seul, avec son résumé final quand il a terminé.
- **Envoyer une instruction** à un sous-agent déjà lancé : s'il travaille encore, l'instruction est **mise en file** et traitée ensuite.
- **Annuler** arrête un sous-agent en cours.
- **Archiver** range un sous-agent terminé. Un sous-agent qui travaille encore doit être annulé d'abord.

Tous ces outils ne voient que **les sous-agents de la conversation en cours**. Une conversation ne peut ni lister ni piloter les sous-agents d'une autre.

### Récupérer le travail d'un sous-agent codeur

C'est la partie la plus délicate, et celle qui mérite le plus d'explications sur le site.

Un sous-agent codeur ne modifie **pas** le projet directement. Il travaille dans un espace Git isolé, et son travail devient un **changement en attente** — une proposition, pas une modification.

Le cycle est en trois temps :

1. **Examiner** — l'agent récupère le détail du changement et le différentiel complet. Aucun effet, aucune approbation.
2. **Intégrer** — le changement est appliqué sur la branche courante de l'utilisateur. **C'est l'étape approuvée.**
3. **Abandonner** — le changement est rejeté et sa branche temporaire supprimée.

Deux détails de conception qui comptent :

- L'agent a pour consigne **d'examiner avant d'intégrer**, sauf si le contenu a déjà été passé en revue.
- Après une intégration faite **à la main** par l'utilisateur, la bonne pratique est d'appeler quand même l'abandon : cela nettoie le changement et sa branche temporaire, qui sinon restent sur le disque.

Les identifiants nécessaires — celui du sous-agent et celui du changement — sont **fournis à l'agent** dans les données du changement. Ils doivent être repris exactement, sans être intervertis. Ce sont des identifiants uniques, vérifiés avant toute opération.

### Ce qui peut mal tourner

L'intégration d'un changement touche à Git, donc à l'état réel du dépôt. Les échecs sont classés avec soin, parce qu'ils n'appellent pas la même réaction.

| Situation | Ce que ça veut dire | Ce qu'il faut faire |
|---|---|---|
| **Conflit** | Le changement ne s'applique pas sur l'état actuel | Examiner et résoudre avant de poursuivre |
| **Branche cible différente** | La branche a changé depuis la création du changement | Revenir sur la bonne branche, ou refaire le changement |
| **État incompatible** | Le changement n'est ni prêt à être intégré, ni abandonnable | Examiner l'état du dépôt et du changement |
| **Changement introuvable** | Il n'existe plus | Rien à faire |
| **Échec de restauration** | Beaver n'a pas pu ramener le dépôt à son état antérieur | **Vérifier le dépôt à la main avant toute opération Git** |
| **Dépendance indisponible** | Un élément nécessaire manque | Vérifier le dépôt : l'opération a pu être partiellement appliquée |
| **Capacité atteinte** | Trop de projets isolés en cours | Nettoyer les changements en attente |

Le message d'échec d'intégration est explicite et **dit quoi faire** : le changement isolé reste non résolu, il faut inspecter son état, et après une intégration manuelle appeler l'abandon pour nettoyer.

**Aucune de ces erreurs n'est présentée comme rejouable à l'aveugle**, sauf la capacité atteinte. Une opération Git interrompue laisse un état que seul un examen permet de connaître.

---

## Tableaux

### Les limites

| Limite | Valeur |
|---|---|
| Sous-agents par conversation | **4** |
| Sous-agents actifs sur toute l'application | **8** |
| Générations de sous-agents | **1** — un sous-agent ne délègue pas |

> Ne pas confondre avec la limite de **10 appels d'outils de lecture en parallèle**, qui concerne l'agent principal et n'a rien à voir avec les sous-agents.

### Les outils par type de sous-agent

| Explorateur | Codeur |
|---|---|
| Lire un fichier | Lire un fichier |
| Lister un dossier | Créer un fichier |
| Chercher par motif | Modifier un fichier |
| Chercher par nom | Travailler dans un espace Git isolé |
| Chercher sur le web | |
| Ouvrir une page | |

### Où ces outils sont interdits

| Contexte | Raison |
|---|---|
| Dans un sous-agent | Pas de récursion |
| Dans une automatisation programmée | Une tâche planifiée ne doit pas se ramifier |

---

## Encadrés

> **Le résultat d'un sous-agent n'apparaît pas devant l'utilisateur.**
> Il revient à l'agent principal, qui doit le résumer. Un utilisateur qui veut le détail doit ouvrir la conversation du sous-agent, qui reste visible.

> **Un sous-agent codeur ne modifie jamais votre projet directement.**
> Il produit une proposition dans un espace isolé. Rien n'entre dans la branche de l'utilisateur sans une intégration explicite — la seule opération approuvée des neuf.

> **Un sous-agent ne peut pas en lancer un autre.**
> La règle est absolue et empêche qu'une délégation se ramifie sans limite.

> **Après une intégration manuelle, abandonner le changement.**
> Sinon la branche temporaire et le changement en attente restent sur le disque.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Je ne vois pas ce que le sous-agent a trouvé » | Son rapport va à l'agent principal | Ouvrir sa conversation, ou demander le détail |
| « Limite de sous-agents atteinte » | 4 par conversation, 8 en tout | Archiver les terminés |
| « Le changement ne s'applique pas » | Conflit avec l'état actuel | L'agent examine et résout |
| « Branche cible incompatible » | Changement de branche depuis la création | Revenir sur la bonne branche |
| « Des branches temporaires traînent dans mon dépôt » | Changements ni intégrés ni abandonnés | Demander l'abandon des changements en attente |
| « Le sous-agent ne répond pas à mon message » | Il travaille : l'instruction est en file | Elle sera traitée à la fin du travail en cours |
| « L'agent a refait lui-même ce qu'il avait délégué » | Le modèle n'a pas suivi la consigne | Signaler ; c'est un défaut de comportement du modèle |

---

## Renvois

- `04-agent/sous-agents.md` — le fonctionnement, les sessions isolées, les espaces de travail
- `05-outils/git.md` — les opérations Git de l'agent principal
- `09-automatisation/git-workflow.md` — le parcours Git complet côté interface
- `04-agent/permissions.md` — pourquoi seule l'intégration demande une approbation
- `05-outils/skills-et-automatisations.md` — pourquoi ces outils sont interdits en automatisation

---

## Points à confirmer

- **Les sous-agents portent des noms visibles fixes** — « Claudiator » pour le codeur, « Geminitor » pour l'explorateur. Ce sont des noms qui évoquent d'autres produits. **À trancher par l'équipe produit avant publication** : sont-ils réellement affichés à l'utilisateur ? Si oui, le site les documentera, et il faut être sûr qu'ils sont voulus. Je ne les ai pas repris dans le corps de la page.
- Deux paramètres de l'outil de délégation sont marqués comme **hérités d'une version antérieure** dans le code. À ignorer pour le site, à nettoyer côté produit.
- La **définition d'agent spécialisé** — un fichier Markdown dans le projet décrivant un sous-agent réutilisable — est mentionnée mais je n'ai pas lu son format ni sa validation. **Fonctionnalité potentiellement importante et non documentée.** À explorer avant publication, ou à laisser de côté explicitement.
- Le nombre maximal de **projets isolés simultanés** n'a pas été relevé ; il apparaît dans les erreurs de capacité.
- Je n'ai **pas vérifié à l'écran** comment se présentent la conversation d'un sous-agent, la liste des changements en attente et l'écran de différentiel.
