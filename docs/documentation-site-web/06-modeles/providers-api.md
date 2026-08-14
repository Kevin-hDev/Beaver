# Les fournisseurs de modèles par clé API

**Emplacement site** — Modèles › Fournisseurs API
**Répond à** — « Quels services puis-je connecter, où récupérer ma clé, et où va-t-elle ? »
**Sources** — `services/llm/catalog.rs`, `services/api_keys.rs`, `services/vault.rs`, `commands/api_keys.rs`, `services/llm/provider_error.rs`, `src/i18n/fr.json` (clés `apiKeys.providers.*`)
**Vérification** — Vérifié dans le code

> **Aucun tarif ne figure sur cette page ni sur le site.** Les prix changent régulièrement, et Beaver les récupère lui-même d'une source qui se met à jour — publier des chiffres créerait une seconde autorité qui divergerait. Chaque fournisseur est accompagné du lien vers sa page officielle. Décision et raison complètes dans `differents-points-a-traiter.md`.

---

## Plan de page proposé

1. Ce qu'apporte une clé API
2. Les dix fournisseurs
3. Ajouter une clé
4. Où va la clé — et ce qui n'y a jamais accès
5. Tester la connexion
6. Retirer une clé
7. Quand une clé ne marche pas

---

## Contenu

### Ce qu'apporte une clé API

Beaver fonctionne sans aucune clé, avec des modèles locaux. Ajouter une clé donne accès aux modèles distants d'un fournisseur : plus puissants, plus rapides, sans consommer les ressources de la machine — et facturés à l'usage.

Le choix entre local et distant est traité dans `01-decouverte/local-vs-cloud.md`. Cette page traite de la mise en place.

### Les dix fournisseurs

| Fournisseur | Où créer la clé |
|---|---|
| **Groq** | `console.groq.com/keys` |
| **Google Gemini** | `aistudio.google.com/app/apikey` |
| **Mistral** | `console.mistral.ai/api-keys` |
| **Cerebras** | `cloud.cerebras.ai` |
| **OpenRouter** | `openrouter.ai/settings/keys` |
| **OpenAI** | `platform.openai.com/api-keys` |
| **DeepSeek** | `platform.deepseek.com/api_keys` |
| **xAI** | `console.x.ai` |
| **Moonshot Kimi** | `platform.kimi.ai/console/api-keys` |
| **Z.ai GLM** | `z.ai/manage-apikey/apikey-list` |

Ces adresses sont celles que Beaver affiche dans son écran de configuration : un bouton y mène directement, il n'y a pas à les recopier.

**Trois de ces fournisseurs proposent aussi une connexion par compte**, sans clé — voir `06-modeles/providers-comptes-web.md`.

### Ajouter une clé

Le parcours, à décrire pas à pas sur le site :

1. **Réglages › Intégrations › Fournisseurs**.
2. Choisir le fournisseur — sa description et son offre d'entrée sont affichées.
3. Suivre le lien vers la page du fournisseur, créer un compte, générer une clé.
4. Coller la clé dans Beaver.
5. Tester la connexion.

La clé est **validée à la saisie** : format, longueur, caractères. Une clé manifestement erronée est refusée immédiatement, avant tout appel réseau.

Une fois enregistrée, le fournisseur apparaît comme configuré et ses modèles rejoignent le sélecteur de la barre de saisie.

### Où va la clé — et ce qui n'y a jamais accès

C'est le passage qui mérite le plus de soin sur le site : c'est ce qui distingue Beaver d'une application qui écrirait la clé dans un fichier de configuration.

**La clé est chiffrée sur le disque.** Elle vit dans un coffre chiffré avec un algorithme moderne. La clé qui déverrouille ce coffre — la clé maîtresse — n'est pas dans le coffre : elle est confiée au **gestionnaire de mots de passe du système d'exploitation** (Trousseau sur macOS, Gestionnaire d'identifiants sur Windows, portefeuille du bureau sur Linux).

Conséquence : **copier le fichier de coffre sur une autre machine ne donne accès à rien.** Sans la clé maîtresse du système, il est illisible.

**L'interface de Beaver ne voit jamais une clé.** Les commandes disponibles permettent d'enregistrer une clé, de la supprimer, de savoir si elle existe, de lister les fournisseurs configurés et de tester la connexion. **Aucune ne permet de la relire.** Une clé saisie ne peut plus être affichée, y compris par Beaver lui-même.

**En mémoire, la clé est protégée et effacée après usage.** Elle est chargée au moment de l'appel au fournisseur, puis écrasée — elle ne reste pas dans la mémoire de l'application entre deux requêtes.

**Rien de tout cela n'apparaît dans les traces.** Les corps de réponse des fournisseurs sont filtrés et tronqués avant écriture.

### Tester la connexion

Chaque fournisseur dispose d'un test qui envoie une requête minimale et rend un verdict clair. C'est le moyen de distinguer une clé invalide d'un problème de réseau, avant de commencer à travailler.

### Retirer une clé

Supprimer une clé la retire du coffre. Le fournisseur disparaît de la liste des services configurés et ses modèles quittent le sélecteur.

L'écriture dans le coffre est **transactionnelle** : soit l'opération aboutit entièrement, soit le coffre reste dans son état précédent. Une interruption au mauvais moment ne peut pas laisser un coffre à moitié écrit — donc illisible, donc toutes les clés perdues.

### Quand une clé ne marche pas

Les erreurs des fournisseurs sont traduites en messages exploitables, qui distinguent les cas :

| Ce qui se passe | Ce que ça veut dire |
|---|---|
| Authentification refusée | La clé est invalide, révoquée, ou pas celle de ce service |
| Limite de requêtes atteinte | Trop de requêtes en peu de temps — attendre |
| Quota épuisé | Le crédit ou le palier gratuit est consommé |
| Service indisponible | Panne côté fournisseur — réessayer plus tard |
| Délai dépassé | Réseau lent, ou fournisseur saturé |

---

## Tableaux

### Les protections de la clé, résumées

| Question | Réponse |
|---|---|
| Où est-elle stockée | Dans un coffre chiffré, dans les données de l'application |
| Qui détient la clé du coffre | Le gestionnaire de mots de passe du système d'exploitation |
| L'interface peut-elle la lire | **Non** — aucune commande ne l'expose |
| Peut-on la réafficher après saisie | **Non** |
| Reste-t-elle en mémoire | Non — chargée à l'appel, effacée après |
| Apparaît-elle dans les traces | Non — les réponses sont filtrées |
| Copier le fichier suffit-il à la lire | **Non** — la clé maîtresse reste sur la machine d'origine |

### Ce que fait chaque commande

| Commande disponible | Ce qu'elle fait |
|---|---|
| Enregistrer | Valide et chiffre une clé |
| Supprimer | La retire du coffre |
| Vérifier la présence | Répond par oui ou non, sans donner la valeur |
| Lister les fournisseurs configurés | Donne les identifiants, jamais les clés |
| Tester | Envoie une requête minimale et rend un verdict |

**Il n'existe aucune commande de lecture.** C'est volontaire et c'est vérifié : rien dans l'interface ne peut demander la valeur d'une clé.

---

## Encadrés

> **Une clé saisie ne peut plus être affichée.**
> Pas même par Beaver. Si vous la perdez, il faut en générer une nouvelle chez le fournisseur. C'est le prix d'une clé que l'application elle-même ne peut pas lire.

> **Le fichier de coffre est inutile sans la machine.**
> La clé qui le déverrouille vit dans le gestionnaire de mots de passe du système. Copier le fichier ailleurs ne donne accès à rien.

> **Beaver fonctionne sans aucune clé.**
> Les modèles locaux ne demandent aucun compte. Une clé ajoute des possibilités, elle n'est jamais un prérequis.

> **Les prix ne sont pas repris ici.**
> Chaque lien mène à la page tarifaire officielle du fournisseur, qui fait autorité. Beaver affiche de son côté une estimation dans son écran d'usage — voir `06-modeles/usage-et-couts.md`.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Ma clé est refusée à la saisie » | Format invalide, espace ou saut de ligne collé avec | Recopier la clé seule |
| « Authentification refusée » au test | Clé révoquée, ou clé d'un autre service | En générer une nouvelle |
| « Je ne retrouve plus ma clé dans Beaver » | Elle n'est jamais réaffichée, par conception | En générer une nouvelle chez le fournisseur |
| « Limite de requêtes atteinte » | Cadence trop élevée pour le palier du compte | Attendre, ou passer à un palier supérieur |
| « Le modèle que je veux n'apparaît pas » | Le fournisseur ne le propose pas à ce compte | Vérifier le catalogue — voir `06-modeles/catalogue-et-favoris.md` |
| « J'ai changé de machine et mes clés ont disparu » | Le coffre est lié au gestionnaire de mots de passe local | Ressaisir les clés sur la nouvelle machine |

---

## Renvois

- `06-modeles/providers-comptes-web.md` — se connecter par compte plutôt que par clé
- `06-modeles/catalogue-et-favoris.md` — parcourir les modèles disponibles
- `06-modeles/usage-et-couts.md` — suivre sa consommation
- `01-decouverte/local-vs-cloud.md` — quand utiliser un modèle distant
- `11-securite/vault-et-cles-api.md` — le détail du coffre chiffré
- `10-reglages/integrations.md`
- `13-depannage/providers-et-cles.md`

---

## Points à confirmer

- **Les descriptions et les paliers gratuits des fournisseurs vivent dans les fichiers de traduction**, pas dans le code. Deux commentaires du code (`catalog.rs`, vérifiés le 30 juillet 2026) signalent que **les paliers gratuits affichés pour Google et Mistral ne sont plus publiables** : ces fournisseurs ne les affichent plus publiquement, et les chiffres présents dans Beaver reposent sur des sources tierces. **À faire vérifier avant publication** — et c'est un argument de plus pour ne rien chiffrer sur le site.
- **Le lien entre un fournisseur configuré et les modèles réellement disponibles** n'est pas décrit ici : il dépend du catalogue interrogé chez le fournisseur. À traiter dans `catalogue-et-favoris.md`.
- **La liste des messages d'erreur** est reconstituée à partir du classement des erreurs de fournisseurs. Les libellés exacts affichés à l'utilisateur n'ont pas été relevés dans les fichiers de traduction. À compléter.
- Le fournisseur **Z.ai GLM** a une configuration particulière (pas de chemin de catalogue standard). Vérifier que la découverte de ses modèles fonctionne comme pour les autres.
- Affichage à vérifier lors de la passe d'interface : présentation de l'écran des fournisseurs, retour visuel du test de connexion, et ce qui s'affiche à la place d'une clé enregistrée.
