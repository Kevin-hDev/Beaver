# Agent : Mémoire, System prompt, Outils, Avancé

**Emplacement site** — Réglages › Agent
**Répond à** — « Comment décide-t-on de ce que l'agent sait, de ce qu'il a le droit de faire et des dossiers auxquels il accède ? »
**Sources** — `src/components/settings/memory-settings.tsx`, `system-prompt-settings.tsx`, `tools-settings.tsx`, `advanced-settings.tsx`, `advanced-settings-state.ts`, `file-access-settings.tsx`, `path-list-editor.tsx`, `session-workspace-settings.tsx`, `ollama-settings-section.tsx`, `vram-table.tsx` ; `src/components/agent-import/agent-import-settings.tsx` ; `src-tauri/src/services/agent_local/tool_group_catalog.rs`, `memory_types.rs`, `memory_settings.rs` ; `src-tauri/src/commands/config.rs`, `commands/directory_access.rs`, `src-tauri/src/models/config.rs` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, sur la version **1.2.2**. Aucune vérification à l'écran (voir « Points à confirmer »).

> **Cette page décrit les écrans de réglage.** Le fonctionnement de la mémoire est dans `04-agent/memoire-persistante.md`, celui des instructions système dans `04-agent/prompts-systeme.md`, celui de chaque outil dans `05-outils/`, et le modèle de permissions dans `04-agent/permissions.md`. Ici : où est le champ, ce qu'il vaut, ce qu'il déclenche.

---

## Plan de page proposé

1. Ce que contient la section Agent
2. Onglet Mémoire
3. Onglet System prompt
4. Onglet Outils — les groupes verrouillés et les groupes optionnels
5. Onglet Avancé — importation depuis d'autres assistants
6. Onglet Avancé — comportement général
7. Onglet Avancé — le moteur Ollama
8. Onglet Avancé — Accès fichiers
9. Onglet Avancé — les fichiers des sessions

---

## Contenu

### 1. Ce que contient la section Agent

Quatre onglets (`src/features/extension-ui/core-occupants.tsx:35-42`) :

| Onglet | Ce qu'on y règle |
|---|---|
| **Mémoire** | Ce que l'agent a le droit de retenir d'une conversation à l'autre |
| **System prompt** | Les instructions qui définissent son comportement |
| **Outils** | Ce qu'il a le droit de faire |
| **Avancé** | Le modèle par défaut, le moteur local, l'accès aux fichiers et aux dossiers |

L'onglet **Avancé** est le plus chargé de toute l'application : il porte six blocs sans rapport direct entre eux. C'est là que vivent les deux réglages que les utilisateurs cherchent le plus souvent ailleurs : **l'accès aux dossiers** et **le comportement du moteur Ollama**.

### 2. Onglet Mémoire

L'onglet s'ouvre sur une phrase de cadrage : « Gère ce que l'agent peut retenir durablement, sans charger tous les détails dans chaque conversation. » (`memory-settings.tsx:97`).

**Deux réglages**, dans une carte (`memory-settings.tsx:99-120`).

**Mode mémoire** — « Le mode manuel exige une demande explicite. Le mode automatique retient seulement les faits durables. » Trois valeurs :

| Libellé affiché | Comportement |
|---|---|
| **Désactivé** | **Valeur par défaut** (`memory_types.rs:36-37`). Rien n'est retenu ; les fichiers déjà écrits sont conservés |
| **Manuel** | L'agent ne retient que sur demande explicite |
| **Automatique** | L'agent retient de lui-même ce qu'il juge durable |

**Budget de contexte** — « Limite maximale réservée au résumé et aux lectures mémoire. » Six valeurs proposées : **512, 1 000, 1 500, 2 000, 2 500 et 3 000 tokens** (`memory-settings.tsx:16`). La valeur par défaut est **3 000**, qui est aussi le maximum accepté par le moteur (`memory_types.rs:3-4`). Le sélecteur est **grisé quand le mode est Désactivé** (`memory-settings.tsx:116`).

Ce budget est un plafond, pas une réservation : c'est la place maximale que la mémoire peut prendre dans la fenêtre de contexte d'une conversation.

**Sous les réglages, la consultation.** Une section **« Mémoires enregistrées »** liste ce que l'agent a retenu, avec un champ de recherche et deux filtres (`memory-settings.tsx:136-149`) :

| Filtre | Valeurs |
|---|---|
| Type | **Tous les types**, Préférence, Retour, Projet, Référence |
| Statut | **Tous les statuts**, Confirmé, Déduit, À vérifier |

Les mémoires sont groupées par portée : **Mémoire globale**, puis **Projet actif · *nom du projet*** s'il y en a un, puis les autres projets (`memory-settings.tsx:150-177`). Les mémoires des autres projets ne sont pas chargées d'emblée : un bouton **« Afficher les mémoires »** les demande à la demande.

Sélectionner une mémoire ouvre son contenu, avec deux actions : **Fermer** et **Archiver** (`memory-settings-topic-preview.tsx`). L'archivage demande une confirmation.

**Deux états particuliers à documenter :**

- **Mode désactivé** — un bandeau explique : « La mémoire est désactivée. Les fichiers existants sont conservés. » (`memory-settings.tsx:137-139`). Rien n'est effacé quand on éteint la mémoire.
- **Ancien contenu détecté** — un bandeau **« Ancien contenu détecté »** apparaît si des fichiers d'une organisation antérieure sont trouvés : « Les anciens dossiers sont conservés mais ne sont pas injectés. » (`memory-settings.tsx:122-127`).

L'onglet a ses quatre états écrits : chargement (« Chargement de la mémoire… »), erreur (« Impossible de charger la mémoire » avec un bouton **Réessayer**), vide (« Aucune mémoire pour cette portée ») et rempli (`memory-settings.tsx:70-79`, `:107` de `memory-settings-state.tsx`).

### 3. Onglet System prompt

L'onglet s'appelle **« System prompt »** dans la liste de gauche, mais son titre à l'ouverture est **« Instructions système »** (`system-prompt-settings.tsx:10` → `fr.json`, clé `settings.systemPrompt.title`). Les deux mots désignent la même chose ; voir « Points à confirmer ».

Cet écran règle les instructions **globales**, celles qui s'appliquent à tous les modèles (`system-prompt-settings.tsx:12-16`, portée `global`). Les modèles Ollama peuvent avoir leurs propres instructions, réglées ailleurs — voir `10-reglages/modeles.md`.

Deux sélecteurs commandent ce qui est affiché :

| Sélecteur | Libellé | Valeurs |
|---|---|---|
| **Mode du modèle** | `settings.systemPrompt.modeLabel` | **Chatbot**, **Agentique** |
| **Format des instructions** | `settings.systemPrompt.tierLabel` | **Compact**, **Détaillé** |

À l'ouverture, l'écran présente le mode **Agentique** au format **Détaillé** (`system-prompt-settings.tsx:14-15`). Il y a donc **quatre textes distincts** à consulter et à personnaliser, un par combinaison.

Trois provenances possibles pour le texte affiché (`fr.json`, clés `settings.systemPrompt.sources.*`) : **Beaver**, **Ollama**, **Custom**. Les actions proposées sont **Modifier**, **Utiliser Beaver**, **Utiliser Ollama**, puis **Enregistrer** ou **Annuler**.

**Deux garde-fous à décrire, parce qu'ils protègent d'une perte de travail :**

- **Un avertissement avant la première modification** — titre « Attention », corps : « Cette modification remplace les instructions système de Beaver pour toutes les IA, sauf les modèles Ollama qui possèdent leurs propres instructions personnalisées. » Une case **« Ne plus afficher cet avertissement »** permet de le taire (`fr.json`, `settings.systemPrompt.warning.*`).
- **Un avertissement de perte** — « Le prompt personnalisé va être supprimé », avec un bouton **« Copier le prompt »** pour le sauvegarder avant de continuer (`fr.json`, `settings.systemPrompt.loss.*`). Il s'affiche quand on revient aux instructions de Beaver ou d'Ollama après avoir écrit les siennes.

Laisser le champ vide est un choix explicite, et le texte du champ le dit : « Laisser vide pour ne transmettre aucune instruction système. »

### 4. Onglet Outils — les groupes verrouillés et les groupes optionnels

L'onglet s'ouvre sur une phrase qui pose tout le principe : « Ces outils définissent ce que l'agent peut faire pour toi. Les outils essentiels sont toujours actifs. Les outils optionnels s'activent ou se désactivent selon tes besoins : l'agent n'utilisera que ce qui est activé. » (`tools-settings.tsx:68`).

Deux sections, **« Tools essentiels »** et **« Tools optionnels »** (`tools-settings.tsx:70`, `:84`).

Les outils ne se règlent pas un par un : ils sont **groupés**, et l'interrupteur agit sur tout le groupe (`tools-settings.tsx:52-64`). La liste des groupes est déclarée une seule fois, côté moteur (`tool_group_catalog.rs:12-81`).

**Les cinq groupes toujours actifs.** Ils portent la mention **« Toujours actif »** à la place d'un interrupteur, et le moteur refuse toute tentative de les éteindre, y compris par un appel direct — le message d'erreur est « Ce groupe d'outils est verrouillé. » (`tool_group_catalog.rs:106-108`).

Les descriptions ci-dessous sont **abrégées** ; le texte affiché commence toujours par « Permet à l'agent de… » et figure en entier dans `fr.json`, clés `settings.tools.groups.*.description`.

| Groupe | Ce qu'il permet |
|---|---|
| **Terminal** | Lancer des commandes directement sur ta machine (git, builds, scripts). L'outil le plus puissant : tout ce que ton terminal peut faire, l'agent peut le faire |
| **Fichiers** | Lire, créer et modifier les fichiers de ton projet : code, configs, notes, documents |
| **Recherche de fichiers** | Fouiller ton projet pour retrouver un fichier ou chercher un mot précis dans le code |
| **Web** | Chercher des infos en ligne et ouvrir des pages web |
| **Connecteurs externes** | Utiliser les connecteurs externes configurés dans l'onglet Connecteurs |

**Les onze groupes activables**, avec leur état à l'installation (`tool_group_catalog.rs:25-81`, troisième paramètre de chaque ligne) :

| Groupe | Actif à l'installation | Ce qu'il permet |
|---|---|---|
| **Skills** | **Oui** | Suivre des guides spécialisés pour certaines tâches |
| **Automatisations** | **Oui** | Créer et gérer des tâches planifiées |
| **Choix utilisateur** | **Oui** | Vous demander votre avis quand l'agent hésite entre plusieurs options |
| **Sous-agents** | **Oui** | Déléguer une sous-tâche à un agent secondaire |
| **Plan mode** | **Oui** | Proposer un plan et attendre votre accord avant d'agir |
| **Todo list** | Non | Tenir une checklist pendant un long travail |
| **Branches Git** | Non | Créer ou changer de branche Git |
| **Forecast** | Non | Utiliser le module Forecast |
| **Spreadsheet / Excel** | Non | Lire ou créer des tableurs (Excel, ODS, CSV, TSV) |
| **Document / Word** | Non | Extraire le texte de documents ou créer des fichiers Word |
| **Images** | Non | Inspecter, redimensionner, recadrer ou convertir des images |

**Le fait le plus utile de cette page**, à mettre en évidence : **six groupes sur onze sont éteints à l'installation.** Un utilisateur qui demande à l'agent de lire un fichier Excel, de créer une branche Git ou de retoucher une image obtiendra un refus tant qu'il n'a pas activé le groupe correspondant ici.

**Un groupe est considéré comme activé seulement si tous ses outils le sont** (`tools-settings.tsx:48-50`). Le cas d'un groupe à moitié activé ne se produit pas par l'interface ; il pourrait apparaître avec un fichier de réglages édité à la main, et l'interrupteur se présenterait alors éteint.

### 5. Onglet Avancé — importation depuis d'autres assistants

Le premier bloc de l'onglet (`advanced-settings.tsx:99`), intitulé **« Importation depuis d'autres assistants »** : « Gère les règles, instructions et skills lus depuis tes autres outils. » Un bouton **« Gérer »** ouvre l'écran dédié.

Le sujet est traité en entier dans `02-installation/import-depuis-un-autre-assistant.md` ; retenir ici seulement que **c'est le chemin pour y revenir après le premier lancement**, ce que l'application annonce d'ailleurs pendant l'accueil : « Tu pourras modifier ces choix plus tard dans Réglages > Avancé. » (`fr.json`, clé `agentImport.onboardingLater`).

### 6. Onglet Avancé — comportement général

Une carte avec deux réglages (`advanced-settings.tsx:101-126`).

**Icône dans la barre** — « Garder Beaver dans la barre système ». **Activé par défaut** (`src-tauri/src/models/config.rs:43`).

**Modèle par défaut** — « Modèle utilisé par défaut pour les nouvelles conversations ». Une liste déroulante avec recherche, groupée par fournisseur, alimentée par les modèles réellement disponibles (`advanced-settings.tsx:79-93`). **Vide par défaut** (`models/config.rs:44`).

### 7. Onglet Avancé — le moteur Ollama

Un bloc titré **« Ollama »**, distinct de l'onglet Ollama de la section Modèles. La séparation est celle-ci, et il faut l'écrire : **l'onglet Ollama gère les modèles ; ce bloc-ci gère le moteur qui les fait tourner.**

Quatre réglages (`ollama-settings-section.tsx:79-127`) :

**Décharger le modèle** — « Durée avant de libérer la mémoire du modèle ». Sept valeurs : **Immédiatement**, **Après 2 min**, **Après 5 min**, **Après 10 min**, **Après 15 min**, **Après 30 min**, **À la fermeture**. Valeur par défaut : **Après 5 min** (`models/config.rs:45`, valeur `5m`).

**Accélération matérielle** — « Moteur d'inférence pour les modèles locaux ». Deux valeurs, **CPU** et **GPU**, avec **GPU** par défaut (`models/config.rs:48`). **Ce réglage n'apparaît pas sur Mac** : il est masqué sur macOS, où la question ne se pose pas (`ollama-settings-section.tsx:90`).

**Charger plusieurs modèles** — « Garder plusieurs modèles en VRAM simultanément (nécessite plus de mémoire GPU) ». **Désactivé par défaut** (`models/config.rs:49`).

**Afficher l'état GPU** — « Afficher l'utilisation RAM/VRAM et l'exécution sur CPU dans le chat ». **Désactivé par défaut** (`models/config.rs:50`).

**Un redémarrage est nécessaire pour trois de ces réglages sur quatre.** Changer le déchargement, l'accélération matérielle ou le multi-modèle fait apparaître une ligne supplémentaire — **« Redémarrage nécessaire »**, « Les changements seront appliqués après le redémarrage d'Ollama » — avec un bouton **« Restart Ollama »** (`ollama-settings-section.tsx:129-143`). Seul « Afficher l'état GPU » s'applique immédiatement (`:125`).

Le redémarrage donne trois résultats possibles, chacun avec son message (`ollama-settings-section.tsx:56-64`) : **« Ollama redémarré »** en cas de succès, **« Ollama externe réutilisé »** quand Beaver a trouvé un moteur déjà lancé qu'il ne possède pas, ou un message d'erreur traduit.

**Sous ces réglages, une table de référence** : **« VRAM requise par modèle »**, « Estimation de la VRAM nécessaire selon la taille et la quantification » (`ollama-settings-section.tsx:145`, `vram-table.tsx`). Son contenu et sa lecture sont détaillés dans `06-modeles/materiel-et-vram.md`, qui est la page à lier ici.

### 8. Onglet Avancé — Accès fichiers

C'est le réglage le plus important de la page, et il est rangé au milieu d'un onglet fourre-tout. Le site doit le mettre en avant.

Titre affiché : **« Accès fichiers »**. Description : « Dossiers de travail des sessions. Les composants nécessaires aux outils et l'accès réseau restent disponibles » (`file-access-settings.tsx:57-60`).

L'écran présente une liste de dossiers, chacun avec une croix pour le retirer, et deux boutons : **« + Ajouter un dossier »** et **« Réinitialiser »** (`path-list-editor.tsx:60-67`).

**La valeur par défaut est la racine du disque** — `/` sur macOS et Linux, `C:\` sur Windows (`src-tauri/src/models/config.rs:116-119`). L'interface ne l'affiche pas comme un chemin mais sous le libellé **« Tout le disque »** (`path-list-editor.tsx:16-17`). C'est un choix assumé de Beaver : l'agent démarre avec l'accès complet au disque, et c'est à l'utilisateur de le restreindre s'il le souhaite. La page `11-securite/modele-de-securite.md` développe ce choix.

**Trois comportements vérifiés, à écrire tels quels :**

- **La liste ne peut jamais être vide.** La croix de suppression est désactivée dès qu'il ne reste qu'un dossier (`path-list-editor.tsx:49`), et le moteur remet le défaut si une liste vide lui parvient malgré tout (`models/config.rs:65-69`).
- **Le bouton « Réinitialiser » rend l'accès complet au disque.** Il ne réduit pas les droits, il les remet à leur valeur d'origine — c'est-à-dire la plus large (`path-list-editor.tsx:34-36`). Le libellé peut se lire comme une mesure de prudence alors qu'il fait l'inverse. Voir « Points à confirmer ».
- **Restreindre l'accès interrompt le travail en cours.** Quand la nouvelle liste est plus étroite que l'ancienne, le moteur **annule toutes les requêtes de l'agent en cours et arrête tous les processus de terminal lancés** (`src-tauri/src/commands/directory_access.rs:19-23`). C'est délibéré : sans cela, une commande déjà partie continuerait de travailler dans un dossier qui vient d'être retiré. Élargir l'accès, au contraire, n'interrompt rien (`:27-34`).

**Ce réglage a un propriétaire unique.** Il ne passe pas par le même chemin que les autres réglages avancés : les deux commandes générales d'enregistrement — celle qui remplace tout et celle qui modifie un champ — **refusent explicitement de toucher à la liste des dossiers** (`commands/config.rs:52-58` et `:79`, `:94`). Seule la commande dédiée peut la changer. Conséquence concrète : aucun autre écran, aucun enregistrement automatique, aucune extension passant par les réglages généraux ne peut élargir vos droits d'accès par effet de bord.

Un lien depuis un message d'erreur peut ouvrir directement ce bloc : la zone est alors mise en évidence pendant **1,8 seconde** et l'écran défile jusqu'à elle (`file-access-settings.tsx:7`, `:26-43`).

### 9. Onglet Avancé — les fichiers des sessions

Dernier bloc, titré **« Fichiers des sessions »** (`session-workspace-settings.tsx:38`), avec deux lignes.

**Emplacement des livrables** — « Beaver y crée un dossier séparé pour les livrables de chaque session ». Deux boutons : **« Choisir »** ouvre un sélecteur de dossier, **« Par défaut »** — visible seulement si un dossier a été choisi — revient à la valeur d'origine, affichée **« Dossier Beaver par défaut »** (`session-workspace-settings.tsx:44-65`).

Le chemin saisi est validé côté moteur avant d'être accepté : il doit être **absolu**, faire moins de **4 096 caractères**, ne contenir aucun caractère de contrôle et **aucun `..`** (`src-tauri/src/models/config.rs:74-98`). Un chemin refusé produit le message « Dossier de sortie invalide. » (`commands/config.rs:71`).

**Données Beaver** — « Ouvrir le dossier qui contient les sessions, la mémoire et les réglages locaux », avec un bouton **« Ouvrir le dossier »** qui ouvre l'explorateur de fichiers du système (`session-workspace-settings.tsx:28-34`, commande `open_app_data_folder`).

C'est le chemin le plus court pour atteindre `~/.local/share/cl-go-dash/` sans le taper — à mentionner dans la page qui décrit ce dossier.

---

## Tableaux

### Les valeurs par défaut de l'onglet Avancé

Toutes vérifiées côté moteur (`src-tauri/src/models/config.rs:38-59`), qui fait autorité sur l'interface.

| Réglage | Valeur à l'installation |
|---|---|
| Icône dans la barre | **Activée** |
| Modèle par défaut | **Aucun** |
| Décharger le modèle | **Après 5 min** |
| Accélération matérielle | **GPU** |
| Charger plusieurs modèles | **Désactivé** |
| Afficher l'état GPU | **Désactivé** |
| Accès fichiers | **Tout le disque** (`/`, ou `C:\` sous Windows) |
| Emplacement des livrables | **Dossier Beaver par défaut** |

---

## Encadrés

> **⚠ Six groupes d'outils sur onze sont éteints à l'installation.**
> Tableurs, documents Word, images, branches Git, listes de tâches et Forecast ne sont pas disponibles tant qu'ils ne sont pas activés dans **Agent › Outils**. Un agent qui « refuse » d'ouvrir un fichier Excel n'a le plus souvent pas l'outil, pas un problème de capacité.

> **⚠ « Réinitialiser » élargit l'accès, il ne le restreint pas.**
> Dans **Accès fichiers**, ce bouton remet la valeur d'origine — c'est-à-dire **tout le disque**. Pour restreindre, il faut retirer les dossiers un à un et n'ajouter que ceux dont l'agent a besoin.

> **⚠ Restreindre l'accès aux dossiers arrête le travail en cours.**
> Retirer un dossier annule les requêtes de l'agent en cours et met fin aux commandes de terminal lancées. C'est voulu : une commande déjà partie ne doit pas continuer à travailler dans un dossier qu'on vient de lui retirer. Élargir l'accès, en revanche, n'interrompt rien.

> **ℹ Éteindre la mémoire n'efface rien.**
> Le mode **Désactivé** cesse d'injecter et d'écrire, mais les mémoires déjà enregistrées restent sur le disque. Les rallumer les rend à nouveau disponibles.

> **ℹ Le moteur Ollama se règle ici, pas dans l'onglet Ollama.**
> L'onglet Ollama de la section Modèles installe et gère les modèles. La libération de la mémoire, le choix processeur ou carte graphique, le chargement simultané de plusieurs modèles sont dans **Agent › Avancé**.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent dit qu'il ne peut pas lire mon fichier Excel » | Le groupe **Spreadsheet / Excel** est éteint à l'installation | L'activer dans **Agent › Outils** |
| « L'agent ne veut pas créer de branche Git » | Le groupe **Branches Git** est éteint à l'installation | Même écran |
| « Je ne peux pas désactiver le terminal » | Le groupe **Terminal** est verrouillé, comme quatre autres | Comportement voulu ; passer par les permissions (`04-agent/permissions.md`) |
| « L'agent ne se souvient de rien » | Le mode mémoire est **Désactivé** par défaut | Le passer en Manuel ou Automatique dans **Agent › Mémoire** |
| « Le budget de mémoire est grisé » | Le mode est sur **Désactivé** | Choisir un autre mode d'abord |
| « J'ai réduit les dossiers accessibles et ma conversation s'est arrêtée » | Restreindre l'accès annule les requêtes en cours et les commandes lancées | Comportement voulu ; relancer la demande |
| « J'ai cliqué sur Réinitialiser en croyant sécuriser » | Ce bouton remet **tout le disque** | Retirer les dossiers et n'ajouter que les nécessaires |
| « Mon changement d'accélération matérielle ne change rien » | Trois réglages du moteur exigent un redémarrage d'Ollama | Cliquer **Restart Ollama** dans la ligne « Redémarrage nécessaire » |
| « Ollama externe réutilisé » après un redémarrage | Un moteur Ollama tournait déjà, lancé hors de Beaver | Comportement attendu ; voir `06-modeles/ollama-runtime.md` |
| « Je ne vois pas le réglage CPU / GPU » | Il est masqué sur Mac | Comportement attendu |
| « Dossier de sortie invalide. » | Le chemin n'est pas absolu, dépasse 4 096 caractères, ou contient `..` | Choisir le dossier avec le bouton **Choisir** plutôt qu'en le saisissant |

---

## Renvois

- `10-reglages/reference-complete.md` — l'arborescence complète des réglages
- `04-agent/memoire-persistante.md` — ce que la mémoire retient, et comment
- `04-agent/prompts-systeme.md` — ce que contiennent les instructions système
- `04-agent/permissions.md` — les trois modes de permission, distincts des outils
- `05-outils/vue-densemble.md` — chaque outil, expliqué
- `06-modeles/materiel-et-vram.md` — la table de VRAM affichée sous les réglages du moteur
- `06-modeles/ollama-runtime.md` — le cycle de vie du moteur Ollama
- `02-installation/import-depuis-un-autre-assistant.md` — l'écran ouvert par « Gérer »
- `11-securite/modele-de-securite.md` — pourquoi l'accès complet au disque est le défaut
- `12-reference/emplacement-des-donnees.md` — ce qu'ouvre le bouton « Ouvrir le dossier »
- `10-reglages/modeles.md` — les instructions système propres à un modèle Ollama

---

## Points à confirmer

1. **La carte « Compression » n'est pas décrite ici, volontairement.** L'onglet Avancé contient un bloc **« Compression »** entre le comportement général et les réglages Ollama (`advanced-settings.tsx:128-130`), avec plusieurs réglages, des profils nommés et un panneau avancé. **Le chantier de la compression est gelé** (`docs/documentation-site-web/_geles/README.md` : « la compression va être revue »). Documenter ces champs maintenant produirait une page fausse le jour où la fonctionnalité change. **À écrire une fois le gel levé** ; la page devra alors mentionner ce bloc à sa place dans l'onglet, entre les sections 6 et 7 de ce plan.
2. **Le groupe d'outils « Plan mode » est listé sans être expliqué**, pour la même raison : le mode Plan est gelé lui aussi. La ligne du tableau reste — c'est ce que l'écran affiche — mais la page ne doit pas décrire le fonctionnement du mode tant que le gel tient.
3. **« System prompt » ou « Instructions système » ?** L'onglet porte un nom dans la liste et un autre dans son titre. Recommandation : trancher pour **« Instructions système »**, qui est en français et déjà utilisé comme titre, et corriger la clé `settings.tabs.systemPrompt` sur les sept langues. Décision produit.
4. **Le libellé « Réinitialiser » de l'accès fichiers est trompeur** : il élargit les droits au disque entier. Recommandation : signaler l'écart à l'équipe et proposer un libellé qui dit ce qu'il fait, du type « Rendre tout le disque accessible ». La page du site doit de toute façon l'expliquer, quelle que soit la décision.
5. **Aucune annulation après un retrait de dossier**, alors que la règle d'interface du projet demande une annulation plutôt qu'une confirmation pour toute action destructive. Ici l'action est réversible en ajoutant à nouveau le dossier, mais elle a un effet secondaire qui, lui, ne l'est pas : les requêtes annulées et les commandes arrêtées ne reprennent pas. À signaler comme écart.
6. **Le sélecteur de budget mémoire ne propose pas la plus petite valeur acceptée.** Le moteur accepte à partir de **256 tokens** (`memory_types.rs:47`), l'interface ne propose que **512** au minimum. Sans conséquence, mais l'écart est réel.
7. **Le cas d'un groupe d'outils partiellement activé** n'a pas été provoqué. Le raisonnement — l'interrupteur se présente éteint — vient de la lecture du code (`tools-settings.tsx:48-50`), pas d'une observation.
8. **Affichage non vérifié — liste de contrôle pour la passe d'interface finale** : la hauteur de l'onglet Avancé, qui porte six blocs et défile longuement ; la mise en évidence de 1,8 seconde du bloc Accès fichiers quand on y arrive depuis un message d'erreur ; l'apparence de la ligne « Redémarrage nécessaire », qui apparaît et disparaît selon les changements ; la mention « Toujours actif » face aux interrupteurs des autres groupes, dans les deux thèmes ; l'état vide de la liste des mémoires enregistrées.
