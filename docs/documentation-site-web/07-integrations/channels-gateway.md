# Piloter l'agent depuis Telegram, Slack ou Discord

**Emplacement site** — Intégrations › Canaux externes
**Répond à** — « Puis-je parler à mon agent depuis mon téléphone, par messagerie ? »
**Sources** — `services/gateway/` (`service.rs`, `service_runtime.rs`, `supervisor.rs`, `watchdog.rs`, `config_validation.rs`, `session_map.rs`, `agent_bridge.rs`, `message_convert.rs`, `tokens.rs`, `token_probe.rs`, `conversation_locks.rs`, `stream_capture.rs`, `service_audit.rs`), `services/gateway/security/` (`allowlist.rs`, `rate_limit.rs`, `audit.rs`, `ids.rs`, `validation.rs`, `secrets.rs`), `services/gateway/channels/`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. À quoi ça sert
2. Les trois messageries
3. Ce qu'il faut mettre en place
4. La liste d'autorisation est obligatoire
5. Comment les conversations sont reliées
6. Les limites de débit
7. Le journal d'audit
8. La surveillance du service
9. Ce qui n'est pas possible

---

## Contenu

### À quoi ça sert

Le gateway permet d'envoyer un message à son agent Beaver depuis une messagerie, et de recevoir sa réponse au même endroit. L'agent tourne toujours sur la machine de l'utilisateur — c'est seulement le point d'entrée qui change.

L'usage typique : lancer une tâche depuis son téléphone, ou consulter l'avancement d'un travail long sans être devant l'ordinateur.

**Beaver doit rester ouvert sur la machine.** Le gateway n'est pas un service hébergé : c'est l'application locale qui écoute.

### Les trois messageries

| Messagerie | Ce qu'il faut créer |
|---|---|
| **Telegram** | Un bot |
| **Slack** | Une application dans l'espace de travail |
| **Discord** | Une application et un bot |

Jusqu'à **16 comptes par messagerie** peuvent être configurés — plusieurs bots, plusieurs espaces de travail.

### Ce qu'il faut mettre en place

Le parcours, à détailler par messagerie sur le site :

1. Créer le bot ou l'application chez la messagerie.
2. Récupérer son jeton.
3. Le saisir dans **Réglages › Intégrations › Canaux**.
4. **Déclarer les utilisateurs autorisés** — obligatoire, voir ci-dessous.
5. Choisir le fournisseur et le modèle qui serviront à répondre.
6. Activer le compte.

Le jeton est vérifié auprès de la messagerie avant activation : cela distingue un jeton invalide d'un problème de réseau, avant de commencer.

Les jetons sont protégés **comme les clés API** : coffre chiffré, clé maîtresse dans le gestionnaire de mots de passe du système, aucune lecture possible depuis l'interface.

### La liste d'autorisation est obligatoire

**C'est la protection la plus importante de cette page, et elle n'est pas contournable.**

Un canal actif **doit** déclarer au moins un utilisateur autorisé. Un canal activé avec une liste vide est **refusé à l'enregistrement**.

Et **le joker est interdit** : il n'existe aucune façon d'écrire « tout le monde ». Chaque utilisateur autorisé doit être désigné par son identifiant.

Pourquoi cela compte, à écrire noir sur blanc : un bot de messagerie est joignable par n'importe qui connaissant son nom. Sans liste d'autorisation, ouvrir un canal reviendrait à **donner à des inconnus un agent qui exécute des commandes sur votre machine**. Beaver rend cette configuration impossible.

**Jusqu'à 100 utilisateurs autorisés** par compte. Les doublons sont refusés, et chaque identifiant est validé.

### Comment les conversations sont reliées

Chaque interlocuteur d'une messagerie est associé à **une conversation Beaver**, de façon stable : écrire deux fois depuis le même endroit continue la même conversation, avec son historique.

Cette correspondance est enregistrée sur le disque et **survit au redémarrage** de l'application.

Elle est **bornée** : au-delà du nombre maximal configuré, la correspondance la plus ancienne est retirée. L'interlocuteur concerné repart alors sur une conversation neuve. La limite se règle, jusqu'à **1 000**.

Le format de ce fichier porte un **numéro de version**, et une version antérieure n'est pas réutilisée : elle est ignorée et repart de zéro plutôt que d'être réinterprétée de travers.

### Les limites de débit

Trois plafonds indépendants, tous réglables :

| Plafond | Ce qu'il protège |
|---|---|
| **Par utilisateur et par minute** | Un utilisateur seul ne peut pas saturer le service |
| **Par canal et par minute** | Un canal actif ne prive pas les autres |
| **Global par minute** | La machine reste utilisable |

Chacun accepte jusqu'à 10 000 par minute, mais aucun ne peut valoir zéro : **il y a toujours une limite active**.

Un message est par ailleurs limité à **12 000 caractères** au maximum configurable.

### Le journal d'audit

Toute activité du gateway est consignée : qui a écrit, quand, sur quel canal, et ce qui a été décidé — message traité, utilisateur refusé, limite atteinte.

La durée de conservation se règle, jusqu'à **365 jours**.

C'est le seul moyen de répondre à « qui a parlé à mon agent, et quand ». Sur une fonctionnalité qui expose l'agent à l'extérieur, ce journal n'est pas un accessoire.

### La surveillance du service

Le gateway tourne en arrière-plan et son fonctionnement est surveillé : un superviseur le redémarre s'il s'arrête, et un chien de garde détecte les blocages.

Deux protections méritent une mention :

- **Une conversation est verrouillée pendant son traitement.** Deux messages arrivés en même temps depuis le même endroit ne sont pas traités en parallèle, ce qui éviterait deux réponses entremêlées dans la même conversation.
- **La réponse est capturée puis envoyée**, plutôt que diffusée mot à mot : une messagerie n'accepte pas des centaines de modifications successives d'un même message.

### Ce qui n'est pas possible

À énoncer clairement pour éviter les déceptions :

- **Les connexions par compte ne peuvent pas servir de modèle au gateway.** Elles supposent une personne devant l'écran pour s'authentifier, ce qui n'est pas le cas d'un message arrivé la nuit. Il faut une clé API, ou un modèle local.
- **Beaver doit être ouvert.** Fermer l'application coupe le canal.
- **Aucun joker dans la liste d'autorisation.**
- Un canal actif sans utilisateur autorisé est refusé.

---

## Tableaux

### Les limites

| Limite | Valeur maximale |
|---|---|
| Comptes par messagerie | **16** |
| Utilisateurs autorisés par compte | **100** |
| Conversations associées | **1 000** |
| Caractères par message | **12 000** |
| Messages par utilisateur et par minute | **10 000** |
| Messages par canal et par minute | **10 000** |
| Messages par minute au total | **10 000** |
| Conservation du journal d'audit | **365 jours** |
| Longueur d'un nom de fournisseur ou de modèle | **128 caractères** |

Aucune de ces limites ne peut valoir zéro : une valeur nulle est refusée.

### Ce qui est refusé à la configuration

| Tentative | Résultat |
|---|---|
| Canal actif sans utilisateur autorisé | **Refusé** |
| Joker dans la liste d'autorisation | **Refusé** |
| Utilisateur en double | Refusé |
| Identifiant de compte en double | Refusé |
| Plus de 16 comptes pour une messagerie | Refusé |
| Plus de 100 utilisateurs autorisés | Refusé |
| Limite de débit à zéro | Refusé |
| Fournisseur exigeant une authentification interactive | **Refusé** |

---

## Encadrés

> **La liste des utilisateurs autorisés est obligatoire, sans joker possible.**
> Un bot de messagerie est joignable par quiconque connaît son nom. Sans cette liste, ouvrir un canal donnerait à des inconnus un agent qui exécute des commandes sur votre machine. Beaver refuse cette configuration.

> **Beaver doit rester ouvert.**
> Ce n'est pas un service hébergé : c'est votre application qui écoute. Fermée, le canal ne répond plus.

> **Une connexion par compte ne peut pas servir ici.**
> Ces connexions supposent quelqu'un devant l'écran. Pour un canal, il faut une clé API ou un modèle local.

> **Tout est consigné.**
> Chaque message, chaque refus, chaque limite atteinte. C'est le seul moyen de savoir qui a parlé à votre agent.

> **L'agent a les mêmes pouvoirs que devant l'écran.**
> Un message reçu par messagerie lance un agent qui accède à vos fichiers et lance des commandes. Le mode de permission et la portée d'accès disque s'appliquent — et méritent d'être revus avant d'ouvrir un canal.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Configuration Gateway invalide » à l'activation | Liste d'autorisation vide, joker, ou doublon | Déclarer au moins un utilisateur, sans joker |
| « Le bot ne répond pas » | Beaver fermé, ou canal désactivé | Ouvrir l'application, vérifier l'activation |
| « Mon message est ignoré » | Utilisateur absent de la liste — visible dans le journal d'audit | L'y ajouter |
| « Le bot répond puis s'arrête » | Limite de débit atteinte | Relever la limite, ou attendre |
| « Je ne peux pas choisir mon modèle habituel » | Modèle accessible par connexion de compte | Utiliser une clé API ou un modèle local |
| « L'agent a perdu le fil » | Correspondance retirée après dépassement de la limite | Relever le nombre de conversations |
| « Deux réponses se mélangent » | Ne devrait pas arriver : les conversations sont verrouillées | À signaler |
| « Ma réponse arrive d'un coup, pas mot à mot » | Comportement voulu : les messageries n'acceptent pas la diffusion continue | Normal |

---

## Renvois

- `04-agent/permissions.md` — **à revoir avant d'ouvrir un canal**
- `04-agent/repertoire-de-travail.md` — la portée d'accès de l'agent
- `06-modeles/providers-api.md` — configurer une clé utilisable par le gateway
- `11-securite/modele-de-securite.md`
- `12-reference/journaux.md` — le journal d'audit
- `10-reglages/integrations.md`
- `13-depannage/mcp-extensions-channels.md`

---

## Points à confirmer

- **Le parcours de création du bot ou de l'application** diffère pour chacune des trois messageries, et c'est la partie la plus longue pour l'utilisateur : où cliquer, quelles permissions accorder, où trouver le jeton, comment obtenir son propre identifiant d'utilisateur pour la liste d'autorisation. **Rien de tout cela n'est dans le code** — ce sont des parcours chez des tiers. **À rédiger avec des captures, et c'est la partie la plus périssable du site** : ces interfaces changent souvent.
- **Comment l'utilisateur trouve son identifiant** dans chaque messagerie doit être expliqué pas à pas. Sans cela, la liste d'autorisation est infranchissable.
- **Le mode de permission appliqué aux messages du gateway** n'a pas été vérifié. C'est une question de sécurité majeure : un message reçu à distance déclenche-t-il les demandes d'approbation, et si oui, comment y répondre depuis une messagerie ? **À clarifier avant publication** — c'est la question qu'un lecteur attentif posera immédiatement.
- **Les valeurs par défaut** des limites de débit, du nombre de conversations et de la rétention d'audit n'ont pas été relevées ; seuls les plafonds sont dans le code lu. À compléter.
- **Ce qui se passe pour une pièce jointe** envoyée par messagerie, ou pour un message trop long, n'a pas été vérifié.
- Affichage à vérifier lors de la passe d'interface : écran de configuration d'un canal, retour du test de jeton, consultation du journal d'audit.
