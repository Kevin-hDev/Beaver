# Application : Mises à jour, Chats archivés, À propos

**Emplacement site** — Réglages › Application
**Répond à** — « Comment met-on Beaver à jour, comment retrouve-t-on une conversation archivée, et quelle version ai-je exactement ? »
**Sources** — `src/components/settings/updates-settings.tsx`, `archived-chats-settings.tsx`, `archived-chats-groups.tsx`, `archived-chats-bubble.tsx`, `confirm-button.tsx`, `about-settings.tsx` ; `src/hooks/use-update-checker.ts`, `use-archived-agent-sessions.ts` ; `src/lib/brand.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, sur la version **1.2.2**. Aucune vérification à l'écran (voir « Points à confirmer »).

> **Un réglage souvent cherché ici n'y est pas.** L'**accès aux fichiers** — les dossiers ouverts à l'agent — vit dans **Agent › Avancé**, avec les autres réglages du comportement de l'agent. Il est décrit dans `10-reglages/agent.md`.

---

## Plan de page proposé

1. Ce que contient la section Application
2. Onglet Mises à jour
3. Onglet Mises à jour — ce qui n'y figure pas
4. Onglet Chats archivés
5. Onglet Chats archivés — supprimer, et ce que ça veut dire
6. Onglet À propos

---

## Contenu

### 1. Ce que contient la section Application

Trois onglets (`src/features/extension-ui/core-occupants.tsx:60-65`). Ils ont un point commun : ils ne changent pas le comportement de Beaver, ils s'occupent **du logiciel lui-même et de ce qu'il a accumulé**.

| Onglet | Ce qu'on y fait |
|---|---|
| **Mises à jour** | Voir les versions installées, chercher et installer les nouvelles |
| **Chats archivés** | Retrouver, désarchiver ou supprimer définitivement une conversation |
| **À propos** | Lire sa version, celle de Tauri, son système, ouvrir le dépôt |

### 2. Onglet Mises à jour

Le panneau porte le titre **« Mises à jour »** et une phrase de cadrage : « Vérifie et installe les nouvelles versions de Beaver et Ollama. » (`updates-settings.tsx:17`, `:24`).

En haut à droite, un bouton **« Rechercher les mises à jour »**, qui devient **« Recherche… »** pendant l'opération et reste inactif tant qu'elle dure (`updates-settings.tsx:18-22`).

Le corps de l'écran a **une section fixe et une section conditionnelle** :

**« Installé »** — toujours présente. Deux lignes, une par produit, chacune avec son logo, son nom et son numéro de version au format `v1.2.2` (`updates-settings.tsx:27-31`, `:65-76`) :

| Produit | D'où vient la version affichée |
|---|---|
| **Beaver** | La version de l'application elle-même |
| **Ollama** | La version du moteur local réellement installé |

Quand une version n'est pas connue — Ollama pas encore installé, par exemple — la ligne affiche **`—`** (`updates-settings.tsx:73`).

**« Mises à jour disponibles »** — n'apparaît **que s'il y en a** (`updates-settings.tsx:13`, `:33`). Chaque ligne porte alors un bouton **« Mettre à jour »**. Pendant le téléchargement, le bouton laisse place à une **barre de progression avec un bouton d'annulation**, qui affiche « Annulation… » le temps que l'arrêt se fasse (`updates-settings.tsx:40-42`, `:51-53`).

**Trois faits sur le rythme de vérification**, tous vérifiés (`src/hooks/use-update-checker.ts:13`, `:111-112`) :

- une vérification a lieu **au lancement de l'application** ;
- puis **automatiquement toutes les heures** ;
- le bouton de l'écran déclenche la même vérification **à la demande**, et c'est le seul cas où un échec de vérification produit un message d'erreur visible (`use-update-checker.ts:74`, `:97-99`). Les vérifications automatiques échouent en silence.

**Une seule mise à jour à la fois.** Tant qu'un téléchargement est en cours, le bouton de l'autre produit est inactif (`updates-settings.tsx:43`, `:54`, champ `binaryBusy`). Et pendant ce temps, une nouvelle vérification ne remplace pas ce qui est en cours d'installation (`use-update-checker.ts:85-86`, `:91-93`) — sans cela, un téléchargement pourrait viser une version différente de celle annoncée à l'écran.

### 3. Onglet Mises à jour — ce qui n'y figure pas

Deux absences à signaler, parce qu'on les cherche dans cet onglet :

- **Les mises à jour des modèles Ollama** ne sont pas listées ici. L'application les détecte pourtant (`use-update-checker.ts:78`, `check_ollama_updates`), mais cet écran ne montre que **l'application et le moteur**. Les modèles se mettent à jour depuis **Modèles › Ollama**.
- **Il n'y a aucun réglage** dans cet onglet : ni case « mettre à jour automatiquement », ni choix de canal de version. La vérification est automatique et l'installation est toujours manuelle.

Le mécanisme complet — d'où viennent les versions, comment l'installation se déroule, ce qui se passe si elle échoue — est dans `02-installation/mise-a-jour.md`.

### 4. Onglet Chats archivés

Le panneau porte le titre **« Chats archivés »** (`archived-chats-settings.tsx:64`).

En haut, une barre avec deux contrôles (`archived-chats-settings.tsx:76-89`) :

- un champ de recherche **« Rechercher des discussions archivées »**, limité à **120 caractères** ;
- un filtre par projet, dont la première valeur est **« Tous les projets »**.

En haut à droite, un bouton **« Tout supprimer »**, inactif quand il n'y a rien à supprimer (`archived-chats-settings.tsx:65-74`).

**Les conversations sont groupées par projet** (`archived-chats-groups.tsx:16-39`), et à l'intérieur de chaque groupe, **de la plus récemment active à la plus ancienne** (`:27`, `:53-55`). Un groupe vide n'est pas affiché.

Les conversations qui n'appartiennent à aucun projet forment un groupe à part, dont le titre est **« Beaver »** (`fr.json`, clé `projects.discussions`). Le filtre propose ce groupe comme une entrée à part entière, entre « Tous les projets » et la liste des projets réels (`archived-chats-groups.tsx:41-50`).

Chaque conversation propose deux actions : **« Désarchiver »** et **« Supprimer »**.

**Une borne d'affichage existe** : au plus **2 000 conversations archivées** sont présentées (`archived-chats-settings.tsx:17`, `:27`). Au-delà, les plus anciennes ne s'affichent pas. La limite n'est pas signalée à l'écran — voir « Points à confirmer ».

L'onglet a ses états écrits : **« Chargement des archives… »**, **« Aucun chat archivé »**, et la liste elle-même (`archived-chats-settings.tsx:106-109`).

Chaque action produit un message de confirmation ou d'échec : **« Chat désarchivé »**, **« Impossible de désarchiver le chat »**, **« Chat supprimé »**, **« Chats archivés supprimés »**, **« Impossible de supprimer le chat »**.

### 5. Onglet Chats archivés — supprimer, et ce que ça veut dire

C'est le seul écran des réglages où l'on peut détruire des données de façon irréversible, et la page du site doit le dire nettement.

**Deux niveaux de gravité, à ne pas confondre :**

- **Désarchiver** rend la conversation à la liste principale. Rien n'est perdu.
- **Supprimer** l'efface. Les textes de l'application sont explicites : « Supprimer définitivement ce chat archivé ? » et, pour l'action de masse, « Supprimer définitivement tous les chats archivés ? »

**Les deux suppressions demandent une confirmation** : le bouton se transforme en **« Confirmer »**, qu'il faut cliquer une seconde fois (`confirm-button.tsx`, utilisé en `archived-chats-settings.tsx:66-73` et pour chaque ligne).

**Il n'y a pas d'annulation après coup.** C'est un écart assumé par rapport à la règle d'interface du projet, qui préfère une annulation à une confirmation. Ici, une conversation supprimée n'est plus récupérable depuis l'application.

**« Tout supprimer » supprime les conversations une par une, en parallèle** (`archived-chats-settings.tsx:52-60`). Deux conséquences honnêtes à écrire : si l'opération échoue en cours de route, **une partie des conversations peut avoir été supprimée** ; et le message d'erreur affiché est unique — il ne dit pas combien ont été effacées.

**Un point que le site doit clarifier** : ce bouton porte sur **toutes** les conversations archivées, pas seulement sur celles que la recherche ou le filtre affichent au moment du clic (`archived-chats-settings.tsx:53`, qui parcourt la liste complète et non les groupes filtrés). Un utilisateur qui a filtré sur un projet et clique « Tout supprimer » perdra aussi les archives des autres projets.

### 6. Onglet À propos

Le plus simple des dix-sept onglets. Il n'a aucun réglage.

En haut, le logo de Beaver, son nom en typographie de marque, et sa description : **« Application desktop agentique pour LLM locaux et cloud. »** (`about-settings.tsx:32-38` ; `fr.json`, clé `about.description`).

Puis une carte de trois lignes (`about-settings.tsx:40-44`) :

| Ligne | Contenu |
|---|---|
| **Version** | La version de Beaver installée |
| **Tauri** | La version du cadre applicatif sur lequel Beaver est construit |
| **Système** | **macOS**, **Windows** ou **Linux**, selon la machine |

Chacune affiche **`—`** tant que la valeur n'est pas connue (`about-settings.tsx:25-27`, `:41-43`).

Enfin, un bouton **« Voir sur GitHub »** qui ouvre le dépôt du projet dans le navigateur du système, hors de Beaver (`about-settings.tsx:47-53`).

**C'est l'écran à indiquer dans toute demande d'aide** : la version de Beaver et le système sont les deux premières informations à donner pour qu'un problème soit reproductible.

---

## Encadrés

> **⚠ La suppression d'un chat archivé est définitive.**
> Une confirmation est demandée, mais il n'y a pas d'annulation après coup. Désarchiver, en revanche, ne perd rien : la conversation revient simplement dans la liste principale.

> **⚠ « Tout supprimer » ignore vos filtres.**
> Le bouton porte sur **toutes** les conversations archivées, y compris celles que la recherche ou le filtre par projet masquent au moment du clic.

> **ℹ Beaver cherche ses mises à jour tout seul, mais n'installe jamais tout seul.**
> Une vérification a lieu au lancement puis toutes les heures. L'installation reste toujours un clic de votre part, et il n'existe aucun réglage de mise à jour automatique.

> **ℹ Les mises à jour de modèles ne sont pas dans cet onglet.**
> Il ne s'occupe que de Beaver et du moteur Ollama. Les modèles se mettent à jour depuis **Modèles › Ollama**.

> **ℹ Où trouver votre version pour une demande d'aide.**
> **Application › À propos** donne la version de Beaver, celle de Tauri et votre système. Ce sont les trois informations à joindre à tout signalement.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Je cherche où limiter les dossiers accessibles » | Ce réglage est dans **Agent › Avancé** | Voir `10-reglages/agent.md` |
| « La version d'Ollama affiche `—` » | Le moteur n'est pas installé, ou sa version n'a pas pu être lue | Voir **Modèles › Ollama** et `06-modeles/ollama-runtime.md` |
| « Le bouton Mettre à jour est grisé » | Une autre mise à jour est déjà en cours | Attendre la fin, ou l'annuler |
| « J'ai cliqué Rechercher et rien ne s'est passé » | Aucune mise à jour n'est disponible : la section correspondante n'apparaît pas | Comportement attendu |
| « Mon modèle Ollama a une mise à jour qui n'apparaît pas ici » | Cet onglet ne liste que l'application et le moteur | Passer par **Modèles › Ollama** |
| « J'ai supprimé un chat archivé par erreur » | La suppression est définitive et sans annulation | Aucune résolution dans l'application |
| « J'ai filtré sur un projet et tout a été supprimé » | « Tout supprimer » porte sur toutes les archives, filtres compris | Supprimer ligne par ligne quand on ne veut qu'une partie |
| « Mes archives les plus anciennes n'apparaissent pas » | L'écran affiche au plus 2 000 conversations | Utiliser la recherche pour retrouver une conversation précise |
| « Je ne trouve pas mes conversations sans projet » | Elles sont groupées sous le titre **Beaver** | Sélectionner cette entrée dans le filtre |

---

## Renvois

- `10-reglages/reference-complete.md` — l'arborescence complète des réglages
- `10-reglages/agent.md` — l'accès aux fichiers, le dossier de données, les livrables des sessions
- `10-reglages/modeles.md` — la mise à jour des modèles Ollama
- `02-installation/mise-a-jour.md` — le mécanisme complet de mise à jour de Beaver
- `03-interface/conversations-et-onglets.md` — archiver une conversation depuis la liste
- `06-modeles/ollama-runtime.md` — le moteur local et sa version
- `13-depannage/` — quelles informations joindre à un signalement

---

## Points à confirmer

1. **La borne de 2 000 conversations archivées n'est pas signalée à l'écran.** Le code coupe silencieusement la liste (`archived-chats-settings.tsx:17`, `:27`). Un utilisateur qui dépasse ce chiffre ne verra ni message ni indication. À signaler à l'équipe : soit afficher un avertissement, soit documenter la limite sur le site. Recommandation : les deux.
2. **« Tout supprimer » ignore les filtres actifs** — comportement lu dans le code (`archived-chats-settings.tsx:52-60`), non observé. Le risque de perte de données est réel et le libellé ne prévient pas. À vérifier à l'écran, puis à arbitrer avec l'équipe : le texte de confirmation devrait-il dire combien de conversations vont être supprimées ?
3. **Une suppression de masse partiellement échouée laisse un état indéterminé.** Les suppressions sont lancées en parallèle et un seul message d'échec est affiché, sans compte. Le comportement n'a pas été provoqué.
4. **Aucune annulation après suppression**, alors que la règle d'interface du projet demande une annulation plutôt qu'une confirmation pour toute action destructive. Écart à signaler ; à décider si la page du site le mentionne ou reste descriptive.
5. **Les mises à jour de modèles Ollama sont détectées mais pas affichées dans cet onglet.** Elles sont bien récupérées par le même mécanisme (`use-update-checker.ts:78`) sans être rendues dans l'écran. À confirmer avec l'équipe : choix délibéré — les modèles appartiennent à l'onglet Ollama — ou oubli ? La réponse change la formulation de l'encadré correspondant.
6. **L'installation d'une mise à jour n'a pas été observée.** Le déroulé — barre de progression, annulation, redémarrage de l'application — vient de la lecture du code. Le brief `02-installation/mise-a-jour.md` a traité ce mécanisme de plus près : c'est lui qui fait foi en cas de désaccord.
7. **Affichage non vérifié — liste de contrôle pour la passe d'interface finale** : la section « Mises à jour disponibles » qui apparaît et disparaît selon l'état, et la stabilité de la hauteur du panneau quand elle apparaît ; la barre de progression et son bouton d'annulation ; le bouton qui se transforme en « Confirmer », dans les deux thèmes, et sa lisibilité ; l'état vide « Aucun chat archivé » ; le groupement des archives par projet quand il y en a beaucoup ; l'écran À propos, le seul de l'application à mettre en avant le logo et la marque.
