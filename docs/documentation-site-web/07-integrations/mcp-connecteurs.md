# Les connecteurs externes

**Emplacement site** — Intégrations › Connecteurs externes
**Répond à** — « Comment je branche Notion, Linear ou GitHub, et puis-je ajouter le connecteur de mon choix ? »
**Sources** — `services/mcp_bridge/` (`config.rs`, `trusted.rs`, `stdio_catalog.rs`, `stdio_cmd.rs`, `process_manager.rs`, `process_spawn.rs`, `process_env.rs`, `registry.rs`, `schema_limits.rs`, `http.rs`, `env_keys.rs`, `token_validation.rs`), `commands/mcp.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Ce qu'est un connecteur
2. Le catalogue est fermé
3. Les connecteurs disponibles
4. Deux façons de se connecter
5. Ajouter un connecteur
6. Activer par conversation
7. Ce qui est verrouillé
8. La gestion des processus

---

## Contenu

### Ce qu'est un connecteur

Un connecteur donne à l'agent accès à un service extérieur : lire une page de documentation interne, créer un ticket, consulter un dépôt, chercher dans une base de connaissances.

Il repose sur un protocole standard, ce qui évite à Beaver d'écrire du code spécifique par service.

Côté agent, tout passe par un seul outil à deux temps — chercher puis appeler. C'est décrit dans `05-outils/mcp.md`.

### Le catalogue est fermé

**C'est la décision la plus importante de cette page, et elle doit être annoncée d'emblée.**

Beaver **ne permet pas d'ajouter un connecteur arbitraire**. Seuls les connecteurs d'une liste établie peuvent être configurés. Pour les connecteurs qui s'exécutent localement, **la commande exacte est figée dans l'application, argument par argument et version par version** : impossible d'en changer un caractère.

Pourquoi ce choix, à expliquer clairement sur le site — c'est un argument, pas une excuse :

Un connecteur qui s'exécute localement est **du code tiers qui tourne avec les droits de l'utilisateur** et qui reçoit ses jetons d'accès. Permettre d'en installer n'importe lequel revient à permettre l'exécution de n'importe quoi. La plupart des outils comparables laissent cette porte ouverte ; Beaver la ferme.

Ce que ça coûte : un utilisateur qui a besoin d'un connecteur absent de la liste ne peut pas l'ajouter. Il doit passer par le terminal, ou attendre que le connecteur soit intégré.

### Les connecteurs disponibles

**Services distants** — treize, joints par une adresse sécurisée :

| Service | Usage |
|---|---|
| Gmail | Courrier |
| Google Drive | Fichiers |
| Google Agenda | Calendrier |
| Notion | Base de connaissances |
| Slack | Messagerie d'équipe |
| Linear | Suivi de tickets |
| GitHub | Dépôts de code |
| Figma | Conception d'interfaces |
| Canva | Création graphique |
| Lucid | Diagrammes |
| Sentry | Suivi d'erreurs |
| Vercel | Hébergement et déploiement |
| Apify | Extraction de données web |

**Services locaux** — cinq, exécutés sur la machine :

| Service | Usage |
|---|---|
| Context7 | Documentation de bibliothèques à jour |
| Hugging Face | Modèles et jeux de données |
| Product Hunt | Veille produit |
| Reddit | Recherche de discussions |
| iMessage | Messages (macOS) |

### Deux façons de se connecter

| | Service distant | Service local |
|---|---|---|
| Où tourne le code | Chez le service | **Sur votre machine** |
| Connexion | Adresse sécurisée figée | Programme lancé localement |
| Authentification | Le plus souvent par compte | Clé placée dans l'environnement |
| Prérequis | Aucun | Un environnement d'exécution — Node, Python ou Deno selon le connecteur |

Pour les services locaux, le programme d'exécution doit être présent sur la machine. S'il manque, le connecteur ne démarre pas : « environnement requis introuvable ».

### Ajouter un connecteur

Chemin : **Réglages › Intégrations › Connecteurs**.

Le parcours dépend du type :

- **Service distant** — l'adresse est déjà connue de Beaver. Il reste à s'authentifier, le plus souvent par compte (voir `07-integrations/mcp-oauth.md`).
- **Service local** — la commande est déjà connue. Il reste à fournir la clé d'accès du service quand il en demande une.

Un connecteur mal formé est refusé à l'enregistrement : identifiant invalide, adresse non reconnue, commande non conforme, ou aucune des deux fournies.

**Limite : 32 connecteurs.**

### Activer par conversation

Chaque connecteur porte un interrupteur d'activation dans les conversations, indépendant de sa configuration.

C'est une distinction utile à expliquer : un connecteur peut être **configuré et authentifié, mais éteint**. L'agent ne le voit alors pas du tout. Cela permet de garder plusieurs connecteurs prêts et de n'exposer que ceux qui servent à un travail donné — chaque connecteur actif ajoute des outils que l'agent doit considérer.

### Ce qui est verrouillé

Le durcissement est le point fort de ce module. Ce qui suit est vérifié dans le code.

**Pour les services distants :**

- L'adresse doit correspondre **exactement** à celle attendue pour ce connecteur — même hôte, même chemin.
- **Chiffrement obligatoire.** Une adresse non sécurisée est refusée.
- **Ni port personnalisé, ni paramètres** dans l'adresse.
- Un connecteur ne peut pas emprunter l'adresse d'un autre : la correspondance est vérifiée par paire.

**Pour les services locaux :**

- **Trois programmes autorisés seulement** : les lanceurs de paquets Node, Python et Deno. Aucun autre, et surtout **aucun interpréteur de commandes**.
- La commande doit correspondre **exactement** à celle du catalogue, y compris **le numéro de version du paquet**. Une version différente est refusée.
- Chaque argument est validé par motif : **ni point-virgule, ni barre verticale, ni accent grave, ni substitution de commande, ni espace**. Les tentatives d'enchaîner une seconde commande sont rejetées.
- Le connecteur iMessage fait l'objet d'un contrôle renforcé : ses permissions d'exécution sont figées à la liste attendue. **Ajouter la permission d'écriture est refusé.**
- La permission « tout autoriser » est refusée pour tous.

**Pour l'exécution :**

- Le programme est lancé **sans passer par un interpréteur de commandes**, avec ses arguments dans une liste.
- Les jetons sont transmis par variables d'environnement, dont les noms sont validés, et **manipulés en mémoire protégée**.

### La gestion des processus

Les connecteurs locaux tournent comme des programmes séparés, dans une réserve gérée par Beaver :

- **8 processus au maximum.** Au-delà, le connecteur inutilisé depuis le plus longtemps est arrêté.
- **Un processus inactif depuis 10 minutes est arrêté**, et redémarré au besoin.
- Tous sont arrêtés **avec leur arbre de processus** à la fermeture de Beaver.
- Les réponses sont bornées : **128 outils par connecteur**, description limitée à **250 caractères**, nom d'outil à **64**.

---

## Tableaux

### Les limites

| Limite | Valeur |
|---|---|
| Connecteurs configurés | **32** |
| Processus locaux simultanés | **8** |
| Inactivité avant arrêt d'un processus | **10 minutes** |
| Outils remontés par connecteur | **128** |
| Longueur d'une description d'outil | **250 caractères** |
| Longueur d'un nom d'outil | **64 caractères** |
| Longueur d'un identifiant de connecteur | **64 caractères** |
| Outils remontés dans une recherche | **15 par connecteur** |

### Ce qui est refusé

| Tentative | Résultat |
|---|---|
| Adresse d'un service non reconnu | Refusé |
| Adresse non chiffrée | Refusé |
| Adresse avec paramètres ou port | Refusé |
| Adresse d'un autre connecteur | Refusé |
| Programme autre que les trois autorisés | Refusé |
| Commande d'interpréteur | Refusé |
| Version de paquet différente | Refusé |
| Option supplémentaire dans la commande | Refusé |
| Enchaînement de commandes | Refusé |
| Permission « tout autoriser » | Refusé |
| Permission d'écriture ajoutée à iMessage | Refusé |

---

## Encadrés

> **Le catalogue est fermé, et c'est délibéré.**
> Un connecteur local est du code tiers qui tourne avec vos droits et reçoit vos jetons. Beaver n'autorise que des connecteurs vérifiés, à des versions figées. Vous ne pouvez pas en ajouter un arbitraire — c'est une limitation assumée en échange d'une garantie.

> **Les versions sont épinglées.**
> Un connecteur local s'installe toujours à la version prévue par Beaver. Une version différente est refusée, ce qui empêche une mise à jour compromise de s'installer silencieusement.

> **Aucun interpréteur de commandes n'est jamais utilisé.**
> Les programmes sont lancés directement, avec leurs arguments en liste. Aucune chaîne de caractères n'est interprétée comme une commande.

> **Configuré ne veut pas dire actif.**
> Un connecteur peut être authentifié et prêt tout en restant éteint pour les conversations. L'agent ne voit que ce qui est activé.

> **Un connecteur local demande un environnement d'exécution.**
> Node, Python ou Deno selon le connecteur. S'il manque sur la machine, le connecteur ne démarre pas.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le connecteur que je veux n'existe pas » | Catalogue fermé | Passer par le terminal, ou demander son intégration |
| « Environnement requis introuvable » | Node, Python ou Deno absent | L'installer |
| « Endpoint MCP non autorisé » | Adresse non reconnue pour ce connecteur | Utiliser celle proposée par Beaver |
| « Commande MCP non autorisée » | Commande ou version modifiée | Conserver celle du catalogue |
| « L'agent ignore mon connecteur » | Connecteur configuré mais désactivé pour les conversations | L'activer |
| « Le connecteur met du temps à répondre la première fois » | Processus arrêté après 10 minutes d'inactivité, redémarrage | Comportement attendu |
| « Limite de connecteurs atteinte » | 32 configurés | En retirer un |
| « Mon connecteur s'est arrêté tout seul » | 8 processus simultanés dépassés, le plus ancien est arrêté | Comportement attendu, il redémarre au besoin |

---

## Renvois

- `05-outils/mcp.md` — comment l'agent utilise les connecteurs
- `07-integrations/mcp-oauth.md` — l'authentification des services distants
- `11-securite/durcissement.md` — le durcissement dans la vue d'ensemble
- `10-reglages/integrations.md`
- `13-depannage/mcp-extensions-channels.md`

---

## Points à confirmer

- **La liste des dix-huit connecteurs sera périmée dès qu'un connecteur sera ajouté.** Elle vit dans deux fichiers du code. **Décision à prendre pour le site** : reproduire la liste et prévoir sa mise à jour à chaque version, ou renvoyer vers l'écran de l'application. Recommandation : donner quelques exemples représentatifs et renvoyer à l'application pour la liste complète.
- **Le catalogue fermé va-t-il rester fermé ?** C'est une décision produit que le site doit refléter fidèlement. Si une ouverture est prévue, ne pas présenter la fermeture comme un principe.
- **La façon dont un service local reçoit sa clé** — quelles variables d'environnement, quel écran de saisie — n'a pas été détaillée. À compléter.
- **Le connecteur iMessage est propre à macOS** et donne accès aux messages personnels. Il demande un traitement particulier sur le site, avec ses implications de confidentialité énoncées clairement. **À arbitrer avec l'équipe.**
- **Aucun mécanisme de mise à jour des connecteurs locaux** n'a été identifié : les versions étant figées dans le code, une mise à jour de connecteur demande une mise à jour de Beaver. À confirmer, et à mentionner si c'est bien le cas.
- Affichage à vérifier lors de la passe d'interface : liste des connecteurs, indicateur d'état, interrupteur d'activation, et retour d'erreur au démarrage d'un connecteur.
