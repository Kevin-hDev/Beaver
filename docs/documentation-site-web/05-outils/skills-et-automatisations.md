# Skills et automatisations — `load_skill` et `manage_automation`

**Emplacement site** — Outils › Skills et automatisations
**Répond à** — « Comment l'agent charge un guide spécialisé, et comment il crée une tâche qui se relance toute seule ? »
**Sources** — `tool_skill_loader.rs`, `tool_definitions_skills.rs`, `skill_catalog.rs`, `skill_parser.rs`, `tool_automation.rs`, `tool_automation_validation.rs`, `tool_definitions_automation.rs`, `models/config.rs`, `commands/heartbeat_validation.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Deux outils, deux groupes
2. Charger un skill
3. Ce que l'agent reçoit d'un skill
4. Créer une automatisation
5. Ce qu'une automatisation embarque
6. Ce qu'une automatisation ne peut pas faire
7. Modifier et supprimer

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

### Créer une automatisation

Une automatisation est une **tâche agentique programmée** : à l'heure dite, Beaver ouvre une conversation, donne une instruction à l'agent, et le laisse travailler.

L'agent peut en créer une lui-même — « rappelle-moi de vérifier les dépendances tous les lundis » — mais la définition de l'outil lui impose de **ne créer ou modifier qu'après confirmation de l'utilisateur** sur les cinq points : le déclencheur, l'instruction, les outils, les skills, et l'état actif.

Trois formes de déclencheur, et trois seulement :

| Forme | Ce qu'il faut préciser |
|---|---|
| Une fois | Une date et une heure |
| Chaque jour | Une heure |
| Chaque semaine | Un jour de la semaine et une heure |

Il n'y a **pas d'expression de planification libre**. Une syntaxe de type cron est refusée.

### Ce qu'une automatisation embarque

Au moment de sa création, l'automatisation fige :

- **le modèle et le fournisseur** de la conversation en cours ;
- **le répertoire de travail** de la conversation en cours ;
- **la liste exacte des outils** dont elle a besoin — au plus **12** ;
- **la liste exacte des skills** à charger — au plus **8** ;
- l'instruction à exécuter.

Ce point mérite d'être écrit clairement sur le site : **une automatisation ne dispose que des outils qui lui ont été explicitement donnés**. Ce n'est pas une conversation ordinaire qui s'ouvre toute seule ; c'est une tâche à portée réduite, décidée à l'avance.

Chaque nom d'outil et chaque identifiant de skill est vérifié à la création : un outil inconnu ou un skill introuvable font échouer la création plutôt que de produire une automatisation qui échouera silencieusement à sa première exécution.

### Ce qu'une automatisation ne peut pas faire

Douze outils sont **refusés** dans une automatisation, pour deux raisons distinctes.

**Parce qu'il n'y a personne devant l'écran** :

- poser une question à choix ;
- proposer un plan et attendre un accord.

**Parce qu'une tâche programmée ne doit pas se ramifier** :

- déléguer à un sous-agent, et les huit outils de suivi et d'application qui vont avec ;
- créer ou modifier une autre automatisation.

Ce dernier point est le plus important : sans lui, une automatisation pourrait en créer d'autres, qui en créeraient d'autres. La règle coupe la récursion à la racine.

### Modifier et supprimer

- Les modifications sont **partielles** : ce qui n'est pas fourni reste inchangé.
- **La suppression exige une confirmation explicite.** Sans elle, l'appel est refusé.
- Seules les automatisations agentiques sont concernées : l'outil ne touche pas aux autres réveils programmés.
- Toute création, modification ou suppression **prévient immédiatement le planificateur** — le changement prend effet sans redémarrage.
- Quand les réveils sont **globalement en pause**, une automatisation créée active est enregistrée mais mise en pause, et marquée comme telle. Elle repart quand la pause globale est levée.

---

## Tableaux

### Les limites

| Limite | Valeur |
|---|---|
| Taille d'un fichier de skill | **256 Ko** |
| Longueur d'un identifiant de skill | **768 octets** |
| Nom d'un skill affiché | **120 caractères** |
| Skills par automatisation | **8** |
| Outils par automatisation | **12** |

### Les outils refusés dans une automatisation

| Outil | Raison |
|---|---|
| Choix interactif | Personne devant l'écran |
| Mode Plan | Personne pour approuver |
| Gestion d'automatisations | Empêche la récursion |
| Délégation à un sous-agent | Pas de ramification |
| Les huit outils de suivi et d'application des sous-agents | Idem |

### Les erreurs

| Message | Cause |
|---|---|
| Identifiant de skill invalide | Identifiant contenant un séparateur de chemin ou `..` |
| Skill introuvable | Identifiant absent du catalogue |
| Skill indisponible | Fichier illisible ou trop volumineux |
| Déclencheur requis / invalide | Planification absente ou de forme non supportée |
| Skill d'automatisation introuvable | Un des skills demandés n'existe pas |
| Outil d'automatisation non autorisé | Outil inconnu ou figurant dans la liste des refusés |
| Liste d'automatisation trop longue | Plus de 8 skills ou 12 outils |
| Confirmation requise | Suppression sans confirmation explicite |
| Automatisation introuvable | Identifiant inconnu |

---

## Encadrés

> **La description d'un skill décide s'il servira un jour.**
> C'est le seul élément que l'agent voit avant de charger le guide. Une description qui dit « pour les revues » ne se déclenchera pas ; une description qui dit quand l'utiliser, oui.

> **Une automatisation n'a que les outils qu'on lui a donnés.**
> Douze au maximum, nommés un par un à la création. Ce n'est pas une conversation ordinaire déclenchée par une horloge.

> **Une automatisation ne peut pas en créer une autre.**
> Ni déléguer à un sous-agent. La règle empêche qu'une tâche programmée se multiplie sans que personne l'ait décidé.

> **Supprimer demande une confirmation explicite.**
> C'est l'une des rares opérations de Beaver où une confirmation est exigée dans le protocole lui-même, pas seulement dans l'interface.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent n'utilise jamais mon skill » | Description trop vague, ou groupe Skills désactivé | Réécrire la description en disant *quand* utiliser le skill |
| « L'agent invente un nom de skill » | Ne devrait pas arriver : les identifiants viennent de la liste | Signaler ; l'appel échoue avec « Skill introuvable » |
| « Mon skill n'apparaît pas dans la liste » | Fichier mal nommé, en-tête absent, ou fichier trop gros | Voir `04-agent/skills-locaux.md` |
| « L'automatisation ne se déclenche pas » | Réveils globalement en pause | Vérifier la pause globale dans l'écran des réveils |
| « L'automatisation échoue à chaque fois » | Il lui manque un outil non déclaré à la création | La modifier pour ajouter l'outil |
| « L'agent refuse de créer une automatisation qui en crée d'autres » | Interdit par conception | Comportement voulu |
| « Je voulais une planification toutes les deux heures » | Seuls une fois, chaque jour et chaque semaine existent | Créer plusieurs automatisations quotidiennes |

---

## Renvois

- `04-agent/skills-locaux.md` — écrire et installer un skill
- `09-automatisation/reveils.md` — les réveils programmés, dont les automatisations sont un cas
- `09-automatisation/historique-des-reveils.md` — lire ce qui s'est passé
- `05-outils/sous-agents-outils.md` — les outils interdits en automatisation
- `10-reglages/agent.md`

---

## Points à confirmer

- **Le nombre maximal d'automatisations** est vérifié à la création par un contrôle de capacité que je n'ai pas lu. À compléter avant publication — c'est une valeur que l'utilisateur voudra connaître.
- **La différence entre un « réveil » et une « automatisation »** doit être tranchée pour le site. Dans le code, une automatisation est un réveil marqué comme agentique ; les deux vivent dans le même fichier de configuration et le même écran. Décider d'un vocabulaire unique, sinon la section 09 et cette page se contrediront.
- Je n'ai **pas vérifié à l'écran** ce que voit l'utilisateur quand l'agent crée une automatisation : y a-t-il une carte de confirmation, ou simplement du texte ?
- La consigne « demander confirmation avant de créer » vit **dans la description de l'outil**, c'est-à-dire dans une instruction au modèle — pas dans une garde technique. Un modèle peut ne pas la suivre. À signaler à l'équipe : faut-il en faire une véritable demande d'approbation ? Aujourd'hui, `manage_automation` est bien soumis à approbation en mode Demande d'approbation, mais pas en mode Accès complet.
