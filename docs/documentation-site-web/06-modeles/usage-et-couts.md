# Suivre sa consommation

**Emplacement site** — Modèles › Usage et coûts
**Répond à** — « Combien ai-je consommé, et d'où viennent les chiffres affichés ? »
**Sources** — `services/provider_usage/` (`mod.rs`, `pricing.rs`, `ledger.rs`, `ledger_aggregate.rs`, `remote.rs`, `request_journal.rs`, `request_measurement.rs`, `snapshot.rs`, `credential_epoch.rs`, `types.rs`), `services/llm/model_pricing.rs`, `services/llm/litellm_catalog_refresh.rs`, `services/llm/stream_metrics.rs`
**Vérification** — Vérifié dans le code

> **Aucun tarif ne figure sur cette page.** Elle explique comment lire les chiffres de Beaver, pas combien coûte tel modèle. Décision et raison dans `differents-points-a-traiter.md`.

---

## Plan de page proposé

1. Ce que Beaver mesure
2. Les trois sources de chiffres
3. Coût exact et coût estimé
4. D'où viennent les tarifs
5. Ce qui n'est pas chiffré
6. Le journal des requêtes
7. Changer de clé remet les compteurs distants à zéro

---

## Contenu

### Ce que Beaver mesure

Chaque requête envoyée à un fournisseur est mesurée : le modèle utilisé, les jetons consommés en entrée et en sortie, la durée, l'issue — terminée, interrompue, annulée, échouée.

Ces mesures sont enregistrées **localement**, dans les données de l'application. Elles ne sont transmises à personne.

L'écran d'usage se met à jour **en direct** : dès qu'une requête se termine, les compteurs bougent.

### Les trois sources de chiffres

C'est le point qui explique la plupart des questions d'utilisateurs, et il mérite d'être posé clairement sur le site.

**1. Ce que Beaver a compté lui-même.** Le registre local, alimenté requête après requête. Il est complet pour l'usage fait *depuis Beaver*, et il ne connaît rien d'autre.

**2. Ce que le fournisseur annonce.** Certains fournisseurs renvoient, dans leurs réponses, l'état des limites du compte : requêtes restantes, quota, fenêtre de renouvellement. Beaver les lit et les affiche.

**3. Le détail requête par requête.** Un journal des requêtes récentes, avec leur modèle, leur issue et leur consommation.

**La différence entre 1 et 2 est normale et doit être expliquée** : le registre local ne compte que ce qui est passé par Beaver. Si le même compte sert ailleurs — un autre outil, un script, le site du fournisseur — les chiffres du fournisseur seront plus élevés. Aucun des deux n'a tort.

### Coût exact et coût estimé

Beaver distingue les deux, et c'est une distinction honnête à mettre en avant.

**Coût exact** — certains fournisseurs renvoient le coût réel de la requête dans leur réponse. Beaver l'enregistre tel quel. C'est un chiffre facturé, pas une estimation.

**Coût estimé** — sinon, Beaver le calcule : jetons d'entrée, jetons de sortie, et jetons lus ou écrits dans le cache, chacun à son propre tarif. Le calcul est fin — il ne facture pas au prix plein les jetons relus depuis un cache — mais il reste une estimation.

Un garde-fou mérite d'être mentionné : si le calcul donne un résultat aberrant — infini, négatif, ou supérieur à un million de dollars — **Beaver renonce à afficher un coût** plutôt que d'afficher une absurdité. Un chiffre manquant vaut mieux qu'un chiffre faux.

### D'où viennent les tarifs

Beaver **ne code aucun tarif en dur**. Il télécharge un catalogue public de tarifs, le conserve localement et le rafraîchit en ne retéléchargeant que si la source a changé.

Trois protections encadrent ce téléchargement :

- seule la source attendue est acceptée ;
- la taille du fichier est bornée ;
- **un catalogue de moins de cent entrées est rejeté** — une source tronquée ou vidée n'écrase pas un catalogue valide.

Conséquence pour l'utilisateur : **les tarifs restent à jour sans qu'il ait rien à faire**, et sans que Beaver ait besoin d'une mise à jour.

Conséquence pour le site : c'est la raison pour laquelle aucun prix n'y figure. L'autorité, c'est le fournisseur, et Beaver suit une source vivante.

### Ce qui n'est pas chiffré

Trois cas où aucun coût n'apparaît, et il faut le dire pour éviter que l'absence soit lue comme un bogue :

- **Les modèles locaux.** Ils ne coûtent rien en argent. Leur consommation en jetons est mesurée, pas leur prix.
- **Les connexions par compte.** L'usage est couvert par un abonnement, pas facturé à la requête : afficher un coût n'aurait pas de sens.
- **Quelques modèles récents** dont la tarification n'est pas exprimable par le calcul simple entrée-sortie. Beaver préfère ne rien afficher plutôt qu'un chiffre approximatif.

### Le journal des requêtes

Chaque requête laisse une trace : identifiant, modèle, tour de conversation, tentative, issue, jetons.

Ce journal sert à répondre à « qu'est-ce qui s'est passé ? » sans avoir à reproduire le problème — une requête peut avoir échoué, avoir été reprise, ou avoir été annulée en cours de route, et l'écran d'usage seul ne le dirait pas.

L'usage est aussi **attribué à son origine** : une conversation ordinaire, ou une tâche programmée. Un utilisateur qui trouve sa consommation élevée peut ainsi voir si elle vient de ce qu'il a fait, ou de ce que Beaver a fait tout seul pendant la nuit.

### Changer de clé remet les compteurs distants à zéro

Détail bien pensé, à expliquer en une phrase : quand une clé ou une connexion change, **les informations venant du fournisseur sont effacées**.

La raison : elles décrivaient l'état d'un autre compte. Les conserver reviendrait à afficher le quota d'un compte pour un autre. Le registre local, lui, est conservé — c'est l'historique de ce que Beaver a fait, indépendamment du compte utilisé.

---

## Tableaux

### Les trois sources, résumées

| Source | Ce qu'elle couvre | Ce qu'elle ignore |
|---|---|---|
| Registre local | Tout l'usage fait depuis Beaver | Ce qui passe par un autre outil |
| Données du fournisseur | L'état réel du compte | Le détail par conversation |
| Journal des requêtes | Le détail récent, avec les échecs | L'historique lointain |

### Quand un coût s'affiche

| Situation | Coût affiché |
|---|---|
| Le fournisseur renvoie le coût réel | **Exact** |
| Tarif connu, jetons connus | **Estimé** |
| Modèle local | Aucun — gratuit |
| Connexion par compte | Aucun — couvert par l'abonnement |
| Tarif inconnu ou calcul aberrant | Aucun — plutôt rien qu'un chiffre faux |

### Ce qui est compté dans une requête

| Élément | Facturé |
|---|---|
| Jetons d'entrée nouveaux | Au tarif d'entrée |
| Jetons d'entrée relus depuis le cache | À un tarif réduit |
| Jetons écrits dans le cache | À un tarif propre |
| Jetons de sortie | Au tarif de sortie |
| **Jetons de raisonnement** | **Comptés comme de la sortie** |

---

## Encadrés

> **Les chiffres de Beaver et ceux de votre fournisseur peuvent différer.**
> Beaver ne compte que ce qui est passé par lui. Si le même compte sert ailleurs, les totaux du fournisseur seront plus élevés. Les deux sont justes.

> **Beaver distingue le coût exact du coût estimé.**
> Quand le fournisseur renvoie le montant réel, c'est celui-là qui s'affiche. Sinon, c'est un calcul — proche, mais pas une facture.

> **Les tarifs se mettent à jour tout seuls.**
> Beaver suit un catalogue public rafraîchi automatiquement. Aucun prix n'est figé dans l'application, ni sur ce site.

> **Rien n'est envoyé nulle part.**
> Les mesures de consommation restent sur la machine. Aucune donnée d'usage n'est transmise à Beaver ni à un tiers.

> **Le raisonnement compte dans la facture.**
> Il n'apparaît pas dans la réponse mais consomme des jetons de sortie. Un effort élevé se voit sur la consommation.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Les chiffres de Beaver ne correspondent pas à ma facture » | Le compte sert ailleurs, ou certains coûts sont estimés | Comportement attendu ; la facture du fournisseur fait foi |
| « Aucun coût n'est affiché » | Modèle local, connexion par compte, ou tarif inconnu | Comportement attendu |
| « Ma consommation a augmenté sans que je fasse plus » | Effort de raisonnement élevé, ou tâches programmées | Vérifier l'origine dans le journal |
| « Mes compteurs de quota ont disparu » | Clé ou connexion changée | Comportement voulu : ils décrivaient un autre compte |
| « Le quota affiché est périmé » | Il vient des réponses du fournisseur, il se met à jour à la requête suivante | Forcer un rafraîchissement |
| « Une requête a échoué, est-elle facturée ? » | Selon le moment de l'échec | Le journal indique l'issue et les jetons réellement consommés |

---

## Renvois

- `06-modeles/raisonnement.md` — l'effet du raisonnement sur la consommation
- `06-modeles/providers-api.md` — configurer un fournisseur
- `06-modeles/providers-comptes-web.md` — l'usage couvert par un abonnement
- `01-decouverte/local-vs-cloud.md` — le local ne coûte rien
- `09-automatisation/reveils.md` — la consommation des tâches programmées
- `11-securite/confidentialite-des-donnees.md`
- `12-reference/journaux.md`

---

## Points à confirmer

- **Je n'ai pas identifié l'écran** qui présente ces informations : onglet dédié, panneau dans les réglages, indicateur dans la barre de saisie. Le code expose un instantané par fournisseur et un événement de mise à jour, mais l'emplacement dans l'interface reste à déterminer. **À compléter avant rédaction du site** — c'est la première chose que le lecteur cherchera.
- **La rétention du journal des requêtes** — combien de temps, combien d'entrées — n'a pas été relevée. À compléter, c'est une question fréquente.
- **Quels fournisseurs renvoient un coût exact** et lesquels exposent leurs limites dans leurs réponses n'est pas listé ici. Le code contient des adaptateurs spécifiques par fournisseur. Une table serait utile mais demanderait une lecture complète du module ; à arbitrer selon le niveau de détail voulu.
- **L'affirmation « les jetons de raisonnement sont comptés comme de la sortie »** est la règle générale du domaine ; je ne l'ai pas vérifiée fournisseur par fournisseur dans le code de Beaver. À confirmer avant publication.
- Le cas **Kimi** dispose d'un traitement particulier (un module dédié au solde du compte). À vérifier s'il mérite une mention.
