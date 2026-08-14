# Importer depuis un autre assistant

**Emplacement site** — Démarrage › Importer d'un autre agent (page prévue au sommaire du mockup)
**Répond à** — « J'utilise déjà Claude Code / Codex / un autre agent. Est-ce que je dois tout reconfigurer ? »
**Sources** — `src-tauri/src/services/agent_import/source_specs.rs`, `registry.rs`, `limits.rs`, `documents.rs`, `discovery.rs`, `rule_walker.rs`, `walker.rs`, `src/components/agent-import/`
**Vérification** — Vérifié dans le code : les neuf sources, leurs chemins de détection, les fichiers repris et toutes les limites

---

## Plan de page proposé

1. Ce que fait l'import
2. Les neuf assistants reconnus
3. Ce qui est repris
4. Comment ça se passe
5. Les sauvegardes
6. Les limites
7. Refaire ou annuler un import

---

## Contenu

### 1. Ce que fait l'import

L'import reprend **trois types de contenu** depuis une autre application d'agent installée sur la machine :

- le **document d'instructions** (le fichier que l'assistant lit à chaque conversation) ;
- les **règles** — des fichiers d'instructions rangés dans un dossier `rules/` ;
- les **skills** — des procédures rangées dans un dossier `skills/`.

Ce qui n'est **pas** repris : les conversations, les clés API, les connecteurs, les réglages d'interface. L'import porte sur les instructions, pas sur l'historique.

### 2. Les neuf assistants reconnus

Tableau complet en section Tableaux, avec les chemins réellement scrutés. C'est l'élément central de la page : un utilisateur veut savoir si son assistant est détecté et pourquoi il ne l'est pas.

Deux particularités à signaler :

- **Hermes Agent** n'a pas de document d'instructions repris — uniquement ses règles et ses skills.
- **OpenClaw** est le cas le plus complexe : plusieurs espaces de travail sont examinés, et le document retenu est le premier `AGENTS.md` trouvé.

Deux sources dépendent de **variables d'environnement**, ce qui explique une non-détection :

- **OpenCode** est cherché dans `$XDG_CONFIG_HOME/opencode`, ou `~/.config/opencode` si la variable n'est pas définie.
- **Kimi Code** est cherché dans `$KIMI_CODE_HOME`, ou `~/.kimi-code` par défaut. Trois emplacements complémentaires sont également examinés.

### 3. Ce qui est repris

- **Le document d'instructions** est copié dans le `AGENTS.md` de Beaver, avec une empreinte de la source pour détecter les modifications ultérieures.
- **Les règles et les skills** sont copiés dans les dossiers correspondants du dossier de données.
- Chaque document importé peut être **activé ou désactivé** individuellement sans être supprimé.

### 4. Comment ça se passe

1. L'assistant d'import **détecte** les applications présentes en cherchant leurs dossiers.
2. L'utilisateur **choisit** les sources à reprendre et ce qu'il veut de chacune.
3. Beaver **copie** le contenu retenu.
4. Une **sauvegarde** de l'état antérieur est conservée.

L'import est proposé pendant le parcours d'accueil, et reste disponible ensuite.

### 5. Les sauvegardes

- **Cinq sauvegardes au maximum sont conservées par document.**
- Elles vivent dans `agent-import-backups/` du dossier de données.
- Le registre de ce qui a été importé est dans `external-agent-sources.json`.

Point rassurant à mettre en avant : **un import ne détruit pas ce qui existait**. Le fichier d'instructions précédent est sauvegardé avant d'être remplacé.

### 6. Les limites

Toutes vérifiées dans le code. Elles ne sont pas décoratives : un dossier de skills volumineux sera tronqué, et l'utilisateur doit savoir pourquoi.

| Limite | Valeur |
|---|---|
| Sources | 9 |
| Emplacements examinés par source | 8 |
| Skills par source | 512 |
| Skills au total | 2 048 |
| Règles par source | 256 |
| Profondeur d'exploration | 12 niveaux |
| Entrées examinées | 10 000 |
| Taille d'un document d'instructions | 256 Ko |
| Taille d'un manifeste | 256 Ko |
| Sauvegardes par document | 5 |
| Taille du registre | 1 Mo |

### 7. Refaire ou annuler un import

Voir *Points à confirmer* — les mécanismes exacts de réimport et de retour arrière n'ont pas été vérifiés dans le détail.

---

## Tableaux

### Tableau — Les neuf assistants et leurs emplacements

| Assistant | Identifiant | Dossier cherché | Document repris | Règles | Skills |
|---|---|---|---|---|---|
| Claude Code | `claude` | `~/.claude` | `CLAUDE.md` | `rules/` | `skills/` |
| Codex | `codex` | `~/.codex` | `AGENTS.md` | `rules/` | `skills/` |
| Agents | `agents` | `~/.agents` | `AGENTS.md` | `rules/` | `skills/` |
| Hermes Agent | `hermes` | `~/.hermes` | **Aucun** | `rules/` | `skills/` |
| Qwen Code | `qwen` | `~/.qwen` | `QWEN.md` | `rules/` et `output-language.md` | `skills/` |
| ZCode | `zcode` | `~/.zcode` | `AGENTS.md` | `rules/` | `skills/` |
| OpenClaw | `openclaw` | `~/.openclaw` et ses espaces de travail | `AGENTS.md` du premier espace qui en contient un | `rules/` de chaque espace | `skills/` et `.agents/skills` de chaque espace |
| OpenCode | `opencode` | `$XDG_CONFIG_HOME/opencode`, sinon `~/.config/opencode` | `AGENTS.md` | `rules/` | `skills/` |
| Kimi Code | `kimi` | `$KIMI_CODE_HOME`, sinon `~/.kimi-code` ; plus `~/.kimi`, `~/.kimi-webbridge`, `~/.kimi-work` | `AGENTS.md` | `rules/` | `skills/` et `~/.kimi/skills` |

---

## Encadrés

**Encadré « Rien n'est écrasé sans sauvegarde »**
> Beaver conserve jusqu'à cinq versions antérieures de chaque document importé. Votre fichier d'instructions précédent n'est pas perdu.

**Encadré « Ce qui n'est pas importé »**
> L'import reprend vos instructions, vos règles et vos skills. Il ne reprend ni vos conversations, ni vos clés API, ni vos connecteurs.

**Encadré « Assistant non détecté »** — à placer près du tableau.
> La détection repose sur la présence d'un dossier à un emplacement précis. Si votre assistant est installé ailleurs — notamment via une variable d'environnement pour OpenCode et Kimi Code — il ne sera pas trouvé.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Un assistant installé n'apparaît pas | Dossier absent de l'emplacement attendu | Vérifier le chemin dans le tableau ; pour OpenCode et Kimi Code, vérifier la variable d'environnement |
| Tous les skills n'ont pas été repris | Plafond de 512 par source ou 2 048 au total | Copier manuellement ce qui manque dans le dossier `skills/` |
| Une arborescence profonde est ignorée | Exploration limitée à 12 niveaux et 10 000 entrées | Aplatir l'arborescence, ou copier manuellement |
| Un document volumineux est refusé | Plafond de 256 Ko | Alléger le fichier avant import |
| Hermes ne propose pas de document | Aucun document n'est défini pour cette source | Normal : seules ses règles et ses skills sont reprises |

---

## Renvois

- *Parcours d'accueil* — où l'import est proposé
- *Agent › Personnalité et AGENTS.md* — ce que devient le document importé
- *Agent › Skills locaux* — comment les skills importés sont utilisés
- *Référence › Stockage local* — l'emplacement des sauvegardes

---

## Points à confirmer

- **Le réimport.** Que se passe-t-il si on relance l'import d'une source déjà importée ? Écrasement, fusion, doublon ? Le registre conserve une empreinte de la source, ce qui suggère une détection de modification, mais le comportement n'a pas été vérifié.
- **L'annulation d'un import.** Les sauvegardes existent ; le chemin de restauration côté interface n'a pas été vérifié.
- **L'emplacement de l'assistant d'import dans les réglages** après le parcours d'accueil.
- **Le sort des skills en conflit de nom** entre deux sources importées.
- **Ce que devient le document d'instructions de Beaver** quand plusieurs sources en fournissent un : concaténation, dernier gagnant, choix de l'utilisateur ? À élucider, c'est la question que se posera tout utilisateur de deux assistants.
- **Les espaces de travail OpenClaw.** La logique de découverte mérite une vérification sur une installation réelle avant d'être décrite plus précisément.
