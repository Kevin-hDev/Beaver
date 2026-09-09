# Skills

**Emplacement site** — Agent › Skills (le mockup les regroupe avec la mémoire ; ils méritent leur page)
**Répond à** — « Comment j'apprends à l'agent une procédure qu'il doit suivre, sans la répéter à chaque fois ? »
**Sources** — `src-tauri/src/services/agent_local/skill_catalog.rs` (lignes 8-9, 17-42, 141-165), `skill_parser.rs` (lignes 7-8), `skill_limits.rs`, `src-tauri/src/services/skill_manifest_policy.rs` (ligne 3), `tool_skill_loader.rs` (lignes 4-5, 19-27), `models/agent_turn_contract.rs` (ligne 12), `subagent_tool_profile.rs` (lignes 46-48), `src/hooks/use-active-skills.ts`, `src/components/agent-local/slash-autocomplete.tsx`, `src-tauri/src/services/agent_import/` (skills importés)
**Vérification** — Vérifié dans le code, revérifié le 9 septembre 2026 : format du fichier, noms acceptés, limites, tri, sources, invocation par commande

---

## Plan de page proposé

1. Ce qu'est un skill
2. Où les ranger
3. Le format d'un skill
4. Comment l'agent s'en sert
5. Les skills importés
6. Les limites
7. Écrire un bon skill

---

## Contenu

### 1. Ce qu'est un skill

Un mode d'emploi rangé dans un dossier, que l'agent **charge à la demande** quand la tâche y correspond.

La différence avec `AGENTS.md` est le point central de la page :

| | AGENTS.md | Skill |
|---|---|---|
| Chargement | À chaque conversation, toujours | À la demande, quand c'est pertinent |
| Coût en contexte | Permanent | Seulement quand il est chargé |
| Contenu | Conventions courtes, interdits | Procédures détaillées |

Autrement dit : ce qui doit toujours être vrai va dans `AGENTS.md` ; ce qui ne sert que parfois va dans un skill. Une procédure de déploiement de trois pages n'a pas à occuper le contexte de toutes vos conversations.

### 2. Où les ranger

Un **dossier par skill**, dans le dossier `skills/` du répertoire de données.

Le dossier peut contenir d'autres fichiers que le manifeste — scripts, gabarits, références — auxquels le skill renvoie.

### 3. Le format d'un skill

Le dossier doit contenir un fichier nommé **`SKILL.md`** ou **`skill.md`** — les deux graphies sont acceptées.

Il commence par un en-tête encadré de `---` contenant deux champs :

```markdown
---
name: Nom du skill
description: Ce que fait ce skill et quand l'employer.
---

Le contenu du skill, en Markdown libre.
```

- `name` — le nom affiché. À défaut, le nom du dossier est employé.
- `description` — **250 caractères au maximum**. C'est ce que l'agent lit pour décider s'il doit charger le skill.

Les valeurs entre guillemets simples ou doubles sont acceptées.

**Sans en-tête**, le fichier reste utilisable : le nom du dossier sert de nom. Mais l'absence de description prive l'agent du critère de sélection.

### 4. Comment l'agent s'en sert

Deux temps :

1. **Le catalogue** — l'agent voit en permanence la liste des skills disponibles, avec leur nom et leur description. C'est peu coûteux en contexte.
2. **Le chargement** — quand une tâche correspond à une description, l'agent charge le contenu complet du skill avec l'outil `load_skill`.

D'où l'importance de la description : c'est le seul élément sur lequel l'agent s'appuie pour décider. Une description vague donne un skill qui ne se déclenche jamais.

L'outil `load_skill` appartient au groupe **Skills**, actif par défaut.

**Vous pouvez aussi appeler un skill vous-même.** Chaque skill reçoit une **commande** construite depuis sa source et son nom. Dans la zone de saisie, taper `/` ouvre une liste d'autocomplétion des skills disponibles ; en choisir un insère sa commande dans le message.

- Un skill **local** s'invoque par `/nom-du-skill`.
- Un skill **importé** s'invoque par `/source:nom`.
- Quand deux skills aboutiraient à la même commande, un suffixe tiré de leur identifiant les distingue — les commandes sont rendues uniques après le tri. Les identifiants, eux, restent distincts en toutes circonstances puisqu'ils intègrent la source.

**Les sous-agents n'y ont presque pas accès** : seul un sous-agent de type **codeur** reçoit `load_skill`, et seulement si le groupe Skills est activé. Un sous-agent explorateur n'a jamais accès aux skills. Voir `05-outils/sous-agents-outils.md`.

### 5. Les skills importés

Les skills reprises d'un autre assistant apparaissent dans le même catalogue, avec l'indication de leur **source d'origine**.

Les skills sont triés par nom, sans distinction de casse.

**Une troisième source existe, invisible dans le catalogue : les skills d'extension.** Leur identifiant est préfixé par `extension:`, et ils sont **délibérément retirés du catalogue global** — le code le dit sans ambiguïté : cet espace de noms est autorisé session par session et ne doit jamais entrer dans le catalogue global. Ils ne se comptent donc pas dans les 2 048 skills locaux et n'apparaissent pas dans la liste permanente.

Sans conséquence directe aujourd'hui pour qui n'installe pas d'extension, mais la page doit dire que le catalogue affiché n'est pas la totalité des skills que l'agent peut charger.

### 6. Les limites

| Limite | Valeur |
|---|---|
| Skills locaux | **2 048** |
| Taille d'un fichier de skill | **256 Ko** |
| En-tête lu | **32 Ko** |
| Description | **250 caractères** |
| Nom affiché | **120 caractères** |
| Identifiant de skill | 768 octets |

**Contraintes sur l'identifiant** : ni `..`, ni `/`, ni `\`. Un identifiant non conforme est rejeté — c'est une protection contre la lecture d'un fichier hors du dossier des skills.

Trois erreurs possibles au chargement : identifiant invalide, skill introuvable, skill indisponible.

### 7. Écrire un bon skill

Conseils, pas description :

- **Soigner la description avant le contenu.** C'est elle qui déclenche le chargement. Y écrire à quoi sert le skill **et quand l'employer**, avec les mots que vous emploieriez naturellement.
- **Écrire à l'impératif.** « Tu vérifies X » plutôt que « il faudrait vérifier X » : une consigne à l'infinitif se lit comme une suggestion et se saute.
- **Donner la raison d'une règle** quand elle n'est pas évidente. Une règle sans son pourquoi ne couvre que les cas prévus.
- **Un skill, une procédure.** Deux procédures dans un même skill, et l'agent charge les deux pour n'en employer qu'une.

---

## Encadrés

**Encadré « La description décide de tout »**
> L'agent choisit de charger un skill à partir de sa description seule, limitée à 250 caractères. Précisez ce que fait le skill **et quand l'employer** : une description vague donne un skill qui ne se déclenche jamais.

**Encadré « Appeler un skill soi-même »**
> Tapez `/` dans la zone de saisie : la liste des skills disponibles s'ouvre, et en choisir un insère sa commande dans votre message. Un skill local répond à `/nom-du-skill`, un skill importé à `/source:nom`.

**Encadré « Skill ou AGENTS.md ? »**
> Ce qui doit toujours s'appliquer va dans `AGENTS.md`. Ce qui ne sert que dans certaines situations va dans un skill : il n'occupe du contexte que lorsqu'il est chargé.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Un skill n'est jamais chargé | Description absente ou trop vague | La réécrire en précisant les cas d'emploi |
| Un skill n'apparaît pas dans le catalogue | Pas de `SKILL.md` ni `skill.md` dans le dossier | Ajouter le fichier |
| Le nom affiché est celui du dossier | Champ `name` absent de l'en-tête | L'ajouter |
| « Skill introuvable » | Dossier renommé ou supprimé | Vérifier le contenu de `skills/` |
| « Identifiant de skill invalide » | Identifiant contenant `..`, `/` ou `\` | Protection volontaire |
| Une partie du skill semble ignorée | Fichier au-delà de 256 Ko | Le scinder en plusieurs skills |
| Des skills inconnus apparaissent | Reprises lors de l'import d'un autre assistant | Leur source est indiquée dans le catalogue |
| La commande d'un skill porte un suffixe illisible | Deux skills aboutissaient à la même commande | Normal : le suffixe les distingue ; renommer l'un des deux pour retrouver une commande courte |
| Un sous-agent n'utilise aucun skill | Seul le codeur y a droit, et seulement si le groupe Skills est activé | Vérifier le groupe Skills, ou confier la tâche à l'agent principal |

---

## Renvois

- *Agent › Instructions permanentes* — la différence avec `AGENTS.md`
- *Agent › Contexte* — les skills disponibles occupent une catégorie propre
- *Outils › Skills et automatisations* — l'outil `load_skill`
- *Importer depuis un autre assistant* — l'origine des skills importés
- *Référence › Stockage local*

---

## Points à confirmer

- **Les skills sont-ils gérables depuis l'interface**, ou seulement en déposant des dossiers ? Détermine la forme de la page : procédure guidée, ou mode d'emploi de fichiers.
- **Le rechargement à chaud.** Un événement de changement des skills existe dans la surveillance de fichiers. Confirmer qu'ajouter un dossier est pris en compte sans redémarrer.
- **Un skill peut-il en charger un autre ?** Non vérifié.
- **L'aspect de la liste d'autocomplétion** ouverte par `/` : ce qu'elle affiche pour chaque skill, et le comportement quand aucun skill ne correspond. Non relevé à l'écran.
- **Les skills d'extension** sont autorisés session par session. Le mécanisme d'autorisation lui-même — qui l'accorde, à quel moment, ce que voit l'utilisateur — n'a pas été lu. À traiter avec la section Extensions.
