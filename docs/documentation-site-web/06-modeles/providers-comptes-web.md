# Se connecter avec un compte plutôt qu'avec une clé

**Emplacement site** — Modèles › Connexion par compte
**Répond à** — « J'ai déjà un abonnement OpenAI. Puis-je m'en servir dans Beaver sans clé API ? »
**Sources** — `services/oauth_providers/mod.rs`, `services/llm_oauth/` (`mod.rs`, `device_flow.rs`, `store.rs`, `types.rs`, `refresh.rs`, `xai.rs`, `kimi.rs`), `services/codex_oauth/` (`login.rs`, `callback.rs`, `callback_server.rs`, `pkce.rs`, `jwt.rs`, `store.rs`), `commands/oauth_providers.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Deux façons de se connecter
2. Les trois fournisseurs concernés
3. Se connecter à OpenAI
4. Se connecter à xAI ou Kimi
5. Où vont les jetons
6. Le renouvellement automatique
7. Se déconnecter
8. Ce qui peut échouer

---

## Contenu

### Deux façons de se connecter

| | Clé API | Compte |
|---|---|---|
| Ce qu'il faut | Une clé générée sur le site du fournisseur | Ses identifiants habituels |
| Facturation | À l'usage, sur le compte développeur | Sur l'abonnement existant |
| Mise en place | Copier-coller une clé | Se connecter dans le navigateur |
| Expire | Non | **Oui — renouvelé automatiquement** |

La connexion par compte convient à qui a déjà un abonnement et ne veut pas ouvrir de compte développeur ni gérer une facturation séparée.

### Les trois fournisseurs concernés

| Fournisseur | État |
|---|---|
| **OpenAI** | Disponible |
| **xAI** (Grok) | Disponible |
| **Moonshot** (Kimi) | **Expérimental** |

Le marquage « expérimental » de Kimi vient du code et doit apparaître sur le site : c'est une information honnête, et l'utilisateur doit savoir à quoi s'attendre avant d'y consacrer du temps.

### Se connecter à OpenAI

Le parcours suit le mécanisme standard d'autorisation :

1. Beaver ouvre le navigateur sur la page de connexion d'OpenAI.
2. L'utilisateur s'authentifie chez OpenAI — **jamais dans Beaver**.
3. OpenAI renvoie vers Beaver, qui reçoit la réponse sur un petit serveur local, actif uniquement pendant la connexion.
4. Beaver échange cette réponse contre des jetons d'accès.

Deux protections méritent d'être mentionnées :

- **Beaver ne voit jamais le mot de passe.** L'authentification se fait entièrement sur le site du fournisseur.
- La demande est **liée à la session** par un secret à usage unique généré au départ. Un tiers qui intercepterait la réponse ne pourrait rien en faire sans ce secret.

Une fois connecté, Beaver affiche **l'adresse du compte**, extraite du jeton reçu. Elle sert uniquement à savoir quel compte est relié.

### Se connecter à xAI ou Kimi

Ces deux fournisseurs utilisent un mécanisme différent, pensé pour les appareils sans navigateur intégré :

1. Beaver affiche **un code court** et une adresse web.
2. L'utilisateur ouvre cette adresse, se connecte, et saisit le code.
3. Pendant ce temps, Beaver **interroge le fournisseur à intervalle régulier** jusqu'à ce que l'autorisation soit accordée.

L'avancement est affiché en direct dans Beaver : démarrage, attente, succès, annulation, échec.

**La connexion est abandonnée au bout de 15 minutes.** Elle peut être annulée à tout moment ; l'annulation est immédiate et propre.

### Où vont les jetons

**Dans le même coffre chiffré que les clés API**, avec les mêmes protections : chiffrement sur le disque, clé maîtresse dans le gestionnaire de mots de passe du système, aucune commande de lecture exposée à l'interface, effacement de la mémoire après usage.

Chaque fournisseur a son **entrée distincte** dans le coffre : se déconnecter de l'un ne touche pas à l'autre.

Les jetons sont validés avant enregistrement — longueur bornée à **4 096 caractères**, date d'expiration cohérente. Un jeton aberrant est refusé plutôt que stocké.

### Le renouvellement automatique

Un jeton d'accès a une durée de vie limitée. Beaver le renouvelle tout seul, **une minute avant son expiration réelle** — cette marge évite qu'une requête parte avec un jeton qui expire pendant son trajet.

L'utilisateur n'a rien à faire. Une connexion établie reste valable tant qu'il ne se déconnecte pas et que le fournisseur ne révoque pas l'accès.

Un mécanisme de génération protège contre un cas subtil : si l'utilisateur se reconnecte pendant qu'un renouvellement est en cours, **le renouvellement périmé n'écrase pas la nouvelle connexion**. Sans cette protection, une reconnexion pourrait être annulée quelques secondes plus tard par une opération lancée avant elle.

### Se déconnecter

La déconnexion annule toute connexion en cours et efface les jetons du coffre. Le fournisseur passe à l'état non connecté et ses modèles quittent le sélecteur.

Cela ne révoque pas l'accès **chez le fournisseur** : pour cela, il faut passer par la page de sécurité de son compte. Point à préciser sur le site.

### Ce qui peut échouer

| Échec | Ce que ça veut dire |
|---|---|
| Annulé | L'utilisateur a interrompu, ou fermé la fenêtre |
| Refusé | L'autorisation a été refusée sur la page du fournisseur |
| Expiré | Plus de 15 minutes se sont écoulées |
| Non autorisé | Le compte n'a pas accès à ce service |
| Échec | Réseau, ou panne côté fournisseur |

Les messages affichés restent volontairement **génériques** : « Connexion impossible », « Connexion annulée ». Le détail technique va dans les traces, pas à l'écran.

---

## Encadrés

> **Beaver ne voit jamais votre mot de passe.**
> L'authentification a lieu entièrement sur le site du fournisseur. Beaver ne reçoit qu'un jeton d'accès, révocable.

> **Les jetons sont protégés comme les clés API.**
> Même coffre chiffré, même clé maîtresse dans le gestionnaire de mots de passe du système, même impossibilité de les relire depuis l'interface.

> **Le renouvellement est automatique.**
> Une connexion établie n'a pas à être refaite. Beaver renouvelle le jeton avant son expiration, sans intervention.

> **La connexion Kimi est expérimentale.**
> Elle peut être instable ou changer. Une clé API reste l'option fiable pour ce fournisseur.

> **Se déconnecter de Beaver ne révoque pas l'accès chez le fournisseur.**
> Pour couper l'accès entièrement, il faut aussi le faire depuis la page de sécurité du compte.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Rien ne se passe après la connexion dans le navigateur » | La page de retour n'a pas atteint Beaver | Vérifier qu'un pare-feu ne bloque pas les connexions locales, réessayer |
| « Connexion expirée » | Plus de 15 minutes entre le début et la validation | Recommencer |
| « Le code n'est pas accepté » | Code mal recopié, ou déjà expiré | Relancer la connexion pour obtenir un nouveau code |
| « J'étais connecté, je ne le suis plus » | Accès révoqué chez le fournisseur, ou changement de mot de passe | Se reconnecter |
| « Connexion impossible » sans plus de détail | Message volontairement générique | Consulter les traces de l'application |
| « Mes modèles OpenAI n'apparaissent pas » | Connexion établie mais compte sans accès à ces modèles | Vérifier l'abonnement chez le fournisseur |

---

## Renvois

- `06-modeles/providers-api.md` — la connexion par clé
- `06-modeles/catalogue-et-favoris.md` — les modèles disponibles une fois connecté
- `11-securite/vault-et-cles-api.md` — le coffre chiffré
- `07-integrations/mcp-oauth.md` — le même mécanisme pour les connecteurs externes
- `13-depannage/providers-et-cles.md`

---

## Points à confirmer

- **Ce que la connexion par compte donne réellement accès** n'est pas déterminé par Beaver mais par le fournisseur : selon l'abonnement, les modèles disponibles et les limites d'usage diffèrent. Le site doit le dire sans promettre d'équivalence avec une clé API. **À faire préciser par l'équipe**, notamment pour OpenAI où l'écart entre un abonnement grand public et un compte développeur est important.
- **Les limites d'usage propres à ce mode de connexion** — nombre de messages, cadence — ne sont pas visibles dans le code lu. Elles viennent du fournisseur. À vérifier si Beaver les affiche quelque part.
- **Le nom « Codex » apparaît dans le code** pour désigner le mécanisme OpenAI. Ne pas l'employer sur le site sans confirmation : ce nom désigne un produit précis chez OpenAI et pourrait induire en erreur sur ce à quoi on se connecte réellement.
- L'état **expérimental de Kimi** doit être reconfirmé au moment de la publication : il peut avoir changé.
- Affichage à vérifier lors de la passe d'interface : présentation de l'écran de connexion, affichage du code à saisir, indicateur d'avancement, et état connecté avec l'adresse du compte.
