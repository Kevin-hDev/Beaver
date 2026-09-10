# Modèles : Ollama, Forecast, LLM

**Emplacement site** — Réglages › Modèles
**Répond à** — « Où installe-t-on un modèle local, où configure-t-on un modèle de prévision, et où consulte-t-on ce que vaut un modèle distant ? »
**Sources** — `src/components/ollama/ollama-tab.tsx`, `ollama-modelfile-view.tsx`, `ollama-models-view.tsx`, `ollama-setup-screen.tsx` ; `src/components/settings/forecast-settings.tsx`, `forecast-settings-config.tsx`, `forecast-settings-models.tsx` ; `src/components/settings/llm-explorer.tsx`, `llm-model-detail.tsx`, `llm-model-list.tsx`, `llm-family-grid.tsx`, `llm-types.ts` ; `src/types/navigation.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, sur la version **1.2.2**. Aucune vérification à l'écran (voir « Points à confirmer »).

> **Cette page décrit trois écrans de réglage.** L'installation et la personnalisation des modèles locaux sont détaillées dans `06-modeles/ollama-modeles.md` et `06-modeles/ollama-personnalisation.md` ; le module de prévision aura sa propre section. Ici : la structure des trois onglets et ce qu'on peut y faire.

---

## Plan de page proposé

1. Ce que contient la section Modèles — et ce qu'elle ne contient pas
2. Onglet Ollama
3. Onglet Ollama — ce qui s'affiche quand le moteur n'est pas prêt
4. Onglet Forecast
5. Onglet LLM
6. Trois onglets, trois natures différentes

---

## Contenu

### 1. Ce que contient la section Modèles — et ce qu'elle ne contient pas

Trois onglets (`src/features/extension-ui/core-occupants.tsx:44-49`) :

| Onglet | Nature | Ce qu'on y fait |
|---|---|---|
| **Ollama** | Gestion | Installer, mettre à jour, supprimer et personnaliser les modèles locaux |
| **Forecast** | Gestion et configuration | Installer les modèles de prévision et régler leurs paramètres |
| **LLM** | **Consultation seule** | Consulter le catalogue des modèles distants : contexte, coûts, capacités |

**Ce qui n'est pas là, et qu'on y cherche souvent.** Deux choses :

- **Les réglages du moteur Ollama** — libération de la mémoire, processeur ou carte graphique, plusieurs modèles à la fois — sont dans **Agent › Avancé**, pas ici. L'onglet Ollama gère les modèles, pas le moteur qui les exécute.
- **Les clés API et les comptes** des fournisseurs de modèles distants sont dans **Intégrations › Providers**. L'onglet LLM montre ce qui existe ; il ne connecte rien.

### 2. Onglet Ollama

L'onglet se divise en **deux sous-onglets** (`ollama-tab.tsx:35-38`) :

| Sous-onglet | Contenu |
|---|---|
| **Modèles** | Parcourir le catalogue Ollama par famille, choisir une variante, l'installer |
| **Modelfile** | Personnaliser un modèle déjà installé |

Le sous-onglet **Modèles** propose une recherche et une navigation à deux niveaux : d'abord une famille de modèles, puis une variante précise à l'intérieur de cette famille (`ollama-tab.tsx:96-104`). L'écran de détail d'une variante affiche ses caractéristiques réelles avant installation — la page `06-modeles/materiel-et-vram.md` explique comment les lire.

Le sous-onglet **Modelfile** liste les modèles déjà installés et permet d'en modifier la définition. Quand aucun modèle n'est installé, l'écran affiche **« Aucun modèle installé »** (`ollama-modelfile-view.tsx:33`). C'est là que se règlent aussi les **instructions système propres à un modèle Ollama**, distinctes des instructions globales de **Agent › System prompt** — le détail est dans `06-modeles/ollama-personnalisation.md`.

### 3. Onglet Ollama — ce qui s'affiche quand le moteur n'est pas prêt

Cet onglet est le seul des réglages dont le contenu dépend entièrement de l'état d'un service extérieur. Le code distingue **cinq situations**, et l'écran change du tout au tout selon celle qui s'applique (`ollama-tab.tsx:50-79`) :

| État du moteur | Ce que montre l'onglet |
|---|---|
| **En cours de lecture** | Un message d'attente |
| **État illisible** | Un message d'erreur avec un bouton pour réessayer |
| **Ollama absent** | **L'écran d'installation d'Ollama**, pas la liste des modèles |
| **Installation en cours** | Un message de progression |
| **Réparation nécessaire** | Un message d'erreur avec un bouton pour réessayer |
| **Prêt** | Les deux sous-onglets décrits plus haut |

Un point à écrire clairement sur le site : **le premier passage dans cet onglet, sur une installation neuve, ne montre pas de modèles mais une proposition d'installer Ollama.** Ce n'est pas une panne, et c'est l'un des chemins prévus pour installer le moteur après avoir passé l'étape à l'accueil.

Un bandeau d'information sur l'état du service tourne au-dessus des sous-onglets dans tous les cas où le moteur est prêt (`ollama-tab.tsx:82`, composant `OllamaDaemonNotice`). Les états qu'il distingue et les messages qu'il affiche relèvent de `06-modeles/ollama-runtime.md`, qui traite le cycle de vie du moteur.

### 4. Onglet Forecast

L'onglet porte le titre **« Forecast »** et se divise en deux sous-onglets (`forecast-settings.tsx:55-58`, `:89-95`) :

| Sous-onglet | Libellé affiché | Contenu |
|---|---|---|
| Configuration | **Config** | Les paramètres du modèle de prévision sélectionné |
| Catalogue | **Models** | Les modèles de prévision disponibles, à installer ou désinstaller |

Le sous-onglet **Models** est organisé comme celui d'Ollama : familles, puis modèle. Chaque modèle affiche son état (`fr.json`, clés `forecast.models.*`) : **Installé**, **Non installé**, **Préparation requise**, **Mise à jour requise**, ou pour un modèle distant **Cloud (clé requise)** — voire **Clé API non configurée** quand la clé manque. Les actions sont **Installer**, **Préparer**, **Désinstaller**, cette dernière demandant une confirmation.

La fiche d'un modèle donne, selon le cas : **Taille disque**, **RAM**, **CPU** et **GPU** (supporté ou non), **Licence**, **Bibliothèque**, **Pipeline**, **Horizon max**, **Fréquences** et ses **Capacités du modèle**.

Le sous-onglet **Config** ne concerne que les modèles configurables : il présente les paramètres du modèle choisi, avec un bouton **Éditer**, puis **Sauvegarder** ou **Annuler** (`forecast-settings-config.tsx:93-113`). Chaque paramètre a une description écrite dans l'application — horizon maximum, contexte historique, quantiles, précision, type numérique. Quand aucun modèle configurable n'est installé, l'écran affiche **« Aucun modèle Forecast disponible. »**

Le détail de ces paramètres et de leur effet sur une prévision relève de la section Forecast de la documentation, pas de cette page.

**Un point de cohérence à noter** : le module Forecast n'est utilisable par l'agent que si le groupe d'outils **Forecast** est activé dans **Agent › Outils** — et il est **éteint à l'installation**. Installer un modèle de prévision ici ne suffit donc pas à ce que l'agent s'en serve.

### 5. Onglet LLM

C'est le seul onglet de toute la section qui **ne règle rien**. C'est un catalogue de référence : on y cherche un modèle distant pour savoir ce qu'il sait faire, combien il coûte et quelle taille de contexte il accepte.

L'écran présente une barre de recherche — **« Rechercher un modèle… »** — et un bouton **« Familles »** qui déplie une grille de familles de modèles (`llm-explorer.tsx:106-129`).

Une recherche ou une famille ouvre une liste ; un modèle de cette liste ouvre sa **fiche**, avec un lien **« Retour »** (`llm-model-detail.tsx:17-26`).

La fiche donne, en haut, la clé du modèle, puis son fournisseur et son mode. Ensuite deux cartes.

**Les mesures** — seules celles renseignées apparaissent (`llm-model-detail.tsx:66-73`) :

| Ligne | Ce qu'elle donne | Format |
|---|---|---|
| **Contexte (entrée)** | La taille de la fenêtre d'entrée | En milliers de jetons, par exemple `128K` |
| **Sortie max** | La longueur maximale d'une réponse | Idem |
| **Coût entrée** | Le prix du million de jetons envoyés | En dollars, `$0.50/M` |
| **Coût sortie** | Le prix du million de jetons produits | Idem |

**Les capacités** — cinq lignes, toujours présentes, chacune valant **Oui** ou **Non** (`llm-model-detail.tsx:75-85`) :

| Ligne | Ce qu'elle dit |
|---|---|
| **Vision** | Le modèle accepte-t-il des images |
| **Tools** | Peut-il utiliser des outils — donc travailler en agent |
| **Raisonnement** | Produit-il une phase de réflexion |
| **Prompt caching** | Sait-il réutiliser un contexte déjà envoyé, ce qui réduit le coût |
| **Recherche web** | Dispose-t-il d'une recherche web côté fournisseur |

**Deux précisions honnêtes à faire figurer sur le site :**

- **Les coûts sont ceux du catalogue embarqué dans Beaver**, pas un relevé en direct chez le fournisseur. Ils vieillissent entre deux versions de l'application.
- **La capacité « Tools » est la plus déterminante** pour Beaver : un modèle qui ne sait pas appeler d'outils ne peut pas travailler en agent, quelles que soient ses autres qualités.

Cet écran ne dit pas si vous avez une clé pour ce modèle, ni s'il est utilisable chez vous. C'est **Intégrations › Providers** qui répond à cette question.

### 6. Trois onglets, trois natures différentes

Une phrase de synthèse utile en fin de page :

- **Ollama** installe ce qui tourne **chez vous** ;
- **Forecast** installe et règle ce qui **prévoit** ;
- **LLM** ne fait que **documenter** ce qui tourne **ailleurs**.

---

## Encadrés

> **ℹ L'onglet LLM ne connecte rien.**
> C'est un catalogue de consultation : contexte, coûts et capacités des modèles distants. Pour utiliser l'un d'eux, il faut une clé API ou un compte, dans **Intégrations › Providers**.

> **⚠ Le moteur Ollama ne se règle pas dans l'onglet Ollama.**
> Cet onglet installe et personnalise les modèles. La libération de la mémoire, le choix processeur ou carte graphique et le chargement simultané de plusieurs modèles sont dans **Agent › Avancé**.

> **ℹ Sur une installation neuve, cet onglet propose d'installer Ollama.**
> Tant que le moteur n'est pas là, l'onglet montre l'écran d'installation à la place de la liste des modèles. C'est le comportement attendu, et l'un des chemins prévus pour installer le moteur après l'avoir passé à l'accueil.

> **⚠ Installer un modèle Forecast ne suffit pas à ce que l'agent l'utilise.**
> Le groupe d'outils **Forecast** est éteint à l'installation. Il s'active dans **Agent › Outils**.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'onglet Ollama ne montre aucun modèle, juste un écran d'installation » | Le moteur n'est pas installé | Suivre l'installation proposée ; voir `06-modeles/ollama-runtime.md` |
| « Aucun modèle installé » dans Modelfile | Aucun modèle local n'est présent | Passer au sous-onglet **Modèles** et en installer un |
| « Je ne trouve pas le réglage CPU / GPU dans l'onglet Ollama » | Il est dans **Agent › Avancé**, et masqué sur Mac | Voir `10-reglages/agent.md` |
| « J'ai trouvé mon modèle dans l'onglet LLM mais je ne peux pas l'utiliser » | Cet onglet est un catalogue, pas une connexion | Saisir une clé dans **Intégrations › Providers** |
| « Clé API non configurée » sur un modèle Forecast | Le modèle est distant et aucune clé du fournisseur n'est enregistrée | Même écran |
| « Aucun modèle disponible pour ce lancement. » | Aucun modèle Forecast compatible n'est installé | Installer un modèle dans le sous-onglet **Models** |
| « Aucun modèle Forecast disponible. » dans Config | Aucun modèle **configurable** n'est installé | Idem |
| « L'agent refuse de lancer une prévision » | Le groupe d'outils **Forecast** est éteint | L'activer dans **Agent › Outils** |
| « Les coûts affichés ne correspondent pas à ma facture » | Ce sont ceux du catalogue embarqué dans la version installée | Vérifier chez le fournisseur ; voir `06-modeles/usage-et-couts.md` |

---

## Renvois

- `10-reglages/reference-complete.md` — l'arborescence complète des réglages
- `10-reglages/agent.md` — les réglages du moteur Ollama, et le groupe d'outils Forecast
- `10-reglages/integrations.md` — les clés API et les comptes des fournisseurs
- `06-modeles/ollama-modeles.md` — installer un modèle local et lire ses caractéristiques
- `06-modeles/ollama-personnalisation.md` — le Modelfile et les instructions propres à un modèle
- `06-modeles/ollama-runtime.md` — le cycle de vie du moteur et ses états
- `06-modeles/materiel-et-vram.md` — quel modèle tient dans votre machine
- `06-modeles/catalogue-et-favoris.md` — choisir un modèle au quotidien
- `06-modeles/usage-et-couts.md` — ce que coûte réellement un modèle distant

---

## Points à confirmer

1. **La section Forecast de la documentation n'existe pas encore** au moment où ce fichier est écrit. Les renvois vers elle sont volontairement absents : cette page décrit l'écran de réglage, pas le module. Quand la section Forecast sera écrite, **remplacer les explications de paramètres esquissées ici par un renvoi** plutôt que de les dupliquer.
2. **Le sous-onglet Forecast s'affiche « Models », en anglais**, dans le fichier français (`fr.json`, clé `forecast.models.sidebarTitle`), alors que son voisin s'affiche « Config ». Écrire le mot affiché ou le traduire est une décision à prendre pour tout le site — voir `10-reglages/reference-complete.md`, point 3.
3. **Les cinq états de l'onglet Ollama n'ont pas été provoqués.** Le tableau vient de la lecture du code (`ollama-tab.tsx:50-79`) ; l'enchaînement visible — notamment le passage de « installation en cours » à « prêt » — n'a pas été observé, et les messages exacts de progression n'ont pas été relevés.
4. **La liste des capacités de la fiche LLM est fixe**, ce qui signifie qu'un modèle sans aucune de ces cinq capacités affichera cinq lignes à « Non ». C'est un choix d'affichage à vérifier lors de la passe d'interface : cinq « Non » d'affilée se lisent-ils comme une information ou comme une panne ?
5. **La fraîcheur du catalogue LLM** n'a pas été vérifiée : le rythme de mise à jour du catalogue embarqué et sa date de dernière révision restent à établir avec l'équipe avant d'écrire une phrase engageante sur les coûts.
6. **Affichage non vérifié — liste de contrôle pour la passe d'interface finale** : la grille des familles de modèles dans l'onglet LLM et son comportement en fenêtre étroite ; l'alignement des lignes de la fiche de modèle, dont les valeurs sont en chasse fixe alignées à droite ; le bandeau d'état du moteur au-dessus des sous-onglets Ollama ; l'écran d'installation d'Ollama vu depuis les réglages plutôt que depuis l'accueil.
