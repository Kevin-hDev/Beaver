# Skills et automatisations — `load_skill` et `manage_automation`

**Emplacement site** — Outils › Skills et automatisations
**Répond à** — « Comment l'agent charge un guide spécialisé, et comment il crée une tâche qui se relance toute seule ? »
**Sources** — `tool_skill_loader.rs`, `tool_definitions_skills.rs`, `skill_catalog.rs`, `skill_parser.rs`, `skill_limits.rs`, `services/skill_manifest_policy.rs`, `models/agent_turn_contract.rs`, `tool_automation.rs`, `tool_automation_validation.rs`, `tool_definitions_automation.rs`, `models/config.rs`, `commands/heartbeat_validation.rs`, `services/scheduler/agentic.rs`, `commands/agent_chat_task/common.rs`, `commands/agent_chat_task/api_tools.rs`
**Vérification** — Vérifié dans le code, le 9 septembre 2026

---

## Avertissement au rédacteur

**Le modèle de sécurité des automatisations a été inversé.** Une version antérieure de ce fichier décrivait une tâche à portée réduite, dotée d'une liste d'outils figée à la création. Ce mécanisme n'existe plus : une automatisation s'exécute aujourd'hui **en Accès complet, avec la totalité des outils et des skills activés dans l'application**.

Toute reprise d'un texte antérieur sur cette page est à écarter. Le cœur de la page est cet avertissement, pas la procédure de création.

---

## Plan de page proposé

1. Deux outils, deux groupes
2. Charger un skill
3. Ce que l'agent reçoit d'un skill
4. Ce qu'est une automatisation
5. **Une automatisation s'exécute en Accès complet**
6. Créer une automatisation
7. Ce qu'une automatisation fige
8. Modifier et supprimer
9. Les bornes qui restent

---

## Contenu

### Deux outils, deux groupes

| Outil | Groupe | Par défaut |
|---|---|---|
| `load_skill` | Skills | **Actif** |
| `manage_automation` | Automatisations | **Actif** |

Les deux sont optionnels et se coupent indépendamment dans **Réglages › Agent › Outils**.

### Charger un skill

Un skill est un guide écrit d'avance : une marche à suivre pour une tâche précise — revue de code, audit de sécurité, conception d'interface. Il vit dans un fichier sur le disque de l'utilisateur.

Le mécanisme est en deux temps, et c'est ce qui le rend économe :

1. **La liste des skills disponibles** — leur nom et leur description courte, rien de plus — est présente dans la conversation en permanence.
2. **Le contenu complet d'un skill n'est chargé que quand l'agent le demande**, par cet outil.

Conséquences pratiques :

- Cinquante skills installés coûtent cinquante descriptions courtes, pas cinquante guides complets.
- **La description décide de tout.** C'est le seul élément sur lequel l'agent se fonde pour décider de charger un skill. Une description vague donne un skill qui ne se déclenche jamais.
- L'agent ne peut charger qu'un skill **présent dans la liste** : les identifiants ne s'inventent pas.
- Un skill déjà chargé dans le tour en cours n'est pas rechargé.

Une consigne forte figure dans la définition de l'outil : quand un skill correspond à la demande, **l'agent doit le charger avant de répondre quoi que ce soit** sur la tâche. C'est ce qui évite qu'il commence à travailler à sa manière puis découvre le guide à mi-parcours.

### Ce que l'agent reçoit d'un skill

Le contenu injecté est précédé de deux informations : **d'où vient le skill** et **dans quel dossier il se trouve**. Ce second point compte : un skill peut renvoyer à des fichiers voisins — des modèles, des exemples, des scripts — et l'agent a besoin de savoir où les chercher.

Contrôles appliqués au chargement :

- l'identifiant ne peut contenir ni barre oblique, ni `..`, ni caractère nul — un identifiant ne sert pas à remonter dans l'arborescence ;
- le fichier doit exister, être un fichier, et peser au plus **256 Ko** ;
- l'en-tête de description est retiré : seul le corps du guide est transmis ;
- le nom affiché est ramené à une seule ligne et plafonné à **120 caractères**.

### Ce qu'est une automatisation

Une automatisation est une **tâche agentique programmée** : à l'heure dite, Beaver ouvre une conversation, y dépose l'instruction enregistrée, et laisse l'agent travailler jusqu'au bout, sans personne devant l'écran.

Dans le code, une automatisation est un **réveil programmé** comme un autre : elle vit dans le même fichier de configuration (`config.json`, tableau `scheduled_wakeups`) et dans le même écran que les réveils non agentiques, et elle partage leur plafond commun de **64**.

### Une automatisation s'exécute en Accès complet

**C'est l'information la plus importante de cette page**, et elle doit être écrite avant toute procédure.

La définition de l'outil l'énonce mot pour mot : *« Every automation runs through the complete Agent Local engine in full-access mode, with all currently enabled tools and skills. »* Le planificateur l'applique : la conversation est lancée en mode **Accès complet**, quel que soit le mode choisi pour vos conversations ordinaires.

Ce que cela signifie concrètement :

- **Aucune demande d'approbation n'est possible.** Personne n'est là pour répondre, et le mode ne le prévoit pas.
- **Tous les outils activés dans l'application sont disponibles** — exactement la même liste que dans une conversation ordinaire, lue au moment de l'exécution dans `agent-settings.json`. Y compris ceux qui écrivent des fichiers, lancent des commandes shell et délèguent à des sous-agents.
- **Tous les skills installés sont disponibles.**
- Il n'existe **aucune liste d'outils propre à une automatisation**, ni à la création, ni à l'exécution. Rien n'est restreint, rien n'est refusé par nature.
- Une automatisation dispose donc de `manage_automation` : **elle peut en créer, en modifier et en supprimer d'autres.** La seule borne est le plafond de 64 réveils.

Le corollaire à écrire pour l'utilisateur : **ce que vous confiez à une automatisation, vous le confiez sans filet**. Le seul point de contrôle est le texte de l'instruction et le choix des outils activés dans les réglages — tous deux décidés à l'avance, une fois pour toutes.

Le changement de mode de permission de vos conversations n'y change rien : régler Beaver en **Demande d'approbation** ne borne pas les automatisations.

### Créer une automatisation

L'agent peut créer une automatisation lui-même — « rappelle-moi de vérifier les dépendances tous les lundis ». La définition de l'outil lui impose de **ne créer ou modifier qu'après confirmation de l'utilisateur** sur **trois points** : le déclencheur, l'instruction et l'état actif.

Trois formes de déclencheur, et trois seulement :

| Forme | Ce qu'il faut préciser |
|---|---|
| Une fois | Une date et une heure, au format `AAAA-MM-JJTHH:MM` |
| Chaque jour | Une heure, au format `HH:MM` |
| Chaque semaine | Un jour de la semaine (**0 à 6**) et une heure |

Il n'y a **pas d'expression de planification libre**. Une syntaxe de type cron est refusée par le lecteur de déclencheur.

Une date ponctuelle **déjà passée** est refusée si l'automatisation est créée active.

### Ce qu'une automatisation fige

Au moment de sa création, l'automatisation reprend de la conversation en cours et fige :

- **le modèle** ;
- **le fournisseur** ;
- **le projet**, s'il y en a un.

Elle enregistre en plus son nom, sa description, son instruction, son déclencheur, son état actif et sa date de création. C'est tout : la structure écrite sur le disque ne contient rien d'autre.

Le fournisseur est vérifié à la création : un fournisseur qui n'accepte pas les requêtes d'automatisation — certains comptes web connectés, réservés à l'usage interactif — fait échouer la création avec « Provider non supporté ».

### Modifier et supprimer

- Les modifications sont **partielles** : ce qui n'est pas fourni reste inchangé. Le modèle, le fournisseur et le projet ne se modifient pas par cet outil.
- **La suppression exige `confirm=true`.** Sans lui, l'appel est refusé — c'est la seule garde technique du protocole.
- Seules les automatisations agentiques sont concernées : l'outil ne touche pas aux autres réveils programmés.
- Toute création, modification ou suppression **prévient immédiatement le planificateur** — le changement prend effet sans redémarrage.
- Quand les réveils sont **globalement en pause**, une automatisation créée ou modifiée active est enregistrée mais mise en pause, et marquée comme telle. Elle repart quand la pause globale est levée.

### Les bornes qui restent

Le modèle de sécurité a beau être ouvert, quelques limites tiennent, et elles valent d'être données :

- **64 réveils au total**, automatisations et réveils simples confondus. Au-delà : « Maximum 64 réveils ».
- **L'instruction fait 12 000 caractères au plus**, le nom **120**, la description **300**.
- Le nom, le modèle, le fournisseur et l'instruction sont **obligatoires et non vides**.
- Une automatisation ne peut pas se donner un modèle ou un fournisseur autres que ceux de la conversation qui l'a créée.

---

## Tableaux

### Les limites

| Limite | Valeur |
|---|---|
| Taille d'un fichier de skill | **256 Ko** |
| Longueur d'un identifiant de skill | **768 octets** |
| Nom d'un skill affiché | **120 caractères** |
| Réveils au total, automatisations comprises | **64** |
| Instruction d'une automatisation | **12 000 caractères** |
| Nom d'une automatisation | **120 caractères** |
| Description d'une automatisation | **300 caractères** |

### Ce dont dispose une automatisation

| | Conversation ordinaire | Automatisation programmée |
|---|---|---|
| Mode de permission | Celui que vous avez choisi | **Accès complet, toujours** |
| Outils disponibles | Les groupes activés dans les réglages | **Les mêmes**, sans restriction |
| Skills disponibles | Tous | **Tous** |
| Demande d'approbation | Selon votre mode | **Jamais** |
| Peut créer une autre automatisation | Oui | **Oui** |
| Peut déléguer à un sous-agent | Oui | **Oui** |

### Les erreurs

| Message | Code interne | Cause |
|---|---|---|
| Identifiant de skill invalide | — | Identifiant contenant un séparateur de chemin ou `..` |
| Skill introuvable | — | Identifiant absent du catalogue |
| Skill indisponible | — | Fichier illisible ou trop volumineux |
| Action d'automatisation invalide | `automation_action_invalid` | Action autre que lister, créer, modifier, supprimer |
| Déclencheur requis | `automation_schedule_required` | Aucune planification fournie à la création |
| Déclencheur invalide | `automation_schedule_invalid` | Forme non supportée, par exemple une syntaxe cron |
| Champ requis, champ trop long, heure ou jour invalides, date déjà passée, fournisseur non supporté | `automation_invalid` | Contrôles de validité à la création |
| Création impossible | `automation_create_failed` | Plafond de 64 réveils atteint, ou écriture refusée |
| Modification impossible | `automation_update_failed` | Identifiant inconnu, ou valeur devenue invalide |
| Confirmation requise | `automation_confirmation_required` | Suppression sans `confirm=true` |
| Suppression impossible | `automation_delete_failed` | Identifiant inconnu |
| Automatisation indisponible | `automation_unavailable` | Configuration ou session illisible |

---

## Encadrés

> **Une automatisation s'exécute sans aucune limite de permission.** — avertissement, en tête de la partie automatisations.
> À l'heure dite, Beaver ouvre une conversation en **Accès complet** et laisse l'agent travailler seul, avec **tous les outils et tous les skills activés** dans l'application : écriture de fichiers, commandes shell, délégation à des sous-agents. Aucune confirmation ne peut être demandée, puisque personne n'est devant l'écran. Le mode de permission que vous avez choisi pour vos conversations ne s'applique pas ici.
> Avant de programmer une tâche, relisez son instruction en vous demandant ce qu'elle autorise au pire, et coupez dans **Réglages › Agent › Outils** les groupes dont elle n'a pas besoin.

> **Une automatisation peut en créer d'autres.**
> Rien ne l'en empêche : elle dispose de l'outil de gestion des automatisations comme n'importe quelle conversation. La seule borne est le plafond global de **64 réveils**.

> **La description d'un skill décide s'il servira un jour.**
> C'est le seul élément que l'agent voit avant de charger le guide. Une description qui dit « pour les revues » ne se déclenchera pas ; une description qui dit quand l'utiliser, oui.

> **Supprimer demande une confirmation explicite.**
> C'est l'une des rares opérations de Beaver où une confirmation est exigée dans le protocole lui-même, pas seulement dans l'interface.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent n'utilise jamais mon skill » | Description trop vague, ou groupe Skills désactivé | Réécrire la description en disant *quand* utiliser le skill |
| « L'agent invente un nom de skill » | Ne devrait pas arriver : les identifiants viennent de la liste | Signaler ; l'appel échoue avec « Skill introuvable » |
| « Mon skill n'apparaît pas dans la liste » | Fichier mal nommé, en-tête absent, ou fichier trop gros | Voir `04-agent/skills-locaux.md` |
| « Mon automatisation a modifié des fichiers sans rien demander » | Elle s'exécute en Accès complet, par conception | Restreindre son instruction, ou couper les groupes d'outils concernés |
| « Je suis en Demande d'approbation, et pourtant elle n'a rien demandé » | Le mode de la conversation ne s'applique pas aux automatisations | Comportement voulu ; le contrôle se fait sur l'instruction et les outils activés |
| « L'automatisation ne se déclenche pas » | Réveils globalement en pause | Vérifier la pause globale dans l'écran des réveils |
| « Maximum 64 réveils » | Plafond commun aux automatisations et aux réveils simples | Supprimer un réveil devenu inutile |
| « Provider non supporté » | Le fournisseur de la conversation n'accepte pas les requêtes d'automatisation | Créer l'automatisation depuis une conversation utilisant un autre fournisseur |
| « Date ponctuelle déjà passée » | Déclencheur unique fixé dans le passé, avec l'état actif | Choisir une date à venir |
| « Je voulais une planification toutes les deux heures » | Seuls une fois, chaque jour et chaque semaine existent | Créer plusieurs automatisations quotidiennes |

---

## Renvois

- `04-agent/skills-locaux.md` — écrire et installer un skill
- `04-agent/permissions.md` — les trois modes, et pourquoi celui-ci ne s'applique pas aux automatisations
- `05-outils/vue-densemble.md` — choisir les groupes d'outils, seul vrai levier de restriction d'une automatisation
- `09-automatisation/reveils.md` — les réveils programmés, dont les automatisations sont un cas
- `09-automatisation/historique-des-reveils.md` — lire ce qui s'est passé
- `10-reglages/agent.md`

---

## Points à confirmer

- **La consigne « demander confirmation avant de créer » vit dans la description de l'outil**, c'est-à-dire dans une instruction au modèle — pas dans une garde technique. Un modèle peut ne pas la suivre. `manage_automation` est bien soumis à approbation en mode Demande d'approbation, mais pas en mode Accès complet — et une automatisation s'exécutant toujours en Accès complet, elle en crée d'autres sans aucune confirmation. **À remonter à l'équipe produit** : faut-il une garde technique, une borne au nombre d'automatisations créées par une automatisation, ou l'exclusion de cet outil des sessions programmées ?
- **La différence entre un « réveil » et une « automatisation »** doit être tranchée pour le site. Dans le code, une automatisation est un réveil marqué comme agentique ; les deux vivent dans le même fichier de configuration et le même écran, et partagent le plafond de 64. Décider d'un vocabulaire unique, sinon la section 09 et cette page se contrediront.
- Je n'ai **pas vérifié à l'écran** ce que voit l'utilisateur quand l'agent crée une automatisation : y a-t-il une carte de confirmation, ou simplement du texte ? Ni comment le mode Accès complet d'une conversation programmée est signalé, s'il l'est.
- **La liste exacte des fournisseurs refusés en automatisation** n'a pas été relevée. Le contrôle interroge la disponibilité du fournisseur pour un usage d'automatisation ; les tests du code montrent au moins deux comptes web réservés à l'interactif. À établir avant de publier une phrase générale.
