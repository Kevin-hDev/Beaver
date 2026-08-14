# Authentifier un connecteur externe

**Emplacement site** — Intégrations › Authentification des connecteurs
**Répond à** — « Comment j'autorise Beaver à accéder à mon Notion ou à mon GitHub ? »
**Sources** — `services/mcp_oauth/` (`mod.rs`, `flow.rs`, `flow_auth.rs`, `callback_server.rs`, `discovery.rs`, `pkce.rs`, `storage.rs`, `trusted_oauth.rs`, `static_credentials.rs`, `types.rs`), `commands/mcp_oauth.rs`, `services/api_keys_mcp.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Pourquoi une autorisation
2. Le parcours
3. Ce que Beaver ne voit jamais
4. Les protections de l'échange
5. Où vont les jetons
6. Le renouvellement
7. Révoquer l'accès

---

## Contenu

### Pourquoi une autorisation

Un connecteur distant accède à des données personnelles ou d'entreprise : messages, documents, tickets, dépôts. Il faut donc l'autoriser explicitement, service par service.

Beaver utilise le mécanisme standard d'autorisation déléguée : l'utilisateur s'authentifie **chez le service**, qui remet à Beaver un jeton d'accès limité et révocable.

### Le parcours

1. Depuis **Réglages › Intégrations › Connecteurs**, lancer la connexion du service.
2. Beaver ouvre le navigateur sur la page du service.
3. L'utilisateur s'y authentifie et accepte les accès demandés.
4. Le service renvoie vers Beaver, qui reçoit la réponse sur un petit serveur local **actif uniquement pendant la connexion**.
5. Beaver échange cette réponse contre un jeton d'accès et le range dans son coffre.
6. Le navigateur affiche une page de confirmation.

La connexion est abandonnée au bout de **5 minutes** sans réponse. Jusqu'à **5 connexions** peuvent être en cours en même temps, et une connexion déjà lancée pour le même service n'est pas dupliquée.

### Ce que Beaver ne voit jamais

À écrire sans détour, c'est la question que se posent les utilisateurs :

**Beaver ne voit jamais vos identifiants.** Ni le mot de passe, ni le second facteur. L'authentification se déroule entièrement sur le site du service, dans le navigateur.

Ce que Beaver reçoit est un **jeton d'accès** : une autorisation limitée à ce que le service a accepté d'accorder, révocable à tout moment depuis le compte, et sans rapport avec le mot de passe.

### Les protections de l'échange

Le module est nettement durci. Ce qui suit est vérifié.

**Les adresses d'authentification sont vérifiées par service.** Beaver découvre automatiquement les adresses d'autorisation du service, mais **il refuse toute adresse qui ne relève pas du domaine attendu pour ce connecteur précis**. Une adresse Notion présentée pour une connexion Sentry est refusée. C'est ce qui empêche un service compromis de rediriger l'authentification ailleurs.

**Chiffrement obligatoire**, et aucune adresse contenant des identifiants n'est acceptée.

**La demande est liée à la session** par un secret à usage unique généré au moment du départ. Un tiers qui intercepterait la réponse du service ne pourrait rien en faire.

**Le serveur local n'écoute que sur la machine**, sur un port choisi au hasard, uniquement pendant la connexion, et les requêtes qu'il accepte sont bornées en taille.

**Ces vérifications s'appliquent aussi au renouvellement**, pas seulement à la première connexion : l'adresse est revalidée à chaque échange.

### Où vont les jetons

**Dans le coffre chiffré**, comme les clés API et les connexions par compte : chiffrement sur le disque, clé maîtresse dans le gestionnaire de mots de passe du système, aucune commande de lecture exposée à l'interface, mémoire protégée et effacée après usage.

Chaque connecteur a **son entrée distincte**. Déconnecter l'un ne touche pas aux autres.

### Le renouvellement

Un jeton d'accès expire. Beaver le renouvelle automatiquement, **30 secondes avant l'expiration réelle** — cette marge évite qu'une requête partie juste avant échoue en vol.

Deux détails de conception qui évitent des problèmes réels :

- **Un seul renouvellement à la fois par connecteur.** Si plusieurs requêtes constatent en même temps qu'un jeton est périmé, une seule le renouvelle ; les autres attendent et récupèrent le résultat. Sans cela, plusieurs renouvellements simultanés peuvent invalider les jetons les uns des autres — certains services n'acceptent qu'un renouvellement à la fois.
- **Le jeton est revérifié après avoir obtenu le tour.** Un autre appel l'a peut-être déjà renouvelé entre-temps ; dans ce cas, aucun échange inutile n'a lieu.

Quand un service ne renvoie pas de nouveau jeton de renouvellement, l'ancien est conservé — certains services ne le renvoient qu'une fois.

L'utilisateur n'a rien à faire : une autorisation accordée reste valable jusqu'à ce qu'il la révoque ou que le service l'invalide.

### Révoquer l'accès

**Deux endroits, et il faut les distinguer sur le site :**

**Dans Beaver** — la déconnexion efface les jetons du coffre. Le connecteur redevient non authentifié.

**Chez le service** — la page de sécurité du compte permet de retirer l'autorisation accordée à Beaver. C'est la révocation réelle, celle qui vaut même si les jetons subsistaient ailleurs.

Pour couper l'accès complètement, faire les deux.

---

## Tableaux

### Les limites

| Limite | Valeur |
|---|---|
| Délai d'une connexion | **5 minutes** |
| Connexions simultanées | **5** |
| Marge avant renouvellement | **30 secondes** |
| Renouvellements simultanés par connecteur | **1** |
| Taille d'une requête au serveur local | **4 Ko** |
| Délai d'un échange avec le service | **15 secondes** |

### Ce qui est refusé

| Tentative | Résultat |
|---|---|
| Adresse d'authentification hors du domaine du service | **Refusé** |
| Adresse d'un autre connecteur | **Refusé** |
| Adresse non chiffrée | **Refusé** |
| Adresse contenant des identifiants | **Refusé** |
| Réponse sans le secret de session attendu | **Refusé** |
| Réponse trop volumineuse | **Refusé** |

---

## Encadrés

> **Beaver ne voit jamais votre mot de passe.**
> Vous vous authentifiez sur le site du service. Beaver ne reçoit qu'un jeton limité et révocable.

> **Les adresses d'authentification sont vérifiées par service.**
> Beaver refuse toute adresse qui ne relève pas du domaine attendu pour ce connecteur — y compris au renouvellement. Un service compromis ne peut pas détourner l'authentification.

> **Se déconnecter dans Beaver ne suffit pas toujours.**
> Pour retirer réellement l'accès, révoquez aussi l'autorisation depuis la page de sécurité de votre compte chez le service.

> **Le renouvellement est automatique et sérialisé.**
> Un seul renouvellement à la fois par service, ce qui évite que des requêtes simultanées invalident les jetons les unes des autres.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Rien ne se passe après l'autorisation dans le navigateur » | La réponse n'a pas atteint Beaver | Vérifier qu'un pare-feu ne bloque pas les connexions locales |
| « La connexion a expiré » | Plus de 5 minutes | Recommencer |
| « Endpoint OAuth non autorisé » | Le service annonce une adresse hors de son domaine | Refus volontaire ; à signaler si le service est légitime |
| « Le connecteur a cessé de fonctionner » | Autorisation révoquée chez le service, ou mot de passe changé | Se reconnecter |
| « Token expiré et pas de refresh » | Le service n'a pas fourni de jeton de renouvellement | Se reconnecter |
| « J'ai déconnecté mais le service liste toujours Beaver » | La déconnexion locale ne révoque pas côté service | Révoquer depuis le compte |

---

## Renvois

- `07-integrations/mcp-connecteurs.md` — configurer un connecteur
- `05-outils/mcp.md` — comment l'agent s'en sert
- `06-modeles/providers-comptes-web.md` — le même mécanisme pour les modèles
- `11-securite/vault-et-cles-api.md` — le coffre chiffré
- `11-securite/confidentialite-des-donnees.md`
- `13-depannage/mcp-extensions-channels.md`

---

## Points à confirmer

- **Quels connecteurs demandent une autorisation par compte et lesquels une clé** n'est pas explicité ici. Le code contient une liste d'hôtes de confiance pour onze services, ce qui suggère que ce sont ceux qui passent par ce mécanisme. À confirmer et à croiser avec la liste de `mcp-connecteurs.md`.
- **Le module gère des identifiants d'application préenregistrés** pour Google et GitHub, rangés dans le coffre. Leur origine et leur mode de configuration n'ont pas été vérifiés. **À clarifier avec l'équipe avant publication** : si l'utilisateur doit enregistrer sa propre application chez le service, c'est une étape supplémentaire majeure qui doit apparaître dans le parcours.
- **Les accès demandés à chaque service** — la portée de l'autorisation — ne sont pas listés. C'est pourtant ce que l'utilisateur voit sur l'écran d'autorisation, et ce qu'il voudra comprendre avant d'accepter. À compléter service par service.
- **L'enregistrement dynamique auprès du service** est prévu dans le code. Vérifier s'il est réellement utilisé, et pour quels services.
- Affichage à vérifier lors de la passe d'interface : état de connexion d'un connecteur, page de confirmation dans le navigateur, et message affiché en cas de refus d'adresse.
