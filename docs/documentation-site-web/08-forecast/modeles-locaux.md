# Les modèles de prévision locaux

**Emplacement site** — Forecast › Les modèles locaux
**Répond à** — « Quels modèles de prévision Beaver peut-il faire tourner sur ma machine, que sait faire chacun, et que se passe-t-il quand j'en installe un ? »
**Sources** — `src-tauri/src/services/forecast/catalog.rs`, `catalog_specs/mod.rs`, `catalog_specs/amazon.rs`, `catalog_specs/google.rs`, `catalog_specs/datadog.rs`, `catalog_specs/salesforce.rs`, `catalog_specs/ibm.rs`, `catalog_specs/experimental.rs`, `catalog_specs/providers.rs` ; `registry.rs`, `registry_specs.rs`, `registry_caps.rs`, `registry_params.rs` ; `model_manager/mod.rs`, `model_manager/install.rs`, `model_manager/download.rs`, `model_manager/model_artifacts.rs`, `model_manager/model_receipt.rs`, `model_manager/runtime_install.rs`, `model_manager/smoke.rs`, `model_manager/uninstall.rs` ; `sidecar_runtime.rs`, `sidecar_runtime_install.rs`, `sidecar_runtime_manifest.rs` ; `hardware_profile.rs` ; `interval_capability.rs` ; `model_listing.rs` ; `model_details.rs` ; `limits.rs` ; `src/components/forecast/forecast-model-readiness.ts`, `src/components/forecast/model-browser/model-install-btn.tsx` ; `src/i18n/fr.json` (section `forecast.models`, `forecast.providers`)
**Vérification** — Vérifié dans le code, catalogue relu ligne à ligne le 10 septembre 2026. L'apparence des écrans n'a pas été observée ; les points concernés sont en fin de fichier.

> **Cette page décrit les modèles locaux.** Le modèle distant TimeGPT-2 de Nixtla est traité dans `08-forecast/modele-cloud-timegpt.md`. Le choix d'un modèle pour une analyse donnée est traité dans `08-forecast/selection-du-modele.md`.

---

## Plan de page proposé

1. Ce qu'est un modèle de prévision dans Beaver
2. Le catalogue complet, famille par famille
3. Ce que chaque famille sait faire
4. Les paramètres réglables par famille
5. Ce qui se passe pendant une installation
6. Les quatre états d'un modèle
7. Le matériel : ce que Beaver vérifie avant de lancer
8. Désinstaller
9. Les intervalles de confiance : deux comportements

---

## Contenu

### 1. Ce qu'est un modèle de prévision dans Beaver

Un modèle de prévision est un fichier de poids téléchargé sur la machine, accompagné d'un environnement Python qui sait le faire tourner. Ces deux morceaux sont **installés séparément et comptés séparément** — c'est la clé pour comprendre les états et les messages que l'application affiche (`model_manager/install.rs:44-112` : le plan d'installation distingue le téléchargement du modèle et la préparation du moteur).

Un modèle appartient à une **famille** (`catalog.rs:29` — champ `family_id`). L'environnement Python est partagé par toute une famille : installer un deuxième Chronos-Bolt ne réinstalle pas le moteur (`sidecar_runtime_manifest.rs:30-44` ; `model_manager/mod.rs:110-131`).

Le vocabulaire de l'interface, à reprendre tel quel (`src/i18n/fr.json`, section `forecast.models`) : **Installer**, **Préparer**, **Installé**, **Préparation requise**, **Mise à jour requise**, **Désinstaller**, **Taille disque**, **RAM**, **Horizon max**, **Fréquences**, **Capacités du modèle**, **Moteur utilisé**.

### 2. Le catalogue complet, famille par famille

**Le catalogue est figé dans le code** (`catalog_specs/mod.rs:16-41`) : il compte **24 modèles** répartis sur **10 fournisseurs** (`catalog_specs/providers.rs:3-84`). Vingt tournent en local ; les quatre TimeGPT-2 sont distants et couverts par l'autre page.

Les noms affichés viennent du champ `display_name` de chaque fiche ; les noms de famille affichés viennent de `src/i18n/fr.json` (`forecast.models.families`).

Voir le tableau complet en section « Tableaux ».

**Les dix fournisseurs**, avec la description exacte affichée dans l'application (`src/i18n/fr.json`, `forecast.providers.*`) :

| Fournisseur (nom affiché) | Description affichée | Clé API |
|---|---|---|
| **Amazon Chronos** | « Modèles Chronos locaux pour la prévision de séries temporelles. » | Non |
| **Google TimesFM** | « TimesFM local, poids ouverts. » | Non |
| **Nixtla (TimeGPT-2)** | « Famille TimeGPT en cloud, accès par API. » | **Oui** |
| **Datadog Toto 2.0** | « Modèles de fondation Toto, séries temporelles et observabilité. » | Non |
| **Salesforce MOIRAI 2.0** | « Famille MOIRAI pour la prévision probabiliste. » | Non |
| **IBM FlowState** | « FlowState local pour séries temporelles. » | Non |
| **PriorLabs TabPFN-TS** | « Famille TabPFN-TS pour la prévision sans entraînement préalable. » | Non |
| **NX-AI TiRex** | « TiRex pour la prévision par quantiles, en zero-shot. » | Non |
| **Foundation Model Research Kairos** | « Kairos pour les séries temporelles multi-régimes. » | Non |
| **THUML Sundial** | « Sundial pour la prévision probabiliste en zero-shot. » | Non |

Tous les fournisseurs locaux affichent **« Local »** comme palier gratuit ; Nixtla affiche **« Aperçu / API »**.

**Deux précisions sur les noms**, à ne pas escamoter sur le site :

- L'identifiant de fournisseur **`google` désigne TimesFM ici, et Gemini dans la page des clés API**. Ce sont deux catalogues distincts, et le code le dit explicitement en tête de fichier (`catalog.rs:1-7`). Sur le site, écrire « Google TimesFM » en toutes lettres et ne jamais employer « Google » seul dans une page Forecast.
- Le modèle publié de la famille TabPFN-TS s'appelle **TabPFN-TS-3** (`catalog_specs/experimental.rs:18-32`). Un identifiant `tabpfn-ts` existe encore, mais **il n'est pas dans la liste du catalogue** : c'est un alias de compatibilité, résolu à part pour les analyses anciennes (`catalog.rs:51-56` ; `model_manager/model_artifacts.rs:45-49`). Ne pas le présenter comme un deuxième modèle installable.

### 3. Ce que chaque famille sait faire

Les capacités ne sont **pas** dans la fiche du modèle : elles sont dans le registre des moteurs (`registry_specs.rs:6-182`, valeurs définies dans `registry_caps.rs:3-73`). Six capacités sont exposées à l'interface, avec ces libellés (`src/i18n/fr.json`, `forecast.models.capabilities`) :

| Capacité affichée | Ce qu'elle veut dire |
|---|---|
| **Variables de contexte** | Le modèle exploite d'autres colonnes que la cible sur la période passée |
| **Contexte futur** | Il exploite en plus les valeurs futures connues de ces colonnes |
| **Multi-séries indépendantes** | Il prévoit plusieurs séries dans le même fichier, chacune pour elle-même |
| **Multivarié conjoint** | Il tient compte des liens entre les séries |
| **Prévision probabiliste** | Il produit un intervalle, pas seulement une valeur centrale |
| **Backtesting** | Il peut être évalué par la validation temporelle glissante |

Deux capacités supplémentaires existent dans la structure — **détection d'anomalies** et **affinage** (`registry.rs:33-34`) — et **aucun modèle du catalogue ne les active** : elles valent `false` partout (`registry_caps.rs:3-12`, aucune fonction ne les met à `true`). À ne pas mentionner sur le site comme une fonctionnalité.

**Répartition exacte** (voir le tableau détaillé en section « Tableaux ») :

- **Chronos-Bolt** (les quatre tailles) : prévision probabiliste et backtesting seulement. Pas de variables de contexte, pas de multi-séries (`registry_caps.rs:14-20`).
- **Chronos-2** et **TimesFM 2.5** : variables de contexte, contexte futur, multi-séries, probabiliste, backtesting (`registry_caps.rs:22-42`).
- **Toto 2.0** (les cinq tailles) : multi-séries, **multivarié conjoint**, probabiliste, backtesting. Pas de variables de contexte (`registry_caps.rs:53-61`).
- **MOIRAI 2.0, FlowState, TabPFN-TS, TiRex, Kairos, Sundial** : multi-séries, probabiliste, backtesting (`registry_caps.rs:44-51`).

**Conséquence concrète à écrire sur le site** : si votre fichier contient des colonnes explicatives — météo, promotions, jours fériés — seuls **Chronos-2** et **TimesFM 2.5** savent s'en servir parmi les modèles locaux. Le refus est explicite au lancement : « Variables futures non supportées par ce moteur » (`src-tauri/src/commands/forecast.rs:110-122`).

### 4. Les paramètres réglables par famille

Chaque famille expose sa propre liste de réglages (`registry_params.rs:1-78`, formes et bornes dans `model_config/schema.rs:27-115`). L'interface les affiche sous **« Paramètres »**.

Quatre réglages reviennent presque partout :

- **`horizon_max_override`** — relever la borne d'horizon du modèle ;
- **`context_length`** — combien de points de passé le modèle regarde ;
- **`quantiles`** — les niveaux de quantiles demandés en plus de l'intervalle choisi ;
- **`non_negative_output`** — empêcher les prévisions négatives.

Les réglages propres à une famille : `decode_block_size` et `has_missing_values` pour Toto ; `batch_size` pour MOIRAI ; `scale_factor` et `batch_first` pour FlowState ; `probabilistic_output` pour TabPFN-TS ; `output_type` (quantiles ou moyenne) pour TiRex ; `preserve_positivity`, `average_with_flipped_input` et `generation` pour Kairos ; `num_samples` et `dtype` pour Sundial (`registry_params.rs:19-78`).

Au plus **16 niveaux de quantiles** sont retenus, et les trois niveaux de l'intervalle choisi ne sont jamais évincés par les niveaux ajoutés à la main (`limits.rs:17` ; `intervals.rs:13-44`, test `never_truncates_the_selected_interval`).

### 5. Ce qui se passe pendant une installation

C'est la section la plus utile de la page : elle explique pourquoi une installation peut durer, et pourquoi elle peut échouer sans laisser de dégâts.

**Étape 1 — le téléchargement des poids.** Les fichiers viennent **exclusivement de Hugging Face**, sur une **révision figée** dans le code — un identifiant de commit de 40 caractères, jamais une branche mouvante (`model_manager/download.rs:75-83` ; les révisions sont dans chaque fiche du catalogue, par exemple `catalog_specs/amazon.rs:19`). Toute autre destination est refusée, y compris après redirection : cinq redirections au maximum, et seulement vers `huggingface.co` ou `hf.co` (`download.rs:61-95`, test `only_approved_hugging_face_hosts_are_downloaded`).

Le téléchargement se fait dans un **dossier de préparation séparé**, nommé `.<identifiant du modèle>.staging` (`install.rs:51`, `:143-150`). Le dossier définitif n'est remplacé qu'à la toute fin, par un **renommage indivisible** (`install.rs:186-188`). Une coupure en cours de route laisse donc soit l'ancienne installation entière, soit rien — jamais un modèle à moitié écrit.

**Étape 2 — la vérification.** Chaque fichier téléchargé a une taille et une **empreinte SHA-256 attendues**, inscrites dans un manifeste compilé dans l'application (`model_manager/model_artifacts.rs:9-33`). La vérification lit le fichier entier et compare l'empreinte ; toute différence fait échouer l'installation (`model_receipt.rs:124-158`). Le résultat est consigné dans un **reçu** `.artifacts-v1.json` déposé à côté des poids (`model_receipt.rs:11`, `:53-67`).

**Étape 3 — la préparation du moteur Python.** Beaver crée un environnement Python isolé pour la famille, puis y installe les bibliothèques (`sidecar_runtime_install.rs:34-89`). Quatre points comptent pour l'utilisateur :

- **Python 3.12 exactement** doit être présent sur la machine. Ni 3.11 ni 3.13 : le code cherche `python3.12`, `python3` puis `python`, et retient le premier qui annonce **3.12** (`sidecar_runtime_install.rs:130-156`, test `runtime_python_is_fixed_to_3_12`). Sans lui, le message est « Runtime Python compatible indisponible ».
- **Chaque bibliothèque est épinglée et vérifiée par empreinte.** L'installation utilise `--require-hashes` et `--only-binary=:all:`, sur `https://pypi.org/simple` uniquement (`sidecar_runtime_install.rs:52-71`). Une liste qui contiendrait `git+`, `file:`, un dépôt supplémentaire ou un paquet non épinglé est refusée avant même de commencer (`sidecar_runtime_manifest.rs:81-104`).
- **Le moteur est préparé à côté, lui aussi**, puis basculé en place une fois validé (`sidecar_runtime_install.rs:76-83`).
- **Si la préparation échoue, le moteur préparé pour rien est retiré** — mais seulement s'il n'existait pas déjà avant (`install.rs:114-141`).

**Étape 4 — le test de fumée.** Avant de déclarer le modèle prêt, Beaver le fait tourner une fois pour de vrai, avec un script dédié `test_model_smoke.py`, **sur le processeur** et **sans accès réseau** (`model_manager/smoke.rs:42-88` : `FORECAST_SMOKE_DEVICE=cpu`, `HF_HUB_OFFLINE=1`, `TRANSFORMERS_OFFLINE=1`). Le test dispose de **quinze minutes** ; au-delà, le processus est arrêté et l'installation échoue (`smoke.rs:7`, `:73-81`). C'est la raison pour laquelle la fin d'une installation peut sembler longue alors que le téléchargement est terminé.

**La barre de progression, décodée** (`install.rs:14`, `:54-57` ; `runtime_install.rs:33-46`) : le téléchargement occupe les **70 premiers pour cent**, la préparation du moteur va de **72 à 98 %**, la finalisation affiche **99 %**. Une barre qui reste longtemps à 80 % est en train d'installer les bibliothèques Python, pas de télécharger.

**L'installation est annulable** : le bouton **Annuler** est présent pendant toute la durée, et la touche **Échap** fait la même chose (`src/components/forecast/model-browser/model-install-btn.tsx:77-105`). Une annulation efface le dossier de préparation.

### 6. Les quatre états d'un modèle

Le moteur classe chaque modèle dans un état, et un seul (`model_manager/mod.rs:19-36`, `:75-92`) :

| État interne | Libellé affiché | Ce que ça veut dire |
|---|---|---|
| `not_installed` | **Non installé** | Les poids ne sont pas sur le disque |
| `invalid` | *(pas de libellé propre)* | Les fichiers sont là mais leurs tailles ne correspondent plus à ce qui est attendu |
| `update_required` | **Mise à jour requise** / **Préparation requise** | Les poids sont là, mais le reçu, le moteur Python ou le test de fumée ne sont plus à jour |
| `ready` | **Installé** | Tout est vérifié, le modèle est utilisable |

Le passage à **Prêt** exige **cinq conditions simultanées** : poids présents avec leur marqueur `.complete`, tailles conformes, reçu à jour, moteur de la famille prêt, test de fumée validé (`mod.rs:58-92`).

Deux conséquences à écrire clairement :

- **Une mise à jour de Beaver peut faire repasser un modèle en « Préparation requise »** sans que rien n'ait bougé sur le disque : il suffit qu'une version de bibliothèque ait changé dans la liste épinglée. Le bouton affiche alors **Préparer** au lieu de **Installer**, et **les poids ne sont pas retéléchargés** (`install.rs:44-51`, `:104-106` : le plan `RuntimeOnly` s'arrête après la validation).
- **Un modèle en état `invalid` n'a pas de libellé qui lui soit propre** dans les traductions relues. Voir « Points à confirmer ».

### 7. Le matériel : ce que Beaver vérifie avant de lancer

Chaque fiche de modèle porte trois chiffres : la **taille sur le disque**, la **mémoire vive nécessaire** et la **mémoire vidéo nécessaire** (`catalog.rs:31-33`). Beaver mesure la machine et compare (`hardware_profile.rs:37-46`, `:75-97`).

**La règle de comparaison**, en clair (`hardware_profile.rs:116-130`) :

| Verdict | Condition | Code interne |
|---|---|---|
| **Confortable** | La mémoire disponible vaut au moins **le double** du besoin | `comfortable` |
| **Contraint** | Elle couvre le besoin **plus 20 % de marge**, sans atteindre le double | `constrained` |

> Corrigé le 10 septembre 2026 : ce verdict s'appelait « Juste » ici et « Contraint » sur la page Sélection du modèle pour le même `constrained` (`hardware_profile.rs:9-15`, autorité unique). Aligné sur « Contraint » partout (la traduction anglaise dit « Constrained » sur les deux pages).
| **Insuffisant** | Elle ne couvre pas le besoin plus **20 %** | `insufficient` |
| **Inconnu** | La mesure a échoué | `unknown` |

**Beaver ajoute donc systématiquement 20 % au besoin annoncé** avant de trancher (`hardware_profile.rs:5`, `:120-124`). Il retient le meilleur des deux verdicts, carte graphique et processeur (`:132-144`) : un modèle qui ne tient pas sur la carte graphique reste utilisable s'il tient en mémoire vive.

**Ce qui est refusé, et avec quel message** (`hardware_profile.rs:99-114`) :

- verdict **Insuffisant** → « Ressources insuffisantes pour ce modèle » ;
- verdict **Inconnu** et modèle demandant plus de **2 048 Mo** de mémoire vive → « Ressources indisponibles pour ce modèle » (`limits.rs:38`).

Ce contrôle est fait **à chaque prévision** (`commands/forecast.rs:76`) et **avant chaque évaluation d'un modèle en backtest** (`evaluation/model_runner.rs:109-110`).

Sur un Mac à puce Apple, la mémoire est partagée : la mémoire vive disponible sert aussi de mémoire vidéo dans le calcul (`hardware_profile.rs:56-58`).

### 8. Désinstaller

La désinstallation se fait en trois temps, dans cet ordre (`model_manager/uninstall.rs:17-26`) :

1. le dossier de préparation éventuellement resté d'une installation interrompue ;
2. le dossier du modèle ;
3. **le moteur Python de la famille — mais seulement si aucun autre modèle de cette famille n'est encore installé** (`uninstall.rs:60-65` ; `mod.rs:110-131`).

Désinstaller Chronos-Bolt Tiny alors que Chronos-Bolt Base est installé ne touche donc pas au moteur, et le second reste utilisable immédiatement.

L'action demande une **confirmation** dans l'interface (bouton « Désinstaller » puis « Confirmer la désinstallation »), et elle est **indisponible tant qu'un téléchargement est en cours** (`model-install-btn.tsx:127-140`). Il n'y a **pas d'annulation après coup** : réinstaller suppose de retélécharger.

### 9. Les intervalles de confiance : deux comportements

Tous les modèles ne savent pas produire l'intervalle que vous demandez, et Beaver ne fait pas semblant (`interval_capability.rs:12-41`) :

| Comportement | Modèles concernés | Ce qui est proposé |
|---|---|---|
| **Grille fixe** | TimesFM 2.5, Toto 2.0, FlowState, TabPFN-TS, TiRex, Kairos | **60 % ou 80 %**, rien d'autre |
| **Continu** | Chronos-Bolt, Chronos-2, MOIRAI 2.0, Sundial, TimeGPT-2 | Tout niveau **entier de 50 % à 99 %** |

Un niveau non supporté est refusé au lieu d'être approché (`interval_capability.rs:32-41`, test `fixed_models_expose_only_honest_central_intervals`). C'est un choix assumé, à présenter comme tel : un intervalle à 90 % annoncé par un modèle qui n'en sait produire qu'un à 80 % serait faux sans que rien ne le signale.

---

## Tableaux

### Le catalogue local complet

Vingt modèles, dans l'ordre du catalogue (`catalog_specs/mod.rs:16-41`). La mémoire vive et la mémoire vidéo sont les besoins annoncés, **avant** les 20 % de marge que Beaver ajoute.

| Modèle affiché | Famille | Paramètres | Disque | Mémoire vive | Mémoire vidéo | Horizon max | Fréquences |
|---|---|---|---|---|---|---|---|
| **Chronos-Bolt Tiny** | Chronos-Bolt | 9 M | 35 Mo | 150 Mo | 60 Mo | 1 000 | 10S à Y |
| **Chronos-Bolt Mini** | Chronos-Bolt | 21 M | 85 Mo | 350 Mo | 120 Mo | 1 000 | 10S à Y |
| **Chronos-Bolt Small** | Chronos-Bolt | 48 M | 191 Mo | 750 Mo | 280 Mo | 1 000 | 10S à Y |
| **Chronos-Bolt Base** | Chronos-Bolt | 205 M | 821 Mo | 3 200 Mo | 1 200 Mo | 1 000 | 10S à Y |
| **Chronos-2** | Chronos-2 | 120 M | 478 Mo | 2 400 Mo | 1 000 Mo | 1 024 | 10S à Y |
| **TimesFM 2.5 200M** | TimesFM 2.5 | 200 M | 925 Mo | 4 200 Mo | 1 800 Mo | 1 000 | Toutes |
| **Toto 2.0 4M** | Toto-2 | 4 M | 17 Mo | 300 Mo | 150 Mo | 2 048 | Toutes |
| **Toto 2.0 22M** | Toto-2 | 22 M | 88 Mo | 650 Mo | 320 Mo | 2 048 | Toutes |
| **Toto 2.0 313M** | Toto-2 | 313 M | 1 251 Mo | 4 200 Mo | 1 800 Mo | 2 048 | Toutes |
| **Toto 2.0 1B** | Toto-2 | 1 Md | 4 164 Mo | 9 000 Mo | 5 200 Mo | 2 048 | Toutes |
| **Toto 2.0 2.5B** | Toto-2 | 2,5 Md | 9 817 Mo | 18 000 Mo | 11 000 Mo | 2 048 | Toutes |
| **MOIRAI 2.0 R Small** | Moirai-2 | non publié | 46 Mo | 700 Mo | 280 Mo | 1 024 | Toutes |
| **FlowState R1** | FlowState | 9 M | 36 Mo | 650 Mo | 260 Mo | 4 096 | Toutes |
| **FlowState R1.1** | FlowState | 18,5 M | 74 Mo | 750 Mo | 320 Mo | 4 096 | Toutes |
| **TabPFN-TS-3** | TabPFN-TS | non publié | 233 Mo | 8 192 Mo | 8 192 Mo | 1 024 | Toutes |
| **TiRex 35M** | TiRex | 35 M | 141 Mo | 900 Mo | 420 Mo | 1 024 | Toutes |
| **Kairos 10M** | Kairos | 10 M | 40 Mo | 320 Mo | 140 Mo | 1 024 | Toutes |
| **Kairos 23M** | Kairos | 23 M | 92 Mo | 520 Mo | 220 Mo | 1 024 | Toutes |
| **Kairos 50M** | Kairos | 50 M | 201 Mo | 980 Mo | 420 Mo | 1 024 | Toutes |
| **Sundial 128M** | Sundial | 128 M | 513 Mo | 2 400 Mo | 1 000 Mo | 1 024 | Toutes |

Trois fiches portent **`—`** dans le champ « paramètres » — MOIRAI 2.0 R Small, TabPFN-TS-3 et les quatre TimeGPT-2 (`catalog_specs/salesforce.rs:8`, `experimental.rs:9`, `nixtla.rs:8`). Le nombre de paramètres n'est pas renseigné, il n'est pas égal à zéro : sur le site, écrire « non publié » plutôt que de laisser un tiret nu.

### Les capacités, famille par famille

| Famille | Variables de contexte | Contexte futur | Multi-séries | Multivarié conjoint | Probabiliste | Backtesting |
|---|---|---|---|---|---|---|
| **Chronos-Bolt** | — | — | — | — | **oui** | **oui** |
| **Chronos-2** | **oui** | **oui** | **oui** | — | **oui** | **oui** |
| **TimesFM 2.5** | **oui** | **oui** | **oui** | — | **oui** | **oui** |
| **Toto-2** | — | — | **oui** | **oui** | **oui** | **oui** |
| **Moirai-2** | — | — | **oui** | — | **oui** | **oui** |
| **FlowState** | — | — | **oui** | — | **oui** | **oui** |
| **TabPFN-TS** | — | — | **oui** | — | **oui** | **oui** |
| **TiRex** | — | — | **oui** | — | **oui** | **oui** |
| **Kairos** | — | — | **oui** | — | **oui** | **oui** |
| **Sundial** | — | — | **oui** | — | **oui** | **oui** |
| *TimeGPT-2 (distant)* | **oui** | **oui** | **oui** | 2.1 seulement | **oui** | **oui** |

Source : `registry_specs.rs:6-182` et `registry_caps.rs:14-73`.

### Où vivent les fichiers

| Élément | Emplacement | Source |
|---|---|---|
| Poids d'un modèle | `~/.local/share/cl-go-dash/forecast-models/<identifiant>/` | `model_manager/mod.rs:38-48` |
| Marqueur d'installation terminée | `.../<identifiant>/.complete` | `mod.rs:105-108` |
| Reçu de vérification | `.../<identifiant>/.artifacts-v1.json` | `model_receipt.rs:11` |
| Dossier de préparation | `.../.<identifiant>.staging` | `install.rs:51` |
| Environnements Python | `~/.local/share/cl-go-dash/forecast-sidecar/` | `mod.rs:42-44` |
| Réglages par modèle | `~/.local/share/cl-go-dash/forecast-model-configs.json` | `commands/forecast_models.rs:92-105` |
| Dernier modèle choisi | `~/.local/share/cl-go-dash/forecast-selected-model.json` | `commands/forecast_models.rs:13-27` |

---

## Encadrés

> **ℹ Un modèle, c'est deux installations.**
> Les poids d'un côté, l'environnement Python de sa famille de l'autre. C'est pourquoi le bouton affiche parfois **Préparer** au lieu de **Installer** : les poids sont déjà là, seul le moteur doit être refait. Rien n'est retéléchargé dans ce cas.

> **⚠ Python 3.12 doit être installé sur votre machine.**
> Exactement cette version : ni 3.11, ni 3.13. Sans elle, aucun modèle local ne peut être préparé, et le message affiché est « Runtime Python compatible indisponible ».

> **ℹ La fin d'une installation est lente, et c'est normal.**
> Une fois les fichiers téléchargés, Beaver fait tourner le modèle une première fois pour vérifier qu'il fonctionne vraiment. Ce test dispose de quinze minutes et se déroule sur le processeur, sans réseau.

> **ℹ Tout est vérifié, rien n'est pris sur parole.**
> Les poids viennent d'une version figée de Hugging Face et sont contrôlés fichier par fichier par leur empreinte. Les bibliothèques Python sont épinglées à une version et à une empreinte. Une seule différence fait échouer l'installation plutôt que de la laisser passer.

> **ℹ Beaver ajoute 20 % à la mémoire annoncée.**
> Un modèle qui demande 4 200 Mo n'est lancé que si 5 040 Mo sont réellement disponibles. En dessous, Beaver refuse au lieu de lancer un calcul qui ramerait ou échouerait à mi-parcours.

> **⚠ Seuls Chronos-2 et TimesFM 2.5 exploitent vos colonnes explicatives, parmi les modèles locaux.**
> Météo, promotions, jours fériés : les autres modèles locaux ignorent ces colonnes, et Beaver refuse le lancement plutôt que de les laisser passer sans effet.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Runtime Python compatible indisponible » | Python 3.12 absent de la machine | Installer Python 3.12 ; les autres versions ne conviennent pas |
| Le bouton affiche **Préparer** alors que le modèle était installé | La liste des bibliothèques a changé avec une mise à jour de Beaver | Cliquer sur Préparer ; les poids ne sont pas retéléchargés |
| L'installation reste bloquée vers 80 % | C'est l'installation des bibliothèques Python, pas le téléchargement | Patienter ; la barre repart à 98 % puis 99 % |
| L'installation échoue tout à la fin | Le test de fumée n'a pas abouti, ou a dépassé quinze minutes | Réessayer ; vérifier que la machine n'est pas saturée |
| « Ressources insuffisantes pour ce modèle » | Mémoire disponible inférieure au besoin **plus 20 %** | Choisir une variante plus petite, ou fermer des applications |
| « Ressources indisponibles pour ce modèle » | La mémoire n'a pas pu être mesurée et le modèle demande plus de 2 048 Mo | Choisir un modèle plus léger |
| « Variables futures non supportées par ce moteur » | Le fichier contient des colonnes explicatives et le modèle choisi ne sait pas les lire | Passer à Chronos-2 ou TimesFM 2.5, ou retirer ces colonnes |
| Le niveau de confiance voulu n'est pas proposé | Le modèle ne produit que des intervalles à 60 % ou 80 % | Choisir 60 % ou 80 %, ou passer à un modèle à intervalle continu |
| Le bouton **Désinstaller** est grisé | Un téléchargement est en cours | Attendre la fin ou l'annuler |

---

## Renvois

- `08-forecast/modele-cloud-timegpt.md` — la famille TimeGPT-2, distante, et sa clé API
- `08-forecast/selection-du-modele.md` — choisir un modèle pour une analyse, et la sélection automatique
- `08-forecast/evaluation-et-comparaison.md` — mesurer un modèle contre les baselines avant de lui faire confiance
- `06-modeles/materiel-et-vram.md` — la mémoire disponible sur votre machine (modèles de langage, mêmes principes)
- `12-reference/emplacement-des-donnees.md` — l'inventaire de `~/.local/share/cl-go-dash/`

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **L'état `invalid` n'a pas de libellé utilisateur dédié.** Les traductions relues (`src/i18n/fr.json`, `forecast.models`) offrent « Non installé », « Installé », « Mise à jour requise » et « Préparation requise », mais rien pour un modèle dont les fichiers sont présents et de mauvaise taille (`model_manager/mod.rs:28-35`). À vérifier à l'écran : que voit réellement l'utilisateur dans ce cas ? À traiter côté produit si l'état se traduit par un bouton muet.
2. **Le champ `params` vaut `—` pour trois fiches** (MOIRAI 2.0 R Small, TabPFN-TS-3, TimeGPT-2). Décision de rédaction proposée : écrire « non publié » sur le site. À confirmer avec Kevin, parce que cela change aussi ce que l'application devrait afficher.
3. **L'alias `tabpfn-ts`** existe hors catalogue pour les analyses anciennes (`catalog.rs:51-56`). Recommandation : ne pas le documenter du tout, sauf si des utilisateurs peuvent encore le rencontrer dans une analyse enregistrée avant la bascule vers TabPFN-TS-3. À trancher avec Kevin.
4. **Les deux révisions de FlowState pointent sur le même dépôt Hugging Face** (`ibm-granite/granite-timeseries-flowstate-r1`) avec deux identifiants de version différents (`catalog_specs/ibm.rs:18-19`, `:40-41`). Ce n'est pas une anomalie en soi — c'est ainsi que le dépôt publie ses révisions — mais la page ne doit pas laisser croire à deux dépôts distincts.

**Non vérifié — hors de portée d'une lecture du code**

5. **Les durées réelles d'installation** par famille et par machine. Le code n'en dit rien ; seules les bornes sont connues (quinze minutes pour le test de fumée). Ne rien annoncer sur le site tant qu'une mesure n'a pas été faite.
6. **Le comportement si Python 3.12 est présent mais incomplet** (par exemple sans le module `venv`, cas courant sur certaines distributions Linux). Le code n'a qu'un seul message générique. À provoquer sur une machine de test avant d'écrire une consigne de dépannage.
7. **Les descriptions longues des modèles** viennent de Hugging Face ou de GitHub au moment de l'affichage (`model_details.rs:19-30`), donc du réseau. Ce que voit un utilisateur hors ligne n'a pas été observé.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

8. L'écran de catalogue : comment les vingt modèles locaux sont regroupés, et si la famille est visible.
9. La fiche détaillée d'un modèle : quels champs sont affichés parmi « Taille disque », « RAM », « CPU », « GPU », « Licence », « Bibliothèque », « Pipeline », « Horizon max », « Fréquences », et lesquels restent vides hors ligne.
10. La barre de progression et ses libellés de phase, aux trois moments décrits (téléchargement, préparation du moteur, finalisation).
11. L'étiquette de machine — **Machine légère**, **Machine moyenne**, **Machine puissante**, **Traitement distant** — est déduite de la **seule taille sur le disque**, pas de la mémoire nécessaire ni de la machine de l'utilisateur : jusqu'à **64 Mo** légère, jusqu'à **512 Mo** moyenne, au-delà puissante, et distante pour les modèles cloud (`src/components/forecast/forecast-model-meta.ts:161-164`). **C'est un écart à signaler** : Toto 2.0 4M pèse 17 Mo sur le disque et demande 300 Mo de mémoire vive, tandis que TabPFN-TS-3 pèse 233 Mo et en demande 8 192. Étiqueté « Machine moyenne », TabPFN-TS-3 est en réalité le modèle local le plus exigeant du catalogue. À arbitrer avec Kevin : soit la page n'emploie pas ces étiquettes, soit le calcul est corrigé côté application.
