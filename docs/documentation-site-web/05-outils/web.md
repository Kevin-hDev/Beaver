# Web — `web_search` et `web_fetch`

**Emplacement site** — Outils › Web
**Répond à** — « Comment l'agent cherche sur Internet et lit une page, et où vont mes requêtes ? »
**Sources** — `tool_web_search.rs`, `tool_web_fetch.rs`, `tool_definitions_web.rs`, `services/search/mod.rs`, `services/search/common.rs`, `services/search/catalog.rs`, `services/searxng/`, `gateway/security/ssrf.rs`, `permission_gate.rs`, `tool_result_truncate.rs`
**Vérification** — Vérifié dans le code, sauf le fonctionnement interne de la recherche locale (voir Points à confirmer)

---

## Plan de page proposé

1. Deux outils, deux usages
2. Chercher sur le web
3. Où part la requête
4. Lire une page
5. Ce que l'agent reçoit d'une page
6. Les protections réseau
7. Ce que l'outil refuse

---

## Contenu

### Deux outils, deux usages

Ils forment le groupe **Web**, **verrouillé** — impossible à désactiver.

| Outil | Rôle | Approbation en mode Demande d'approbation |
|---|---|---|
| `web_search` | Cherche sur le web et renvoie une liste de résultats | Non |
| `web_fetch` | Ouvre une adresse et en extrait le texte | **Oui, systématiquement** |

La différence de traitement est volontaire : chercher n'expose rien, alors qu'ouvrir une adresse fait sortir une requête vers un serveur précis, choisi par le modèle.

### Chercher sur le web

- L'agent envoie une requête, il reçoit une liste : **titre, adresse, extrait**.
- Au plus **10 résultats**. La requête est limitée à **512 caractères**.
- Chaque champ est borné : **160 caractères** pour le titre, **300** pour l'extrait, **2 048** pour l'adresse.
- **L'agent ne choisit pas le moteur de recherche et ne peut pas filtrer par site.** C'est verrouillé dans la définition même de l'outil — un test automatisé vérifie d'ailleurs que les noms des services utilisés n'apparaissent nulle part dans ce que voit le modèle.
- Une recherche sans résultat n'est **pas une erreur** : l'agent reçoit une liste vide et poursuit.

### Où part la requête

C'est la question que se posera tout utilisateur soucieux de confidentialité, et la réponse dépend de ce qui est configuré.

Beaver essaie les sources **dans cet ordre**, et s'arrête à la première qui répond :

1. **Brave Search** — si une clé a été saisie
2. **Exa** — si une clé a été saisie
3. **Firecrawl** — si une clé a été saisie
4. **Une instance de recherche locale**, embarquée dans Beaver — **sans aucune clé**

La quatrième option est le point à mettre en avant : **la recherche web fonctionne sans qu'aucun compte ni aucune clé ne soit configuré**. Beaver embarque son propre moteur, qui tourne sur la machine de l'utilisateur.

Ce que ça change concrètement :

- **Aucun compte n'est associé aux recherches**, puisqu'il n'y a pas de compte à configurer.
- Les moteurs interrogés en aval **voient l'adresse IP de la machine** — le moteur local les interroge directement, sans relais anonymisant. Ce qu'ils ne voient pas, c'est l'identité de l'utilisateur ni le lien entre deux recherches.
- **Beaver ne conserve aucun historique de recherche.**
- En contrepartie, la qualité et la rapidité des résultats sont inférieures à celles d'un service payant dédié, et **le moteur local exige que Python 3 soit installé** sur la machine.

Le détail complet est dans `07-integrations/recherche-web.md`.

Quand une source échoue, Beaver passe à la suivante. Si toutes échouent, l'erreur remontée **agrège les causes** — et elle est expurgée avant affichage : aucune clé, aucun jeton ne peut y apparaître.

### Lire une page

`web_fetch` ouvre une adresse et en extrait le contenu lisible.

- La réponse doit être du **texte** : page web, JSON, XML, texte brut. Une image, un PDF, un fichier binaire sont **refusés** — l'outil ne les convertit pas.
- Pour une **page web**, l'outil isole le contenu principal de l'article et écarte la navigation, les menus et les pieds de page. Quand cette extraction donne un résultat trop maigre — moins d'une centaine de caractères, typiquement une page très interactive — il bascule sur une conversion complète de la page en texte structuré, scripts et styles retirés.
- Pour du **JSON, du XML ou du texte brut**, le contenu est rendu tel quel, sans transformation.
- **Il n'y a aucun cache.** Deux lectures de la même adresse dans la même conversation déclenchent deux requêtes réelles. L'agent voit donc toujours l'état courant de la page.
- **Les adresses en `http://` sont acceptées** telles quelles ; il n'y a pas de passage forcé en `https://`.

### Ce que l'agent reçoit d'une page

Trois plafonds successifs :

| Étape | Limite |
|---|---|
| Corps de la réponse téléchargé | **5 Mo** — au-delà, la lecture est abandonnée |
| Délai de réponse | **15 secondes** |
| Texte transmis au modèle | **50 000 caractères**, puis troncature avec le texte complet écrit sur le disque |

Une page trop lourde renvoie une erreur avant d'être analysée : Beaver ne télécharge pas 200 Mo pour en garder 50 000 caractères.

### Les protections réseau

Toute adresse est validée **avant** la requête, et **à chaque redirection**. C'est le mécanisme le plus élaboré de cette page ; le site peut le résumer en trois lignes et garder le détail pour la page sécurité.

Ce qui est vérifié :

- **Le schéma** — seuls `http` et `https`. Pas de `file://`, pas de `ftp://`, pas de schéma exotique.
- **La longueur** — 2 048 caractères maximum.
- **Les identifiants dans l'adresse** — une adresse contenant un nom d'utilisateur ou un mot de passe est refusée.
- **Les adresses privées** — tout ce qui pointe vers la machine elle-même ou vers le réseau local est bloqué : `localhost`, les plages privées, les adresses de lien local, leurs équivalents en IPv6, ainsi que les écritures détournées en octal ou en hexadécimal.
- **Les serveurs de métadonnées des hébergeurs cloud** — bloqués nommément. Ce sont les adresses qui, sur un serveur hébergé, livrent les identifiants de la machine.
- **Les ports sensibles** — une trentaine de ports de services internes sont refusés : bases de données, partage de fichiers, administration à distance, courrier, annuaires.
- **La résolution du nom de domaine** — le nom est résolu, **toutes** les adresses obtenues sont vérifiées, et la requête est ensuite **épinglée sur l'adresse validée**. Un serveur ne peut donc pas répondre une adresse publique à la vérification puis une adresse interne à la requête.

Les redirections sont suivies **trois fois au maximum**, et chacune repasse l'intégralité de ces contrôles. Une redirection ne peut pas servir de porte dérobée vers le réseau local.

### Ce que l'outil refuse

Les messages d'erreur sont volontairement **génériques** : ils disent qu'une requête a échoué, jamais pourquoi le serveur distant a répondu ce qu'il a répondu. Un message d'erreur détaillé cartographierait le réseau de l'utilisateur.

---

## Tableaux

### Les limites

| Limite | Valeur |
|---|---|
| Longueur d'une requête de recherche | **512 caractères** |
| Résultats de recherche | **10** |
| Longueur d'un titre de résultat | **160 caractères** |
| Longueur d'un extrait | **300 caractères** |
| Longueur d'une adresse | **2 048 caractères** |
| Corps téléchargé par `web_fetch` | **5 Mo** |
| Délai de réponse | **15 secondes** |
| Redirections suivies | **3** |
| Texte transmis au modèle — recherche | **10 000 caractères** |
| Texte transmis au modèle — page | **50 000 caractères** |

### Les sources de recherche

| Source | Clé nécessaire | Nature |
|---|---|---|
| Brave Search | Oui | Moteur de recherche |
| Exa | Oui | Recherche orientée contenu |
| Firecrawl | Oui | Extraction de pages |
| Moteur local embarqué | **Non** | Tourne sur la machine, utilisé en dernier recours |

L'ordre est fixe et ne se configure pas. Configurer une clé Brave revient à la privilégier sur tout le reste.

### Les types de contenu acceptés par `web_fetch`

| Accepté | Refusé |
|---|---|
| Pages web (`text/html`, XHTML) | Images |
| Texte brut, Markdown, CSV | PDF |
| JSON, y compris ses variantes | Documents Office |
| XML | Archives, exécutables, binaires |

Une réponse sans type déclaré est acceptée et traitée comme du texte.

### Ce qui bloque une adresse

| Motif | Message |
|---|---|
| Schéma autre que `http`/`https` | Schéma non autorisé |
| Adresse trop longue | Adresse trop longue |
| Identifiants dans l'adresse | Identifiants interdits |
| Machine locale ou réseau privé | Adresse privée bloquée |
| Serveur de métadonnées d'hébergeur | Métadonnées cloud bloquées |
| Port de service interne | Port non autorisé |
| Nom de domaine non résolu | Résolution impossible |
| Plus de 3 redirections | Trop de redirections |
| Réponse au-delà de 5 Mo | Réponse trop volumineuse |
| Type de contenu non textuel | Type de contenu non supporté |

---

## Encadrés

> **La recherche web marche sans clé et sans compte.**
> Beaver embarque son propre métamoteur, qui tourne localement. Configurer une clé chez un service dédié améliore la qualité des résultats, mais n'est jamais obligatoire.
>
> Sans compte ne signifie pas anonyme : le moteur local interroge des moteurs publics depuis la machine, qui en voient donc l'adresse IP. Ce qui n'existe pas, c'est le compte, le profil et l'historique.

> **`web_fetch` demande toujours une approbation en mode Demande d'approbation.**
> Ouvrir une adresse fait sortir une requête vers un serveur choisi par le modèle. C'est l'un des douze outils qui déclenchent une demande dans tous les cas.

> **Le réseau local est inaccessible à l'agent par cet outil.**
> Un serveur de développement sur la machine, un service interne, une base de données : `web_fetch` refuse. Pour interroger un service local, l'agent passe par une commande shell — qui, elle, est soumise au mode de permission et au bac à sable.

> **Aucune mise en cache.**
> Chaque lecture est une vraie requête. L'agent voit l'état courant d'une page, jamais une version périmée.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent ne peut pas lire mon serveur local » | Les adresses privées sont bloquées | Comportement voulu ; passer par une commande shell |
| « L'agent ne peut pas lire ce PDF en ligne » | Seul le texte est accepté | Télécharger le fichier puis activer le groupe Document |
| « Les résultats de recherche sont médiocres » | Aucune clé configurée : c'est le moteur local qui répond | Ajouter une clé Brave ou Exa dans les réglages |
| « Recherche web indisponible » | Toutes les sources ont échoué | Vérifier la connexion, puis les clés configurées |
| « La page est vide alors qu'elle s'affiche dans mon navigateur » | Page construite entièrement par du code exécuté côté navigateur | Utiliser le navigateur intégré, ou donner une adresse d'API |
| « L'agent me demande une approbation pour chaque page » | `web_fetch` est toujours soumis à approbation | Passer en mode Accès complet, ou approuver au cas par cas |
| « Trop de redirections » | Plus de trois sauts | Donner l'adresse finale directement |

---

## Renvois

- `07-integrations/recherche-web.md` — configurer Brave, Exa, Firecrawl et le moteur local
- `04-agent/permissions.md` — pourquoi `web_fetch` déclenche une demande
- `03-interface/navigateur-integre.md` — pour les pages que `web_fetch` ne sait pas lire
- `11-securite/durcissement.md` — les protections réseau dans la vue d'ensemble sécurité
- `11-securite/confidentialite-des-donnees.md` — ce qui sort de la machine

---

## Points à confirmer

- Le moteur local a été vérifié depuis : c'est une instance de SearXNG lancée localement, qui interroge des moteurs publics **depuis la machine de l'utilisateur** — ils en voient donc l'adresse IP. La page a été corrigée en conséquence. Détail complet dans `07-integrations/recherche-web.md`, qui signale aussi **le prérequis Python non documenté**.
- **La liste des ports bloqués** est exhaustive dans le code mais je ne la reproduis pas ici. Recommandation : donner les catégories sur le site, pas la liste. Publier la liste documente aussi ce qui n'est pas bloqué.
- Je n'ai **pas vérifié à l'écran** l'affichage d'un résultat de recherche ni d'une page récupérée dans la conversation.
- La description interne de `web_fetch` annonce une extraction « de type article » ; le seuil de bascule vers la conversion complète est de 100 caractères. **Très bas** : une page dont l'extraction donne 150 caractères de bruit sera considérée comme réussie. À signaler à l'équipe, sans conséquence pour le site.
