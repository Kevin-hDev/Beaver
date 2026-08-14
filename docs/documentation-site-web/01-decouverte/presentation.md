# Qu'est-ce que Beaver

**Emplacement site** — Démarrage › Présentation (page d'entrée de la documentation)
**Répond à** — « Je viens d'arriver sur ce site, c'est quoi Beaver et est-ce que ça me concerne ? »
**Sources** — `README.md`, `CHANGELOG.md`, `LICENSE`, `package.json` (version), `src-tauri/tauri.conf.json`
**Vérification** — Issu du README pour le positionnement ; version et licence vérifiées dans les fichiers du dépôt

---

## Plan de page proposé

1. Définition en deux phrases
2. Ce que Beaver n'est pas
3. À qui ça s'adresse
4. Ce qui le distingue
5. Ce que fait l'agent concrètement
6. Licence
7. Où aller ensuite

---

## Contenu

### 1. Définition en deux phrases

À dire, dans cet ordre :

- Application **de bureau** — pas un site, pas un outil en ligne de commande.
- Elle héberge un **agent autonome** : il reçoit un objectif, décide des étapes, exécute des outils, constate les résultats, recommence.
- Il travaille avec des modèles **locaux** (via Ollama) ou **distants** (clé API ou compte web).
- Plateformes : **macOS, Windows, Linux**.
- Version courante au moment de la rédaction : **1.1.2** (`package.json`). Une v1.1.3 est décrite au CHANGELOG, et une section *Unreleased* couvre le changement de licence.

Éviter absolument le mot « chatbot » dans cette définition : Beaver possède un mode chat, mais le présenter comme tel désoriente sur ce qui fait sa valeur.

### 2. Ce que Beaver n'est pas

Trois négations utiles, chacune corrigeant une attente fausse fréquente :

| Ce qu'on croit | La réalité |
|---|---|
| « C'est un chat » | Un chat répond puis attend. Beaver exécute des actions en boucle jusqu'à l'objectif. |
| « C'est un outil en ligne de commande » | Tout passe par une interface graphique : conversations, approbations, fichiers, graphiques. |
| « C'est un service hébergé » | L'application s'exécute sur le poste. Conversations, mémoire, plans et clés restent dans un dossier local. |

Précision à ne pas omettre dans la troisième ligne : **ce qui sort de la machine**, ce sont uniquement les requêtes envoyées aux modèles distants et aux services de recherche, quand l'utilisateur en configure. Avec un modèle local et sans recherche web, rien ne sort.

### 3. À qui ça s'adresse

Formuler par situations, pas par métiers — un profil (« développeurs ») exclut à tort, une situation inclut.

Les trois situations à décrire :

- **Travailler sur des fichiers locaux** — l'agent ouvre, modifie, lance les commandes, vérifie, propose ses changements.
- **Garder les données sur la machine** — avec un modèle local, aucune conversation ne quitte l'ordinateur ; ni clé API ni compte nécessaires.
- **Automatiser du récurrent** — les réveils programmés lancent une conversation à heure fixe, le résultat attend dans l'application.

### 4. Ce qui le distingue

Quatre arguments, dans cet ordre d'importance :

**Le runtime local est fourni.**
- Ollama est téléchargé et géré par l'application au premier lancement.
- Rien à installer à part Beaver.
- Si un démon Ollama tourne déjà sur `localhost:11434`, Beaver le réutilise au lieu d'en lancer un second.
- Les modèles sont partagés avec une installation Ollama existante (`~/.ollama/models/`).

**Le contrôle des actions reste à l'utilisateur.**
- Trois modes de permission : **Accès complet**, **Demande d'approbation**, **Chatbot**.
- Le mode Plan pousse plus loin : exploration en lecture seule, plan rédigé en Markdown, aucune écriture avant approbation.

**Les secrets ne transitent pas par l'interface.**
- Coffre chiffré **XChaCha20-Poly1305**.
- Clé maîtresse dans le trousseau du système d'exploitation.
- Aucune commande de l'application ne permet de relire une clé depuis la partie graphique — il n'existe pas de `get_api_key`.
- Le cœur Rust charge la clé au moment de l'appel réseau, puis l'efface de la mémoire.

**Tout est dans la même fenêtre.**
- Conversations, terminal, arbre de fichiers, prévisualisations, navigateur web, historique Git, espace de prévision.

### 5. Ce que fait l'agent concrètement

Liste à donner telle quelle, sans détailler — le détail est dans la section Outils :

- lire, écrire et modifier des fichiers, lister des dossiers
- chercher par nom ou par contenu dans une arborescence
- exécuter des commandes shell, y compris en arrière-plan
- chercher sur le web et récupérer le contenu d'une page
- créer des branches Git et changer de branche
- déléguer une partie du travail à des sous-agents isolés
- tenir une liste de tâches et rédiger un plan
- lire et écrire des tableurs et des documents bureautiques
- transformer des images
- lancer des prévisions de séries temporelles
- appeler les outils de connecteurs externes

Mentionner que certains outils sont actifs par défaut et d'autres s'activent dans les réglages, puis renvoyer.

### 6. Licence

- **GNU Affero General Public License v3.0**.
- Droits : utiliser, étudier, modifier, redistribuer.
- Obligation : toute version **distribuée ou hébergée en réseau**, modifiée ou non, doit être publiée sous AGPL v3 avec son code source complet.
- Les versions **jusqu'à la 1.1.2 incluse** ont été publiées sous **Apache License 2.0** et restent disponibles selon ces termes.
- Une **licence commerciale** exemptant des obligations de l'AGPL est disponible sur demande.
- Les composants tiers gardent leurs licences propres (`THIRD_PARTY_NOTICES.md`).
- Contribuer suppose de signer le **CLA** (`CLA.md`, `CONTRIBUTING.md`).

Le contact commercial figure dans le README. Vérifier avant publication quelle adresse doit apparaître publiquement sur le site — le dépôt en mentionne une, ce n'est pas forcément celle qu'on veut exposer.

### 7. Où aller ensuite

Trois portes de sortie, pas plus :

- Installer → *Installation*
- Comprendre le vocabulaire → *Concepts clés*
- Voir l'étendue → *Tour des fonctionnalités*

---

## Encadrés

**Encadré « Confidentialité »** — à placer après la section 2.
> Avec un modèle local et sans recherche web configurée, aucune donnée ne quitte votre ordinateur. Les conversations, la mémoire et les fichiers restent dans un dossier local.

**Encadré « Licence »** — à placer en section 6, style avertissement.
> Beaver est sous AGPL v3. Si vous distribuez une version modifiée ou si vous l'hébergez comme service accessible par le réseau, vous devez en publier le code source complet sous la même licence.

---

## Pièges et erreurs fréquentes

Aucun à ce stade : la page est descriptive et n'implique aucune manipulation.

Un seul risque de rédaction : **survendre l'autonomie**. Un agent qui exécute des commandes sur les fichiers de quelqu'un mérite une présentation qui mentionne le contrôle dès la page d'accueil, pas trois pages plus loin. Le paragraphe sur les permissions doit apparaître sur cette page.

---

## Renvois

- *Concepts clés* — le vocabulaire employé ici
- *Tour des fonctionnalités* — le panorama complet
- *Modèles locaux, clés API et comptes web* — le choix du modèle
- *Installation* — la mise en route
- *Sécurité* — le détail du coffre et du modèle de menace

---

## Points à confirmer

- **La version affichée sur le site.** `package.json` indique 1.1.2, le CHANGELOG décrit une v1.1.3 et une section *Unreleased*. Déterminer quelle version fait référence au lancement du site, et prévoir comment cette mention se met à jour.
- **L'adresse de contact commercial.** Présente dans le README ; confirmer qu'elle doit apparaître sur le site public.
- ~~Le nom des modes de permission côté utilisateur.~~ **Tranché** : Accès complet (`auto`), Demande d'approbation (`manual`), Chatbot (`chat`).
