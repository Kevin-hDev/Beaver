# Les réglages, écran par écran

**Emplacement site** — Réglages › Référence complète
**Répond à** — « Où se règle telle chose dans Beaver ? »
**Sources** — `src/features/extension-ui/core-occupants.tsx` (autorité unique de la liste des onglets), `src/components/settings/settings-sections.ts`, `settings-subtab-list.tsx`, `settings-tab.tsx`, `settings-child-slots.tsx`, `src/types/navigation.ts`, `src/types/extension-ui-contract.generated.ts`, `src/lib/app-shortcuts.ts`, `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, sur la version **1.2.2**. Les libellés cités sont ceux de `src/i18n/fr.json`, relus un par un. Aucune vérification à l'écran (voir « Points à confirmer »).

> **Cette page est un sommaire, pas un manuel.** Chaque onglet a sa page détaillée dans cette même section ; celle-ci sert à retrouver *où* se trouve un réglage, en une seule lecture. Les cinq autres pages de `10-reglages/` disent *ce que fait* chaque réglage.

---

## Plan de page proposé

1. Ouvrir les réglages
2. Comment l'écran est organisé
3. Les cinq sections, d'un coup d'œil
4. Section Préférences
5. Section Agent
6. Section Modèles
7. Section Intégrations
8. Section Application
9. Les onglets qui ont eux-mêmes des sous-onglets
10. Ce qu'une extension peut changer dans cet écran

---

## Contenu

### 1. Ouvrir les réglages

Les réglages sont l'un des **quatre onglets principaux** de Beaver, aux côtés de l'Agent, du Heartbeat et de la Personnalité (`core-occupants.tsx:19-26`).

Deux chemins :

- cliquer sur l'onglet dans la barre latérale ;
- le raccourci clavier **`Cmd` + `,`** sur Mac, **`Ctrl` + `,`** ailleurs (`src/lib/app-shortcuts.ts:36`, identifiant `openSettings`).

**Point de vocabulaire à trancher avant publication.** L'onglet principal s'appelle **« Paramètres »** dans l'application (`fr.json`, clé `nav.settings`), alors que le raccourci correspondant se nomme **« Ouvrir les réglages »** (`fr.json`, clé `settings.shortcuts.openSettings`) et que ce dossier de documentation parle de « réglages ». Voir « Points à confirmer », point 1.

### 2. Comment l'écran est organisé

L'écran se lit en deux colonnes :

- **à gauche**, la liste des onglets, groupés sous cinq en-têtes de section ;
- **à droite**, le contenu de l'onglet sélectionné.

Trois faits vérifiés qui méritent d'être écrits sur le site :

- **Les en-têtes de section ne sont pas cliquables.** Ce n'est pas un oubli : le code les prive volontairement de rôle de bouton et d'ordre de tabulation, pour que la navigation au clavier ne s'arrête pas sur une ligne qui n'ouvre rien (`settings-subtab-list.tsx:20-23`, avec le commentaire d'origine).
- **La liste se parcourt aux flèches du clavier.** Les flèches passent d'un onglet au suivant à travers toutes les sections confondues, dans l'ordre d'affichage (`settings-tab.tsx:65-75`).
- **L'onglet ouvert est mémorisé dans l'état de navigation**, avec le détail de la position dans l'onglet : fournisseur sélectionné, connecteur ouvert, modèle Ollama affiché, vue de l'explorateur LLM (`src/types/navigation.ts:44-62`). Revenir dans les réglages ramène là où l'on était.

### 3. Les cinq sections, d'un coup d'œil

L'ordre des sections et des onglets n'est pas un accident d'affichage : il est déclaré une seule fois, dans une liste que le code appelle explicitement « l'autorité unique des occupants disponibles » (`core-occupants.tsx:14-17`). Tout l'écran en découle.

Le regroupement suit un principe énoncé dans ce même commentaire : **les réglages sont groupés par objet configuré** — les modèles, les connexions, le comportement de l'agent, les préférences, le cycle de vie de l'application.

| Section | Libellé affiché | Onglets | Ce qu'on y règle |
|---|---|---|---|
| `preferences` | **Préférences** | 3 | Ce que vous voyez et comment vous naviguez |
| `agent` | **Agent** | 4 | Ce que l'agent sait, dit et a le droit de faire |
| `models` | **Modèles** | 3 | Les moteurs qui produisent les réponses |
| `integrations` | **Intégrations** | 4 | Ce à quoi Beaver se connecte à l'extérieur |
| `application` | **Application** | 3 | Le logiciel lui-même : version, mises à jour, archives |

**Total : 17 onglets.** Le compte se vérifie en additionnant les entrées de `core-occupants.tsx:28-65`.

### 4. Section Préférences

| Onglet | Ce qu'il contient | Page détaillée |
|---|---|---|
| **Général** | Thème, taille et police de l'interface, thème de code, langue de l'interface, langue de réponse du modèle, aperçu des liens, lancement au démarrage | `10-reglages/general-et-preferences.md` |
| **Mascotte** | Activer la mascotte de bureau, sa taille, et le choix parmi huit personnages | idem |
| **Raccourcis clavier** | La liste des **20 raccourcis** de l'application, en lecture seule | idem |

### 5. Section Agent

| Onglet | Ce qu'il contient | Page détaillée |
|---|---|---|
| **Mémoire** | Mode mémoire, budget de contexte, et la consultation des mémoires enregistrées | `10-reglages/agent.md` |
| **System prompt** | Les instructions système globales : celles de Beaver, celles d'Ollama, ou les vôtres | idem |
| **Outils** | Les **16 groupes d'outils** : 5 toujours actifs, 11 activables | idem |
| **Avancé** | Import depuis un autre assistant, icône dans la barre système, modèle par défaut, compression, moteur Ollama, **accès fichiers**, fichiers des sessions | idem |

**Attention à un déplacement contre-intuitif** : le réglage **« Accès fichiers »**, qui décide des dossiers auxquels l'agent a droit, vit dans l'onglet **Avancé** de la section Agent — pas dans la section Application (`advanced-settings.tsx:142-147`).

### 6. Section Modèles

| Onglet | Ce qu'il contient | Page détaillée |
|---|---|---|
| **Ollama** | Installer et gérer les modèles locaux, et personnaliser leur Modelfile | `10-reglages/modeles.md` |
| **Forecast** | Les modèles de prévision installés et leur configuration | idem |
| **LLM** | Le catalogue consultable des modèles distants : capacités, tailles de contexte, coûts | idem |

L'onglet **Forecast** est le seul dont le libellé ne vient pas de la famille `settings.tabs.*` : il reprend le titre du module, `forecast.title` (`core-occupants.tsx:46-47`). Sans conséquence visible, mais à noter pour qui cherchera la clé.

### 7. Section Intégrations

| Onglet | Ce qu'il contient | Page détaillée |
|---|---|---|
| **Providers** | Vos connexions aux fournisseurs de modèles : clés API d'un côté, comptes web de l'autre | `10-reglages/integrations.md` |
| **Connecteurs** | Les connecteurs MCP, leur configuration et leur autorisation | idem |
| **Canaux** | Les canaux externes qui peuvent parler à Beaver | idem |
| **Extensions** | Les extensions installées, leurs droits, et l'état de leur hôte | idem |

### 8. Section Application

| Onglet | Ce qu'il contient | Page détaillée |
|---|---|---|
| **Mises à jour** | Les versions installées de Beaver et d'Ollama, et l'installation des nouvelles | `10-reglages/application.md` |
| **Chats archivés** | Retrouver, désarchiver ou supprimer définitivement une conversation archivée | idem |
| **À propos** | Version de Beaver, version de Tauri, système, lien vers le dépôt | idem |

### 9. Les onglets qui ont eux-mêmes des sous-onglets

Cinq onglets se subdivisent. Leurs sous-onglets sont déclarés dans les types de navigation (`src/types/navigation.ts:19-22`), et leurs libellés viennent de `fr.json`.

| Onglet | Sous-onglets affichés | Source du libellé |
|---|---|---|
| **Ollama** | **Modelfile** · **Modèles** | `ollama.modelfileTab`, `ollama.modelsTab` (`src/components/ollama/ollama-tab.tsx:35-38`) |
| **Forecast** | **Config** · **Models** | `forecast.modelConfig.sidebarTitle`, `forecast.models.sidebarTitle` (`forecast-settings.tsx:55-58`) |
| **Providers** | **Clés API** · **OAuth** | `providers.tabs.apiKeys`, `providers.tabs.oauth` (`src/components/providers/providers-shell.tsx:18-21`) |
| **Extensions** | **Plugins** · **Extensions** · **Hôte** | `extensions.sections.*` (`src/components/extensions/extension-sections.ts:16-20`) |

**Anomalie de traduction relevée** : dans le fichier français, le second sous-onglet de Forecast s'affiche **« Models »**, en anglais, alors que le premier est « Config ». De même, l'onglet **« Providers »** garde son nom anglais quand ses voisins sont traduits. Voir « Points à confirmer », point 3.

### 10. Ce qu'une extension peut changer dans cet écran

Cette liste d'onglets n'est pas figée : une extension installée peut **ajouter son propre onglet de réglages** dans n'importe laquelle des cinq sections, et il apparaît dans la liste de gauche comme les autres (`settings-sections.ts:73-92`). Les cinq sections sont déclarées comme points d'accueil ouverts dans le contrat des extensions (`extension-ui-contract.generated.ts:11`).

Une extension peut aussi **retirer, remplacer ou déplacer** un onglet existant — avec deux exceptions verrouillées par le contrat lui-même (`extension-ui-contract.generated.ts:12`) :

| Élément protégé | Opérations interdites |
|---|---|
| L'onglet principal **Paramètres** | retrait, remplacement |
| L'onglet de réglages **Extensions** | retrait, remplacement |

Le raisonnement se devine et mérite d'être écrit sur le site : **on ne peut pas installer une extension qui vous enlèverait le moyen de la désinstaller.** Une tentative de mutation sur ces deux éléments est rejetée avec le code de diagnostic `ui_protected_occupant` (`slot-resolution-validation.ts:35-53`).

Une limite chiffrée existe : **128 occupants par emplacement** au maximum (`extension-ui-contract.generated.ts:17`, `maxOccupantsPerPlacement`).

---

## Tableaux

### L'arborescence complète, en un seul tableau

À reprendre tel quel sur le site : c'est l'objet même de la page.

| # | Section | Onglet | Identifiant technique |
|---|---|---|---|
| 1 | Préférences | **Général** | `general` |
| 2 | Préférences | **Mascotte** | `mascot` |
| 3 | Préférences | **Raccourcis clavier** | `shortcuts` |
| 4 | Agent | **Mémoire** | `memory` |
| 5 | Agent | **System prompt** | `system-prompt` |
| 6 | Agent | **Outils** | `tools` |
| 7 | Agent | **Avancé** | `advanced` |
| 8 | Modèles | **Ollama** | `ollama` |
| 9 | Modèles | **Forecast** | `forecast` |
| 10 | Modèles | **LLM** | `llm` |
| 11 | Intégrations | **Providers** | `providers` |
| 12 | Intégrations | **Connecteurs** | `connectors` |
| 13 | Intégrations | **Canaux** | `channels` |
| 14 | Intégrations | **Extensions** | `extensions` |
| 15 | Application | **Mises à jour** | `updates` |
| 16 | Application | **Chats archivés** | `archived-chats` |
| 17 | Application | **À propos** | `about` |

Les identifiants techniques ne sont **pas à publier** : ils servent au rédacteur et à la revérification future (`core-occupants.tsx:28-65`).

### « Je cherche à régler… » — l'index inverse

C'est le tableau le plus utile de la page pour un lecteur pressé.

| Je veux… | Section › Onglet |
|---|---|
| Changer le thème sombre ou clair | Préférences › **Général** |
| Changer la langue de l'application | Préférences › **Général** |
| Faire répondre le modèle dans une autre langue | Préférences › **Général** |
| Lancer Beaver au démarrage de la machine | Préférences › **Général** |
| Agrandir le texte de l'interface | Préférences › **Général** |
| Faire disparaître la mascotte | Préférences › **Mascotte** |
| Retrouver un raccourci clavier | Préférences › **Raccourcis clavier** |
| Empêcher l'agent de retenir des choses | Agent › **Mémoire** |
| Remplacer les instructions système | Agent › **System prompt** |
| Interdire un outil à l'agent | Agent › **Outils** |
| Restreindre les dossiers accessibles à l'agent | Agent › **Avancé** |
| Choisir le modèle par défaut des nouvelles conversations | Agent › **Avancé** |
| Enlever l'icône de la barre système | Agent › **Avancé** |
| Choisir où sont écrits les livrables des sessions | Agent › **Avancé** |
| Ouvrir le dossier de données de Beaver | Agent › **Avancé** |
| Installer ou supprimer un modèle local | Modèles › **Ollama** |
| Décider quand un modèle local libère la mémoire | Agent › **Avancé** |
| Forcer le processeur au lieu de la carte graphique | Agent › **Avancé** |
| Comparer les capacités de deux modèles distants | Modèles › **LLM** |
| Saisir une clé API | Intégrations › **Providers** › Clés API |
| Me connecter avec un compte web | Intégrations › **Providers** › OAuth |
| Ajouter un connecteur MCP | Intégrations › **Connecteurs** |
| Installer une extension | Intégrations › **Extensions** |
| Mettre Beaver à jour | Application › **Mises à jour** |
| Retrouver une conversation archivée | Application › **Chats archivés** |
| Connaître ma version de Beaver | Application › **À propos** |

---

## Encadrés

> **ℹ En tête de page — Cette page répond à une seule question.**
> « Où se règle telle chose ? » Le tableau « Je cherche à régler… » y répond en une ligne ; les pages voisines expliquent ce que le réglage fait vraiment.

> **⚠ Deux réglages ne sont pas là où on les cherche.**
> **Accès fichiers** — les dossiers ouverts à l'agent — est dans **Agent › Avancé**, pas dans Application. Et **tout ce qui concerne le moteur Ollama** — libération de la mémoire, processeur ou carte graphique, plusieurs modèles à la fois — est aussi dans **Agent › Avancé**, pas dans Modèles › Ollama, qui ne gère que les modèles eux-mêmes.

> **ℹ Une extension peut ajouter ses propres réglages.**
> Un onglet supplémentaire peut apparaître dans n'importe laquelle des cinq sections. Il vient de l'extension, pas de Beaver, et disparaît avec elle. Deux éléments ne peuvent jamais être retirés par une extension : l'onglet Paramètres lui-même et l'onglet Extensions — sans quoi on ne pourrait plus désinstaller ce qu'on a installé.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Je ne trouve pas où limiter les dossiers de l'agent » | Le réglage est dans **Agent › Avancé**, pas dans Application | Voir `10-reglages/agent.md` |
| « L'onglet Ollama ne propose aucun réglage du moteur » | Cet onglet gère les modèles ; le moteur se règle dans **Agent › Avancé** | Voir `10-reglages/agent.md` |
| « Un onglet a disparu de mes réglages » | Une extension installée peut retirer un onglet, sauf Paramètres et Extensions | Désactiver l'extension depuis **Intégrations › Extensions** |
| « Un onglet que je ne connais pas est apparu » | Il vient d'une extension installée | Même écran |
| « Le clic sur un titre de section ne fait rien » | Les en-têtes ne sont pas des destinations, volontairement | Comportement attendu |
| « Je cherche un onglet Clés API dans la liste » | Ce n'est plus un onglet de premier niveau : c'est un sous-onglet de **Providers** | Ouvrir Providers, puis **Clés API** |

---

## Renvois

- `10-reglages/general-et-preferences.md` — section Préférences
- `10-reglages/agent.md` — section Agent
- `10-reglages/modeles.md` — section Modèles
- `10-reglages/integrations.md` — section Intégrations
- `10-reglages/application.md` — section Application
- `03-interface/vue-densemble.md` — la disposition générale de l'application
- `03-interface/raccourcis-clavier.md` — les 20 raccourcis, détaillés
- `07-integrations/extensions-centre.md` — ce qu'une extension a le droit de faire

---

## Points à confirmer

1. **« Paramètres » ou « Réglages » ?** L'onglet principal s'affiche **« Paramètres »** (`fr.json`, `nav.settings`), le raccourci s'appelle « Ouvrir les réglages » (`fr.json`, `settings.shortcuts.openSettings`), et ce dossier de documentation écrit « réglages » partout. Le site doit choisir un mot et s'y tenir — recommandation : reprendre **« Paramètres »**, puisque c'est ce que l'utilisateur voit à l'écran, et corriger la clé du raccourci dans l'application. Décision produit, à porter sur sept langues si elle change quelque chose.
2. **Une clé de traduction orpheline subsiste** : `settings.tabs.apiKeys` (« Clés API ») existe toujours dans `fr.json` alors qu'aucun onglet de premier niveau ne la consomme — les clés API sont devenues un sous-onglet de Providers, qui utilise sa propre clé `providers.tabs.apiKeys`. Sans effet visible ; à signaler à l'équipe pour nettoyage.
3. **Deux libellés non traduits en français** : le sous-onglet Forecast **« Models »** (`forecast.models.sidebarTitle`) et l'onglet **« Providers »** (`settings.tabs.providers`). Écrire le site avec ces mots serait fidèle mais bancal ; les traduire dans la documentation les rendrait introuvables à l'écran. Recommandation : **écrire les mots affichés**, et ouvrir une correction côté application. Décision à prendre avant rédaction.
4. **Le compte de 17 onglets vaut pour une installation sans extension.** Une extension peut en ajouter ou en retirer. La page doit donc dire « les dix-sept onglets livrés avec Beaver », pas « les dix-sept onglets ».
5. **Affichage non vérifié — liste de contrôle pour la passe d'interface finale** : la largeur de la colonne de gauche et le comportement des cinq en-têtes de section quand la fenêtre est étroite ; l'apparence d'un onglet ajouté par une extension au milieu des onglets natifs ; le repère visuel de l'onglet actif dans les deux thèmes.
