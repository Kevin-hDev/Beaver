# La recherche web

**Emplacement site** — Intégrations › Recherche web
**Répond à** — « Comment l'agent cherche sur Internet, et est-ce que mes recherches partent chez quelqu'un ? »
**Sources** — `services/search/` (`mod.rs`, `catalog.rs`, `common.rs`, `brave.rs`, `exa.rs`, `firecrawl.rs`), `services/searxng/` (`lifecycle.rs`, `runtime.rs`, `settings.rs`, `client.rs`, `process.rs`, `paths.rs`, `wheels.rs`, `source_filter.rs`), `resources/searxng-sidecar/settings.template.yml`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Quatre sources, un ordre fixe
2. Le moteur local intégré
3. Ce que voient les moteurs interrogés
4. Le prérequis Python
5. Les trois services par clé
6. Configurer une clé
7. Quand la recherche échoue

---

## Contenu

### Quatre sources, un ordre fixe

Beaver essaie les sources dans cet ordre et s'arrête à la première qui renvoie des résultats :

| Ordre | Source | Clé nécessaire |
|---|---|---|
| 1 | **Brave Search** | Oui |
| 2 | **Exa** | Oui |
| 3 | **Firecrawl** | Oui |
| 4 | **Moteur local intégré** | **Non** |

**L'ordre n'est pas configurable.** Configurer une clé Brave revient à la privilégier sur tout le reste.

Une source qui échoue ou qui ne renvoie rien passe la main à la suivante. Si toutes échouent, l'erreur remontée **agrège les causes** — et elle est expurgée avant affichage : aucune clé ne peut y apparaître.

### Le moteur local intégré

C'est le point le plus intéressant de la page, et le moins connu.

Beaver embarque **une véritable instance de SearXNG** — un métamoteur libre — qu'il lance comme un programme séparé sur la machine de l'utilisateur. Ce n'est pas un service distant ni un intermédiaire : c'est un logiciel qui tourne en local.

Comment il est configuré, et ce que ça garantit :

- **Il n'écoute que sur la machine locale**, sur un port choisi au hasard à chaque démarrage. Aucun autre appareil du réseau ne peut l'atteindre.
- **Aucune métrique n'est collectée.** La collecte de statistiques de SearXNG est désactivée.
- **Aucune base de données de session.** Rien n'est mémorisé entre deux recherches.
- **Aucun proxy d'images**, aucun cache d'images.
- **Aucune interface publique.** Le mode instance publique est désactivé.
- Il ne répond qu'en format de données brut, pas en pages web.
- Sa clé de session interne est **générée aléatoirement à chaque démarrage** avec un générateur cryptographique.

Le moteur est **préchauffé au lancement de Beaver**, en arrière-plan, pour que la première recherche ne soit pas ralentie par son démarrage.

Détails de fonctionnement vérifiés :

- Beaver attend jusqu'à **10 secondes** que le moteur soit prêt, en l'interrogeant régulièrement.
- Après un échec de démarrage, il **n'essaie pas de nouveau pendant 30 secondes**. Sans ce délai, chaque recherche relancerait une tentative coûteuse et vouée à l'échec.
- L'identifiant du processus est enregistré ; un moteur orphelin laissé par un arrêt brutal est nettoyé au démarrage suivant.
- Le contenu du moteur est **validé avant lancement** : quatre fichiers attendus doivent être présents, sinon Beaver refuse de démarrer plutôt que d'exécuter un programme incomplet.

### Ce que voient les moteurs interrogés

**C'est le point à formuler avec précision, sans surpromettre.**

Un métamoteur ne possède pas d'index : il interroge d'autres moteurs de recherche publics et agrège leurs réponses. Donc :

- **Les moteurs interrogés voient votre adresse IP**, exactement comme si vous visitiez leur site. Il n'y a pas de relais anonymisant.
- **En revanche, aucun compte n'est associé à vos recherches.** Il n'y a ni clé API, ni identifiant, ni cookie persistant : rien qui relie deux recherches entre elles ni qui les rattache à une personne.
- **L'agrégation se fait sur votre machine.** Aucun tiers ne voit la liste complète de ce que vous avez cherché.
- **Beaver n'enregistre aucun historique de recherche.**

La formulation juste pour le site : *les recherches partent bien vers des moteurs publics, mais sans compte, sans profil et sans historique conservé.* Ce n'est pas de l'anonymat — c'est l'absence d'identification.

Un utilisateur qui veut davantage doit passer par un réseau anonymisant au niveau de son système, ce que Beaver ne fournit pas.

### Le prérequis Python

**Point critique, à faire figurer dans les prérequis d'installation** : le moteur local a besoin d'un **interpréteur Python 3 présent sur la machine**.

Beaver cherche les versions 3.13, 3.12, 3.11, 3.10, puis les commandes génériques. S'il n'en trouve aucune, le moteur local **ne démarre pas** et la recherche sans clé est indisponible.

Ce que Beaver fait ensuite tout seul :

- il crée un environnement Python isolé, à part, dans ses propres données ;
- il y installe les dépendances nécessaires — depuis des paquets **fournis avec l'application** quand ils sont présents, donc **sans accès réseau** ;
- il calcule une empreinte de la source pour ne réinstaller que si elle a changé.

Conséquence pratique :

| Plateforme | Python 3 présent par défaut |
|---|---|
| macOS | Généralement oui, parfois après installation des outils de développement |
| Linux | Presque toujours |
| **Windows** | **Non** |

Sur Windows, la recherche web sans clé peut donc être indisponible tant que Python n'est pas installé. **À vérifier et à documenter dans `02-installation/installation-windows.md`.**

### Les trois services par clé

| Service | Nature | Où créer la clé |
|---|---|---|
| **Brave Search** | Moteur de recherche avec son propre index | `api-dashboard.search.brave.com/app/keys` |
| **Exa** | Recherche orientée sens plutôt que mots-clés | `dashboard.exa.ai/api-keys` |
| **Firecrawl** | Extraction de contenu de pages | `www.firecrawl.dev/app/api-keys` |

Ce qu'ils apportent par rapport au moteur local : des résultats plus rapides, plus pertinents, et un fonctionnement qui ne dépend ni de Python ni du démarrage d'un programme séparé.

Ce qu'ils coûtent : une inscription, une facturation à l'usage, et **le service voit vos requêtes associées à votre compte**.

### Configurer une clé

Même parcours que les fournisseurs de modèles : **Réglages › Intégrations › Fournisseurs**, choisir le service, suivre le lien, coller la clé, tester.

Les clés de recherche sont protégées **exactement comme les clés de modèles** : coffre chiffré, clé maîtresse dans le gestionnaire de mots de passe du système, aucune commande de lecture, effacement de la mémoire après usage.

Chaque service dispose d'un test de connexion dédié.

### Quand la recherche échoue

| Message | Ce que ça veut dire |
|---|---|
| Authentification refusée | Clé invalide ou révoquée |
| Limite de requêtes atteinte | Trop de requêtes en peu de temps |
| Service indisponible | Panne côté service |
| Délai dépassé | Réseau lent |
| Aucun fournisseur configuré, et moteur local indisponible | Aucune source ne fonctionne |

Le dernier cas est celui à expliquer : le message le dit explicitement, et la cause la plus fréquente est l'absence de Python.

---

## Tableaux

### Comparaison des sources

| | Moteur local | Services par clé |
|---|---|---|
| Inscription | **Aucune** | Requise |
| Coût | **Gratuit** | À l'usage |
| Compte associé aux requêtes | **Non** | Oui |
| Adresse IP visible des moteurs | Oui | Non — le service s'interpose |
| Prérequis | **Python 3** | Aucun |
| Vitesse | Plus lente | Rapide |
| Qualité des résultats | Correcte | Meilleure |
| Démarrage | Quelques secondes au premier usage | Immédiat |

### Les limites communes

| Limite | Valeur |
|---|---|
| Longueur d'une requête | **512 caractères** |
| Résultats renvoyés | **10** |
| Longueur d'un titre | **160 caractères** |
| Longueur d'un extrait | **300 caractères** |
| Longueur d'une adresse | **2 048 caractères** |
| Réponse lue d'un service | **512 Ko** |
| Délai d'une recherche locale | **15 secondes** |
| Attente au démarrage du moteur local | **10 secondes** |
| Délai avant nouvelle tentative après échec | **30 secondes** |

---

## Encadrés

> **La recherche web fonctionne sans aucun compte.**
> Beaver embarque son propre métamoteur, qui tourne sur votre machine. Aucune inscription, aucune clé, aucun coût.

> **Sans compte ne veut pas dire anonyme.**
> Le métamoteur interroge des moteurs publics depuis votre machine : ils voient votre adresse IP. Ce qu'ils ne voient pas, c'est qui vous êtes ni ce que vous avez cherché avant.

> **Le moteur local demande Python 3.**
> Présent par défaut sur Linux et généralement sur macOS, **absent sur Windows**. Sans lui, la recherche sans clé est indisponible — il faut alors configurer une clé.

> **Beaver ne conserve aucun historique de recherche.**
> Ni les requêtes, ni les résultats.

> **L'ordre des sources n'est pas configurable.**
> Configurer une clé Brave la rend prioritaire sur tout le reste, y compris sur le moteur local.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Recherche web indisponible » sans clé configurée | Moteur local non démarré, souvent Python absent | Installer Python 3, ou configurer une clé |
| « La première recherche est lente » | Démarrage du moteur local | Normal ; les suivantes sont rapides |
| « Les résultats sont médiocres » | Moteur local | Configurer une clé Brave ou Exa |
| « Ma clé Brave est configurée mais Exa est utilisé » | Brave a échoué ou n'a rien renvoyé | Vérifier la clé Brave |
| « La recherche échoue puis refuse de réessayer » | Délai de 30 secondes après un échec de démarrage | Attendre, puis réessayer |
| « Le moteur local ne redémarre pas après un plantage » | Processus orphelin | Nettoyé au démarrage suivant de Beaver |

---

## Renvois

- `05-outils/web.md` — les outils de recherche et de lecture de page côté agent
- `06-modeles/providers-api.md` — le même parcours de configuration
- `11-securite/vault-et-cles-api.md` — la protection des clés
- `11-securite/confidentialite-des-donnees.md` — ce qui sort de la machine
- `02-installation/prerequis.md` — **le prérequis Python à y ajouter**
- `02-installation/installation-windows.md` — idem
- `12-reference/journaux.md`

---

## Points à confirmer

- **Le prérequis Python n'est documenté nulle part** — ni dans le README, ni dans les prérequis d'installation, ni dans l'interface. Un utilisateur Windows sans Python découvrira que la recherche ne fonctionne pas sans savoir pourquoi. **À remonter à l'équipe en priorité** : soit embarquer un interpréteur, soit l'annoncer dans les prérequis et afficher un message clair.
- **Quels moteurs le métamoteur interroge** dépend de sa configuration par défaut, que Beaver ne modifie pas (`use_default_settings: true`). La liste peut donc évoluer avec la version embarquée. Ne pas la fixer sur le site.
- **La taille du téléchargement et la durée de la première installation** du moteur local n'ont pas été mesurées. À compléter — c'est ce que l'utilisateur voit au premier lancement.
- **Le moteur local est-il arrêté quand il ne sert pas ?** Le code l'arrête à la fermeture de l'application, mais je n'ai pas vu de mise en veille après inactivité. À vérifier : un processus Python qui tourne en permanence consomme de la mémoire.
- Affichage à vérifier lors de la passe d'interface : indication de la source utilisée pour une recherche, et signalement de l'indisponibilité du moteur local.
