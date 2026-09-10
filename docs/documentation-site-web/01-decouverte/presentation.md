# Qu'est-ce que Beaver

**Emplacement site** — Démarrage › Présentation (page d'entrée de la documentation)
**Répond à** — « Je viens d'arriver sur ce site, c'est quoi Beaver et est-ce que ça me concerne ? »
**Sources** — `README.md`, `CHANGELOG.md`, `LICENSE`, `package.json:3`, `src-tauri/Cargo.toml:4`, `src-tauri/tauri.conf.json:4`
**Vérification** — Issu du README pour le positionnement ; version vérifiée dans le code (les trois fichiers qui font autorité) ; licence vérifiée dans les fichiers du dépôt

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
- Il travaille avec des modèles **cloud** (clé API ou compte web) ou **locaux** (via Ollama). Le cloud se cite en premier (corrigé le 10 sept. 2026 avec Kevin) : la page donnait l'impression d'un produit dédié à l'IA locale, ce que Beaver n'est pas — le README fait foi (« The Agent workspace supports both, rather than being dedicated to local AI »).
- Plateformes : **macOS, Windows, Linux**.
- Version courante au moment de la rédaction : **1.2.2**, valeur identique dans les trois fichiers qui font autorité (`package.json:3`, `src-tauri/Cargo.toml:4`, `src-tauri/tauri.conf.json:4`).

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

Les quatre situations à décrire (la première ajoutée le 10 sept. 2026, même correction d'équilibre cloud/local que la section 1) :

- **Utiliser les modèles auxquels on a déjà accès** — un compte web connecté ou une clé API suffit ; l'agent travaille avec le fournisseur choisi, sans changer d'espace de travail.
- **Travailler sur des fichiers locaux** — l'agent ouvre, modifie, lance les commandes, vérifie, propose ses changements.
- **Garder les données sur la machine** — avec un modèle local, aucune conversation ne quitte l'ordinateur ; ni clé API ni compte nécessaires.
- **Automatiser du récurrent** — les réveils programmés lancent une conversation à heure fixe, le résultat attend dans l'application.

### 4. Ce qui le distingue

Cinq arguments, dans cet ordre d'importance (réordonné le 10 sept. 2026 : le multi-fournisseurs passe devant le runtime local, même raison que la correction de la section 1) :

**Cloud et local dans le même espace de travail.**
- Beaver n'est pas dédié à l'IA locale : comptes web connectés, clés API et modèles Ollama se choisissent dans la même interface, conversation par conversation.

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
- Les versions **jusqu'à la 1.1.2 incluse** avaient été publiées sous **Apache License 2.0** ; elles ne sont plus distribuées (corrigé le 10 sept. 2026 : les anciennes releases n'existent plus sur GitHub — la liste des releases commence à la v1.1.9. Les deux README portaient la même phrase fausse, corrigée le 10 sept. 2026).
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

- **La version affichée sur le site.** La version du dépôt est **1.2.2** et ne fait plus débat ; ce qui reste à décider est éditorial : quelle version le site affiche au lancement, et par quel mécanisme cette mention se met à jour à chaque publication.
- **L'adresse de contact commercial.** Présente dans le README ; confirmer qu'elle doit apparaître sur le site public.
- ~~Le nom des modes de permission côté utilisateur.~~ **Tranché** : Accès complet (`auto`), Demande d'approbation (`manual`), Chatbot (`chat`).
