# Instructions permanentes — AGENTS.md et personnalité

**Emplacement site** — Agent › Instructions permanentes
**Répond à** — « Comment je donne à l'agent des consignes qu'il applique à chaque conversation, sans les répéter ? »
**Sources** — `src-tauri/src/services/agent_local/agent_md.rs` (lignes 3-4, 26-112), `src-tauri/src/services/personality_injection.rs`, `src-tauri/src/storage_migration.rs` (défauts d'injection), `src-tauri/src/commands/agent_md.rs`, `src-tauri/src/commands/personality.rs`, `src/components/personality/`
**Vérification** — Vérifié dans le code : ordre d'assemblage, limite globale, sources prises en compte

---

## Le point le plus utile de cette page

**Six sources d'instructions sont assemblées dans un ordre précis**, et cet ordre détermine qui gagne en cas de contradiction : les instructions de projet, plus spécifiques, viennent après les globales.

C'est l'information qu'un utilisateur cherche quand une consigne semble ignorée.

---

## Plan de page proposé

1. Les deux mécanismes
2. AGENTS.md — l'ordre d'assemblage
3. Où placer ses instructions
4. La personnalité
5. La limite de taille
6. Écrire de bonnes instructions

---

## Contenu

### 1. Les deux mécanismes

Deux façons distinctes de donner des consignes permanentes :

| | AGENTS.md | Personnalité |
|---|---|---|
| Contenu | Conventions, commandes du projet, interdits | Ton, comportement, façon de travailler |
| Portée | Global et par projet | Global |
| Format | Markdown libre | Fichiers Markdown activables un par un |
| Emplacement | Dossier de données et dossier du projet | `memory/core/` |

Elles se cumulent. Un troisième mécanisme existe pour ce qui doit évoluer tout seul : la **mémoire**. Renvoyer vers sa page — la distinction vaut d'être posée : `AGENTS.md` est ce que **vous** écrivez, la mémoire est ce que l'**agent** retient.

### 2. AGENTS.md — l'ordre d'assemblage

Six sources, assemblées **dans cet ordre** :

| Ordre | Source | Emplacement |
|---|---|---|
| 1 | Instructions globales | `AGENTS.md` du dossier de données |
| 2 | Instructions importées activées | Documents repris d'un autre assistant |
| 3 | Règles externes sélectionnées | Règles importées, triées par source |
| 4 | Instructions de projet | `AGENTS.md` à la racine du projet |
| 5 | Instructions de projet | `.cl-go/AGENTS.md` dans le projet |
| 6 | Règles de projet | `.cl-go/rules/*.md`, triées par nom de fichier |

L'ensemble est précédé d'un en-tête qui indique au modèle qu'il doit suivre ces instructions à la lettre, et que **les instructions de projet, plus spécifiques, viennent après les globales**.

Chaque section est étiquetée par sa provenance, ce qui permet au modèle de distinguer une consigne globale d'une consigne de projet.

### 3. Où placer ses instructions

Recommandation à donner clairement, sinon on hésite entre six emplacements :

- **Ce qui vaut pour tout votre travail** → `AGENTS.md` du dossier de données, modifiable depuis l'application.
- **Ce qui vaut pour un projet** → `AGENTS.md` à la racine du projet. Il se versionne avec le code, donc il suit le dépôt et profite à toute l'équipe.
- **Des règles nombreuses sur un projet** → des fichiers séparés dans `.cl-go/rules/`, un par sujet. Ils sont lus par ordre alphabétique, ce qui permet de les numéroter pour fixer leur ordre.

### 4. La personnalité

Des fichiers Markdown rangés dans `memory/core/`, **activables individuellement**. Quatre existent à l'installation, **tous désactivés par défaut** :

| Fichier | Objet |
|---|---|
| `identity.md` | Qui est l'agent |
| `principles.md` | Ses principes de travail |
| `user.md` | Qui vous êtes |
| `idea-discovery.md` | Sa façon d'explorer des idées |

L'activation de chacun est enregistrée dans `personality-injection.json`. Seuls les fichiers activés sont injectés.

La section **Personnalité** de la navigation principale permet de les consulter et de les modifier.

### 5. La limite de taille

**200 Ko au total**, toutes sources confondues.

Quand la limite est atteinte, les instructions restantes sont omises et un avertissement est ajouté : « Additional selected instructions were omitted because the context limit was reached. »

Point important : **l'ordre détermine ce qui survit**. Les sources lues en premier — les instructions globales — passent ; celles de la fin — les règles de projet — sont les premières sacrifiées. Quelqu'un qui accumule des règles doit le savoir.

### 6. Écrire de bonnes instructions

Section de conseils, pas de description. Ce qui vaut la peine d'être dit :

- **Être impératif et précis.** « Utilise `npm test` avant de conclure » plutôt que « les tests sont importants ».
- **Donner la raison** quand elle n'est pas évidente. Une règle sans son pourquoi ne couvre que les cas prévus.
- **Ne pas dupliquer** entre le global et le projet : deux consignes qui se recouvrent finissent par se contredire sur un cas limite.
- **Rester bref.** Ces instructions occupent du contexte à chaque conversation, sur les 200 Ko disponibles.

---

## Encadrés

**Encadré « Qui gagne en cas de contradiction »**
> Les instructions de projet sont lues après les instructions globales, et le modèle est explicitement informé qu'elles sont plus spécifiques. En cas de contradiction, c'est le projet qui doit l'emporter.

**Encadré « AGENTS.md se versionne »**
> Placé à la racine d'un projet, `AGENTS.md` suit votre dépôt Git. Toute l'équipe travaille alors avec les mêmes consignes.

**Encadré « Personnalité désactivée par défaut »**
> Les quatre fichiers de personnalité existent mais sont inactifs à l'installation. Activez ceux qui vous intéressent depuis la section Personnalité.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Une consigne semble ignorée | Contredite par une source lue plus tard | Vérifier l'ordre d'assemblage |
| Une règle de projet n'a aucun effet | Limite de 200 Ko atteinte, les dernières sources sont omises | Alléger les instructions globales |
| Les fichiers de personnalité n'ont pas d'effet | Désactivés par défaut | Les activer dans la section Personnalité |
| Un fichier de `.cl-go/rules/` est ignoré | Seuls les fichiers `.md` sont lus | Renommer avec l'extension `.md` |
| L'ordre des règles de projet n'est pas celui voulu | Tri alphabétique | Préfixer par un numéro |
| Des instructions importées apparaissent sans avoir été écrites | Reprises d'un autre assistant lors de l'import | Les désactiver dans les réglages d'import |

---

## Renvois

- *Agent › Mémoire* — ce que l'agent retient de lui-même
- *Agent › Prompts système* — les instructions fournies par Beaver
- *Importer depuis un autre assistant* — l'origine des instructions importées
- *Référence › Stockage local*

---

## Points à confirmer

- **Le dossier `.cl-go/` porte l'ancien nom du projet.** Vérifier si un dossier `.beaver/` est également reconnu, ou s'il est prévu. Un utilisateur qui découvre Beaver ne devinera pas `.cl-go`.
- **L'édition de `AGENTS.md` depuis l'application** — quel fichier est modifié, le global ou celui du projet ?
- **Le contenu par défaut des quatre fichiers de personnalité.** Ils sont créés à l'installation ; leur contenu n'a pas été relevé. Nécessaire pour dire à quoi sert chacun.
- **Peut-on ajouter ses propres fichiers de personnalité** dans `memory/core/`, ou la liste est-elle figée à quatre ?
- **La sensibilité à la casse des noms de fichiers** (`AGENTS.md` contre `agents.md`), en particulier sous Windows et macOS où le système de fichiers ne distingue pas toujours.
- **Ce que voit l'utilisateur de l'assemblage final.** Peut-il consulter le texte réellement envoyé au modèle ? Ce serait le meilleur outil de diagnostic pour cette page.
