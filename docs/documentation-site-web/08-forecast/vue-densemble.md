# L'espace Forecast — vue d'ensemble

**Emplacement site** — Forecast › Vue d'ensemble
**Répond à** — « À quoi sert Forecast, où est-ce que ça se trouve dans Beaver, et qu'est-ce qu'il me faut pour obtenir ma première prévision ? »
**Sources** — `src/components/agent-local/mode-selector.tsx`, `src/components/agent-local/use-agent-local-forecast-content.tsx`, `src/types/forecast-panel.ts`, `src/hooks/use-forecast-panel.ts` ; `src/components/forecast/forecast-panel.tsx`, `forecast-empty.tsx`, `forecast-data.ts`, `forecast-config.tsx`, `forecast-nav.tsx`, `forecast-section-router.tsx`, `forecast-errors.ts`, `forecast-limits.ts` ; `src/components/forecast/workbench/open-forecast-workbench.ts`, `forecast-workbench-nav.tsx`, `forecast-workbench-section.tsx`, `forecast-workbench-types.ts` ; `src-tauri/src/commands/forecast.rs`, `commands/forecast_models.rs` ; `src-tauri/src/services/forecast/limits.rs`, `storage_paths.rs`, `storage_index.rs`, `catalog.rs`, `catalog_specs/providers.rs` ; `src-tauri/src/services/agent_local/tool_definitions_forecast.rs` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026. Aucun écran n'a été observé : tous les points d'affichage sont listés en fin de fichier pour la passe d'interface.

> **Cette page est la porte d'entrée de la section Forecast.** Elle décrit à quoi sert l'espace, où il se trouve et le parcours complet d'une prévision. Chaque étape a ensuite sa page : les données (`08-forecast/donnees-et-audit.md`), le choix du modèle (`08-forecast/selection-du-modele.md`), les exports (`08-forecast/exports.md`), les modèles eux-mêmes (`08-forecast/modeles-locaux.md`, `08-forecast/modele-cloud-timegpt.md`).

---

## Plan de page proposé

1. À quoi sert Forecast
2. Où ça se trouve dans Beaver
3. Ce qu'il faut avant de commencer
4. Le parcours, de bout en bout
5. Les deux surfaces : le panneau et la fenêtre d'espace de travail
6. Ce que Beaver garde après le calcul
7. Le même travail depuis la conversation

---

## Contenu

### 1. À quoi sert Forecast

Forecast prend une **série de valeurs datées** — un chiffre d'affaires mois par mois, une consommation heure par heure, un nombre de commandes par jour — et prolonge cette série dans le futur sur un nombre de périodes choisi.

Trois choses le distinguent d'un simple prolongement de courbe, et toutes les trois sont dans le code :

- Le résultat n'est pas une seule courbe mais une **fourchette**. Chaque point prévu est accompagné de trois valeurs — bas, milieu, haut — dérivées du niveau de confiance demandé (`services/forecast/export/quantile_labels.rs:11-31`). Seuls des modèles capables de produire cette fourchette sont acceptés : la capacité `probabilistic` est exigée de tout modèle candidat (`services/forecast/auto_selection_candidate.rs:85`).
- Les données sont **auditées avant le calcul**, et un défaut bloquant arrête la prévision au lieu de produire un résultat trompeur (`commands/forecast.rs:37` → `services/forecast/data_quality/audit.rs:29-36`). Voir `08-forecast/donnees-et-audit.md`.
- Le résultat est **enregistré** avec ses données d'entrée, son profil de données et la trace du modèle choisi, ce qui permet de le rouvrir, de le comparer, de l'annoter et de l'exporter plus tard (`commands/forecast.rs:92-107`).

### 2. Où ça se trouve dans Beaver

Forecast n'est **pas un onglet séparé** de l'application. C'est l'un des modes du **panneau latéral partagé** d'une conversation Agent.

Le sélecteur de mode de ce panneau propose (`src/components/agent-local/mode-selector.tsx:89-107`) :

| Libellé affiché | Clé i18n | Toujours proposé |
|---|---|---|
| **Aperçu fichier** | `forecast.panelMode.preview` | Oui — c'est le mode par défaut (`src/types/forecast-panel.ts:17`) |
| **Forecast** | `forecast.panelMode.forecast` | Oui |
| **Navigateur** | `browser.title` | Non — l'entrée disparaît quand le navigateur intégré est indisponible (`mode-selector.tsx:97`) |

Le bouton qui ouvre ce sélecteur porte l'infobulle **« Mode du panneau »** (`fr.json`, `forecast.panelMode.title`).

**Conséquence à écrire clairement sur le site** : le mode choisi, la section ouverte et l'analyse affichée sont **attachés à la conversation**, pas à l'application. Chaque conversation retrouve son propre état de panneau (`src/hooks/use-agent-session-workspace.ts:23`, `:60` ; `src/types/forecast-panel.ts:28-44`). Ouvrir une autre conversation ne change pas ce que la première affichait.

### 3. Ce qu'il faut avant de commencer

Trois conditions, et une seule est vraiment bloquante.

**Un fichier de données.** Le sélecteur de fichier de l'écran d'accueil accepte six extensions : `csv`, `tsv`, `xlsx`, `xls`, `ods`, `xlsm` (`src/components/forecast/forecast-empty.tsx:32`). Le détail de ce qu'il doit contenir est dans `08-forecast/donnees-et-audit.md`.

**Un modèle utilisable.** Deux voies, et il faut au moins l'une des deux :

- un **modèle local installé** — il se télécharge depuis l'écran des modèles Forecast, et le calcul refuse de démarrer tant qu'il ne l'est pas (`commands/forecast.rs:73-75`, message « Modèle non installé ») ;
- un **modèle distant configuré** — le seul fournisseur du catalogue qui demande une clé API est **Nixtla (TimeGPT-2)** (`services/forecast/catalog_specs/providers.rs:20-27`, `requires_api_key: true`). Sans clé enregistrée, le calcul s'arrête avec « Clé API Nixtla non configurée » (`commands/forecast.rs:67-68`).

Tous les autres fournisseurs du catalogue — Amazon Chronos, Google TimesFM, Datadog Toto, Salesforce MOIRAI, IBM FlowState, PriorLabs TabPFN-TS — sont marqués `requires_api_key: false` et tournent en local (`catalog_specs/providers.rs`). Le détail est dans `08-forecast/modeles-locaux.md` et `08-forecast/modele-cloud-timegpt.md`.

**De la place mémoire pour un modèle local.** Avant de lancer un modèle local, Beaver vérifie que la machine peut le porter et refuse sinon, avec « Ressources insuffisantes pour ce modèle » (`commands/forecast.rs:76` → `services/forecast/hardware_profile.rs:99-114`). Le mécanisme est décrit dans `08-forecast/selection-du-modele.md`.

### 4. Le parcours, de bout en bout

Sept étapes. Les cinq premières sont visibles, les deux dernières se font toutes seules.

**Étape 1 — L'écran d'accueil.** Tant qu'aucune analyse n'est ouverte, le panneau affiche **« Aucune analyse en cours »** et, en sous-titre, **« Demander à l'agent »** (`forecast-empty.tsx:48-49`). Trois boutons suivent :

| Bouton | État |
|---|---|
| **Importer un fichier (CSV, Excel)** | Actif (`forecast-empty.tsx:51-58`) |
| **Coller des données** | **Désactivé**, infobulle **« Bientôt disponible »** (`forecast-empty.tsx:59-63`) |
| **Depuis une URL** | **Désactivé**, infobulle **« Bientôt disponible »** (`forecast-empty.tsx:64-68`) |

Sous ces boutons, quand il y en a, la liste **« Analyses récentes »** permet de rouvrir une analyse déjà calculée, avec son nom, son modèle, son horizon et son erreur moyenne quand elle existe (`forecast-empty.tsx:70-83`).

**À écrire honnêtement sur le site** : aujourd'hui, la seule entrée de données par l'interface est l'import de fichier. Les deux autres boutons sont présents et inactifs.

**Étape 2 — La lecture du fichier.** Le fichier est lu et converti en tableau de lignes. Cette lecture applique déjà des bornes strictes, avant même l'audit : au plus **5 000 lignes**, **256 colonnes**, **80 caractères** par nom de colonne, **32 768 caractères** par cellule, et **5 Mo** de données une fois converties (`src/components/forecast/forecast-limits.ts:1-6`, `forecast-data.ts:42-73`). Un dépassement affiche **« Import impossible. Vérifie le format du fichier. »** (`forecast-panel.tsx:95` ; `fr.json`, `forecast.errors.importFailed`).

Deux gestes utiles à mentionner, faits automatiquement à ce moment : une colonne sans en-tête reçoit un nom de repli `column_1`, `column_2`… (`forecast-data.ts:107`), et un nombre écrit avec une virgule décimale est reconnu comme un nombre (`forecast-data.ts:86-98`).

**Étape 3 — L'écran « Configuration ».** Huit réglages, avec leurs valeurs de départ (`forecast-config.tsx:47-53`, `:96-198`) :

| Réglage | Libellé affiché | Valeur de départ |
|---|---|---|
| Colonne à prévoir | **Cible** | La 2ᵉ colonne du fichier |
| Colonne des dates | **Colonne date** | La 1ʳᵉ colonne du fichier |
| Colonne qui sépare plusieurs séries | **Colonne série** | Aucune — l'option affiche **« Aucune colonne série »** |
| Colonnes explicatives | **Covariables** | Aucune ; proposées sous forme de pastilles cliquables |
| Nombre de périodes à prévoir | **Horizon** | **12** |
| Pas de temps | **Fréquence** | **Mois** |
| Modèle | **Modèle** | Voir `08-forecast/selection-du-modele.md` |
| Largeur de la fourchette | **Confiance** | **80 %** |

Les huit fréquences proposées sont **Jour**, **Jour ouvré**, **Semaine**, **Mois**, **Trimestre**, **Année**, **Heure**, **Minute** (`forecast-limits.ts:8` ; libellés dans `fr.json`, `forecast.frequency.*`).

Le bouton de lancement porte **« Lancer le forecast »**, et **« Calcul en cours… »** pendant le travail (`forecast-config.tsx:197`). Il reste inactif tant qu'une cible, une colonne de date, un modèle et un horizon supérieur à zéro ne sont pas tous renseignés et qu'aucune incohérence n'est détectée (`forecast-config.tsx:91`).

**Étape 4 — Les vérifications avant calcul.** Dans l'ordre exact du code (`commands/forecast.rs:28-64`) : normalisation de la demande, lecture de la politique de sélection, application de cette politique, relecture du fichier si besoin, validation de la demande, **audit des données**, puis vérification que le modèle demandé est bien un candidat acceptable. Un défaut bloquant à n'importe laquelle de ces étapes arrête tout.

**Étape 5 — Le calcul.** Pour un modèle distant, Beaver récupère la clé Nixtla et interroge le service (`commands/forecast.rs:66-71`). Pour un modèle local, il vérifie l'installation et les ressources, démarre le moteur de prédiction, calcule, puis **programme l'arrêt du moteur après inactivité** (`commands/forecast.rs:72-90`).

**Étape 6 — L'enregistrement.** Le résultat reçoit sa trace d'origine — quel modèle, sur quelles données, pour quelle raison, en combien de temps (`commands/forecast.rs:92-101`) — puis le profil de données et l'analyse sont écrits sur le disque (`:103-106`).

**Étape 7 — L'affichage.** Le panneau bascule sur la vue principale de la nouvelle analyse (`src/hooks/use-forecast-panel.ts:31-37`).

En cas d'échec, un seul message générique s'affiche : **« Le calcul a échoué. Vérifie les colonnes, le modèle et la clé API. »** — sauf pour un cas nommé, le stockage plein, qui affiche **« Le stockage Forecast est plein. Supprime d'anciennes analyses puis réessaie. »** (`forecast-errors.ts:3-10` ; `fr.json`, `forecast.errors.launchFailed` et `forecast.errors.capacityReached`).

### 5. Les deux surfaces : le panneau et la fenêtre d'espace de travail

C'est le point de structure le plus important de la page, et il n'est pas évident à l'usage.

**Le panneau latéral** est étroit et volontairement resserré. Il ne propose que **trois** sections (`src/types/forecast-panel.ts:1`, `forecast-nav.tsx:9-13`, `forecast-section-router.tsx:20-29`) :

| Libellé affiché | Contenu |
|---|---|
| **Vue principale** | Le graphique de la prévision et ses couches |
| **Comparaisons** | La mise en regard de plusieurs analyses |
| **Historique** | La liste des analyses enregistrées |

**La fenêtre « Espace Forecast »** est une **fenêtre séparée**, redimensionnable, qui retrouve sa position d'une fois sur l'autre (`workbench/open-forecast-workbench.ts:33-45`). Elle propose **sept** sections (`workbench/forecast-workbench-nav.tsx:4-12`, `forecast-workbench-section.tsx:32-54`), avec leur description telle qu'écrite dans l'application (`fr.json`, `forecast.workbench.sectionDescriptions`) :

| Libellé affiché | Description dans l'application |
|---|---|
| **Données** | « Inspecter les données, leur mapping et les contrôles de qualité. » |
| **Prévision** | « Consulter la prévision complète et ses différentes couches. » |
| **Évaluation** | « Mesurer les modèles avec des baselines et des validations temporelles. » |
| **Comparaison** | « Comparer la qualité, la vitesse et les contraintes des modèles. » |
| **Scénarios** | « Créer et modifier des hypothèses sans perdre la prévision de référence. » |
| **Notes** | « Créer, organiser et consulter les notes liées à cette prévision. » |
| **Rapport** | « Rassembler la provenance, les notes et les exports avancés. » |

**Une seule fenêtre à la fois** : rouvrir l'espace alors qu'il est déjà ouvert le ramène au premier plan au lieu d'en créer un second (`open-forecast-workbench.ts:26-32`). L'analyse affichée dans la fenêtre suit celle du panneau, synchronisée à chaque changement (`use-agent-local-forecast-content.tsx:46-53`).

**Le message à faire passer** : le panneau sert à regarder pendant qu'on discute avec l'agent ; l'espace sert à travailler. Tout ce qui demande de la place — les données brutes, les contrôles de qualité, l'évaluation, les scénarios, le rapport — vit dans la fenêtre.

Une **documentation technique intégrée** existe par ailleurs, dans sa propre fenêtre, sous le titre **« Documentation Forecast »** (`use-agent-local-forecast-content.tsx:66-69` ; `fr.json`, `forecast.docs`). Elle couvre treize sujets, dont *Vue d'ensemble*, *Datasets*, *Modèles*, *Évaluation*, *Incertitude*, *Covariables*, *Multi-séries*, *Scénarios* et *Limites*.

### 6. Ce que Beaver garde après le calcul

| Ce qui est gardé | Où | Borne |
|---|---|---|
| Les analyses calculées | `~/.local/share/cl-go-dash/forecast-analyses/<id>.json` | **500 analyses** au total, la plus ancienne du même espace de travail étant remplacée au-delà (`limits.rs:19`, `storage_index.rs:181-189`) |
| L'index des analyses | `forecast-analyses/index.json` | **2 Mo** (`limits.rs:16`, `storage_paths.rs:27-29`) |
| Le profil de données | Un dossier par espace de travail : `forecast-data-profiles-project-<id>/` ou `forecast-data-profiles-session-<id>/` | **20 profils** par espace, les plus anciens supprimés au-delà (`limits.rs:18`, `data_profiles.rs:48-53`, `:126-133`) |
| La politique de sélection du modèle | `forecast-selection-policy.json` | **4 Ko** (`selection_policy.rs:9`, `:77-79`) |
| Les notes attachées à une analyse | `forecast-notes/` | Voir `08-forecast/scenarios-notes-rapports.md` |

Une analyse enregistrée pèse au plus **64 Mo** (`limits.rs:15`).

**Point à énoncer sur le site** : les analyses sont rangées **par espace de travail** — un projet ou une conversation. Ce qui a été calculé dans une conversation n'apparaît pas dans la liste d'une autre (`storage_index.rs:150-158`, champ `workspace`).

### 7. Le même travail depuis la conversation

Le sous-titre de l'écran vide, **« Demander à l'agent »**, n'est pas décoratif : l'agent dispose de sept outils Forecast et peut faire tout le parcours à votre place (`services/agent_local/tool_definitions_forecast.rs:12-131`) :

| Outil | Ce qu'il fait |
|---|---|
| `forecast_data_audit` | Contrôle les données et fixe un profil réutilisable |
| `forecast_models` | Inspecte la politique de sélection et propose les modèles utilisables |
| `forecast_run` | Lance la prévision |
| `forecast_backtest` | Mesure les modèles sur des fenêtres passées |
| `forecast_compare_models` | Relit le classement obtenu |
| `forecast_analyze` | Ajoute une annotation, un scénario, un ensemble |
| `forecast_read` | Relit une analyse enregistrée |

Le résultat produit par l'agent est **la même analyse** que celle produite par l'écran : elle s'ouvre dans le panneau, s'exporte et s'annote de la même façon.

---

## Tableaux

### Les limites d'entrée, en un coup d'œil

| Ce qui est borné | Valeur | Source |
|---|---|---|
| Lignes du fichier | **5 000** | `limits.rs:3` |
| Colonnes | **256** | `limits.rs:4` |
| Caractères par cellule | **32 768** | `limits.rs:5` |
| Caractères par nom de colonne | **80** | `limits.rs:6` |
| Covariables | **64** | `limits.rs:7` |
| Séries distinctes | **256** | `limits.rs:9` |
| Horizon | **5 000** périodes | `limits.rs:10` |
| Points prévus au total (séries × horizon) | **100 000** | `limits.rs:11`, `:64-75` |
| Taille du fichier ouvert | **50 Mo** | `limits.rs:2` |
| Données converties, envoyées au calcul | **5 Mo** | `limits.rs:1` |

---

## Encadrés

> **ℹ À placer en tête de page — Forecast en quatre phrases.**
> Forecast prolonge une série de valeurs datées et rend une fourchette, pas une seule courbe. Il vit dans le panneau latéral d'une conversation Agent, à côté de l'aperçu de fichier, et se déplie dans une fenêtre séparée quand il faut de la place. Vos données sont contrôlées avant le calcul : un défaut bloquant arrête la prévision au lieu de produire un résultat trompeur. Le résultat est enregistré avec ses données et sa trace d'origine, et se rouvre, s'annote et s'exporte plus tard.

> **ℹ À placer dans « Où ça se trouve » — Forecast appartient à la conversation.**
> Le mode du panneau, la section ouverte et l'analyse affichée sont mémorisés **par conversation**. Vous pouvez laisser une prévision ouverte dans un fil pendant que vous travaillez sur autre chose dans un autre.

> **⚠ À placer dans « Ce qu'il faut avant de commencer » — Sans modèle, pas de prévision.**
> Il faut soit un modèle local installé, soit une clé API Nixtla pour le modèle distant. Le calcul s'arrête proprement dans les deux cas manquants, avec « Modèle non installé » ou « Clé API Nixtla non configurée ».

> **ℹ À placer dans « Le parcours » — Aujourd'hui, l'import de fichier est la seule entrée par l'écran.**
> « Coller des données » et « Depuis une URL » sont présents mais inactifs, avec l'infobulle « Bientôt disponible ». Passer par l'agent est la seconde voie disponible.

> **ℹ À placer dans « Les deux surfaces » — Le panneau montre, la fenêtre travaille.**
> Le panneau n'a que trois sections : Vue principale, Comparaisons, Historique. Les données brutes, les contrôles de qualité, l'évaluation, les scénarios et le rapport sont dans la fenêtre « Espace Forecast ».

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Je ne trouve pas Forecast dans l'application » | Ce n'est pas un onglet : c'est un mode du panneau latéral d'une conversation | Ouvrir une conversation, ouvrir le panneau, choisir **Forecast** dans « Mode du panneau » |
| « Import impossible. Vérifie le format du fichier. » | Extension non reconnue, ou dépassement d'une borne de lecture (5 000 lignes, 256 colonnes, 5 Mo convertis) | Réduire le fichier ou l'enregistrer dans un des six formats acceptés |
| « Le calcul a échoué. Vérifie les colonnes, le modèle et la clé API. » | Message unique couvrant toutes les causes de refus | Ouvrir la section **Données** de la fenêtre Espace Forecast pour lire les alertes de qualité ; voir `08-forecast/donnees-et-audit.md` |
| « Le stockage Forecast est plein. » | **500 analyses** enregistrées, et aucune de cet espace de travail à remplacer | Supprimer d'anciennes analyses depuis **Historique** |
| « Mes analyses ont disparu » | Les analyses sont rangées par espace de travail | Rouvrir la conversation ou le projet dans lequel elles ont été calculées |
| « Je ne vois que trois sections » | Le panneau en expose trois ; les sept autres sont dans la fenêtre | Ouvrir **Espace Forecast** |
| Deux fenêtres Espace Forecast attendues | Une seule est possible | Le second appel ramène la fenêtre existante au premier plan |

---

## Renvois

- `08-forecast/donnees-et-audit.md` — ce que doit contenir le fichier et ce que Beaver contrôle
- `08-forecast/selection-du-modele.md` — Manuel, Auto, et ce que le code regarde vraiment
- `08-forecast/exports.md` — les sept formats de sortie
- `08-forecast/modeles-locaux.md` et `08-forecast/modele-cloud-timegpt.md` — le catalogue et l'installation
- `08-forecast/evaluation-et-comparaison.md` — mesurer un modèle sur des fenêtres passées
- `08-forecast/analyse-avancee.md` — décomposition, anomalies, importance des variables, dérive
- `08-forecast/scenarios-notes-rapports.md` — hypothèses, notes et rapport
- `03-interface/vue-densemble.md` — le panneau latéral et les autres modes
- `12-reference/emplacement-des-donnees.md` — l'inventaire de `~/.local/share/cl-go-dash/`

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **Deux clés de navigation inutilisées.** `fr.json` définit `forecast.nav.scenarios` (« Scénarios ») et `forecast.nav.notes` (« Notes »), mais la navigation du panneau ne comporte que trois entrées (`forecast-nav.tsx:9-13`) et le type `ForecastSection` n'en connaît que trois (`src/types/forecast-panel.ts:1`). Ces deux libellés ne s'affichent nulle part. Sans conséquence pour l'utilisateur, mais à signaler à l'équipe : soit ce sont des restes, soit une entrée manque.
2. **`CLAUDE.md` cite `forecast-selected-model.json` comme fichier de données.** Ce fichier n'est plus l'autorité : la politique de sélection vit dans **`forecast-selection-policy.json`**, et l'ancien fichier n'est plus lu qu'une fois, pour être migré (`services/forecast/selection_policy.rs:77-83`, `:93-107`). À corriger dans la documentation interne du dépôt ; sans effet sur la page du site.
3. **Les dossiers de profils de données par espace de travail ne figurent pas dans l'inventaire de `CLAUDE.md`** : `forecast-data-profiles-project-<id>/` et `forecast-data-profiles-session-<id>/` (`data_profiles.rs:126-133`). À ajouter à `12-reference/emplacement-des-donnees.md`.
4. **Le message d'échec de calcul est unique** pour toutes les causes sauf le stockage plein (`forecast-errors.ts:3-10`), alors que le moteur distingue au moins une douzaine de refus nommés (`commands/forecast.rs`, `data_quality/types.rs:92-108`). Décision produit : le site décrit-il ce comportement tel quel, ou attend-on que les causes remontent à l'écran ?

**Non tranché — décision produit**

5. **Faut-il montrer sur le site les deux boutons inactifs** (« Coller des données », « Depuis une URL ») ? Recommandation : les mentionner en une phrase, sans date ni promesse — un lecteur qui les voit à l'écran doit comprendre qu'il n'a rien mal fait.
6. **Le vocabulaire « covariables ».** L'écran de configuration dit **« Covariables »** (`fr.json`, `forecast.config.covariates`) alors que la fenêtre Espace Forecast dit **« Variables externes »** pour la même chose (`fr.json`, `forecast.workbench.data.covariates`). Deux mots pour un même objet dans deux écrans voisins : à unifier dans l'application, et en attendant, à signaler dans la page pour que le lecteur reconnaisse les deux.

**Affichage non vérifié — liste de contrôle pour la passe d'interface**

7. L'emplacement exact du sélecteur « Mode du panneau » et son apparence quand le navigateur intégré est masqué.
8. L'écran d'accueil Forecast : disposition des trois boutons, apparence des deux boutons désactivés et de leur infobulle, et la liste « Analyses récentes » quand elle est vide.
9. L'écran « Configuration » : lisibilité des pastilles de covariables, comportement du curseur de confiance quand le modèle choisi n'accepte que certains paliers (`forecast-config.tsx:172-177`).
10. La fenêtre « Espace Forecast » : sa taille d'ouverture, la restauration de sa position, et le comportement de sa navigation à sept entrées sur une fenêtre étroite.
11. Le contenu réel de la fenêtre « Documentation Forecast » — treize entrées de sommaire ont été relevées dans `fr.json`, aucune page n'a été ouverte.
