# L'effort de raisonnement

**Emplacement site** — Modèles › Raisonnement
**Répond à** — « À quoi sert le réglage d'effort, et pourquoi ses options changent d'un modèle à l'autre ? »
**Sources** — `services/reasoning.rs`, `services/reasoning_effort.rs`, `services/reasoning_google.rs`, `services/llm/stream_reasoning.rs`, `services/llm/providers/` (`openai.rs`, `groq.rs`, `moonshot.rs`, `xai.rs`, `mistral.rs`), `services/stream_utils.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Ce qu'est le raisonnement
2. Le réglage d'effort
3. Pourquoi les options changent selon le modèle
4. Ce que ça coûte
5. Voir le raisonnement
6. Les cas particuliers

---

## Contenu

### Ce qu'est le raisonnement

Certains modèles réfléchissent avant de répondre : ils produisent un cheminement interne — hypothèses, vérifications, retours en arrière — puis rédigent leur réponse à partir de ce travail.

Ce cheminement est **facturé comme le reste** et occupe de la place dans le contexte, alors qu'il n'apparaît pas dans la réponse finale. D'où l'intérêt d'un réglage : sur une question simple, réfléchir longuement coûte sans rien apporter.

### Le réglage d'effort

Beaver expose une échelle commune, du moins au plus poussé :

| Réglage | Ce qu'il demande |
|---|---|
| **Désactivé** | Répondre sans étape de réflexion |
| **Automatique** | Laisser le modèle décider |
| **Faible** | Réflexion brève |
| **Moyen** | Réflexion équilibrée |
| **Élevé** | Réflexion approfondie |
| **Très élevé** | Réflexion prolongée |
| **Maximum** | Le plus loin que le modèle accepte |
| **Ultra** | Palier supplémentaire, sur les rares modèles qui le proposent |

Le réglage se choisit **par conversation**, à côté du modèle.

### Pourquoi les options changent selon le modèle

C'est la question que se posera tout utilisateur, et la réponse est simple à énoncer : **chaque fournisseur expose son propre réglage, avec ses propres paliers.** Beaver ne présente que ceux que le modèle sélectionné accepte réellement.

Le site doit le dire clairement : ce n'est pas une incohérence de l'interface, c'est le reflet de ce que chaque modèle sait faire. Proposer un palier que le modèle ignore reviendrait à afficher un réglage sans effet.

Trois conséquences concrètes :

- **Certains modèles n'ont aucun réglage** — ils ne raisonnent pas, ou raisonnent toujours de la même façon.
- **Certains ne peuvent pas être désactivés** — leur réflexion fait partie de leur fonctionnement.
- **Le nombre de paliers varie de deux à six** selon le modèle.

Une règle vaut partout : **un réglage non supporté n'est jamais envoyé de force.** Beaver écarte silencieusement une demande incompatible plutôt que de risquer un refus du fournisseur.

### Ce que ça coûte

Trois effets, à énoncer sur le site :

1. **Le temps.** Un effort élevé peut multiplier par plusieurs fois la durée avant la première ligne de réponse.
2. **Le prix**, sur un modèle facturé : le raisonnement consomme des jetons, souvent au tarif de sortie.
3. **Le contexte.** Le cheminement occupe de la place, donc réduit d'autant ce qui reste pour la conversation.

**Le conseil pratique** : garder un effort faible ou automatique par défaut, et le monter pour les tâches qui le justifient réellement — un bogue difficile, une conception à trancher, une analyse. Sur une reformulation ou une question factuelle, l'effort élevé coûte sans rien apporter.

### Voir le raisonnement

Quand un modèle expose son cheminement, Beaver l'affiche **séparément de la réponse**, dans une zone repliable.

Deux points à préciser :

- Certains fournisseurs livrent un **résumé** du raisonnement plutôt que son texte intégral. C'est leur choix, pas une troncature de Beaver.
- Certains modèles locaux encadrent leur réflexion par des balises dans le texte. Beaver **les repère et les extrait** pour que le raisonnement n'apparaisse pas mélangé à la réponse.

### Les cas particuliers

À mentionner sur le site sans entrer dans le détail par modèle, qui se périmerait aussitôt :

- **Modèles locaux** — la plupart n'offrent que « désactivé » ou « automatique ». Quelques familles récentes acceptent trois paliers.
- **Modèles à réflexion imposée** — le réglage est absent ou limité à « automatique ».
- **Google** — l'effort se traduit par un budget de réflexion chiffré, différent selon la génération du modèle.
- **Certains modèles récents** ajoutent des paliers au-delà du maximum habituel.

---

## Tableaux

### Ordre de grandeur des paliers proposés

| Famille de modèles | Étendue du réglage |
|---|---|
| Modèles locaux courants | Désactivé / Automatique |
| Modèles locaux à effort réglable | Faible à Élevé |
| Modèles généralistes distants | Désactivé, puis Faible à Très élevé |
| Modèles distants les plus récents | Jusqu'à Maximum, parfois Ultra |
| Modèles à réflexion imposée | Automatique seulement |
| Modèles sans raisonnement | Aucun réglage |

> **Ce tableau donne des ordres de grandeur, pas une liste par modèle.** Une liste nominative serait périmée à la première sortie de modèle. L'interface affiche toujours les options réellement disponibles pour le modèle sélectionné : c'est elle qui fait foi.

### Quel effort pour quelle tâche

| Tâche | Effort conseillé |
|---|---|
| Reformuler, traduire, résumer | Désactivé ou Faible |
| Écrire du code simple, corriger une erreur évidente | Faible |
| Conversation ordinaire | Automatique |
| Déboguer un problème non reproductible | Élevé |
| Concevoir une architecture, arbitrer entre options | Élevé à Maximum |
| Analyser un système entier | Maximum |

---

## Encadrés

> **Les options changent selon le modèle, et c'est normal.**
> Beaver n'affiche que les paliers que le modèle sélectionné accepte vraiment. Un réglage absent signifie que ce modèle ne le propose pas.

> **Le raisonnement est facturé et occupe le contexte.**
> Il n'apparaît pas dans la réponse mais consomme des jetons. Un effort élevé sur chaque message coûte inutilement.

> **Un réglage incompatible n'est jamais forcé.**
> Beaver l'écarte plutôt que d'envoyer une demande que le fournisseur refuserait.

> **Le raisonnement affiché peut être un résumé.**
> Plusieurs fournisseurs ne livrent pas le cheminement intégral. Ce n'est pas Beaver qui le raccourcit.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le réglage d'effort a disparu » | Le modèle sélectionné ne raisonne pas | Comportement attendu |
| « Je ne peux pas désactiver la réflexion » | Modèle à réflexion imposée | Changer de modèle si le délai gêne |
| « Les réponses sont devenues très lentes » | Effort monté à Élevé ou plus | Redescendre pour les tâches ordinaires |
| « Ma facture a augmenté sans plus de messages » | Le raisonnement consomme des jetons | Baisser l'effort par défaut |
| « Je ne vois pas le raisonnement » | Le fournisseur ne l'expose pas, ou la zone est repliée | Déplier la zone dédiée |
| « Le raisonnement apparaît mélangé à la réponse » | Balises non reconnues sur un modèle local | À signaler — c'est un défaut |
| « Mon contexte se remplit vite » | Le raisonnement occupe de la place | Baisser l'effort, ou voir `04-agent/contexte.md` |

---

## Renvois

- `06-modeles/catalogue-et-favoris.md` — voir les capacités d'un modèle
- `06-modeles/usage-et-couts.md` — l'effet du raisonnement sur la consommation
- `04-agent/contexte.md` — la place occupée
- `01-decouverte/local-vs-cloud.md`
- `10-reglages/modeles.md`

---

## Points à confirmer

- **La correspondance exacte entre les paliers de Beaver et ceux de chaque fournisseur** existe dans le code, modèle par modèle. Je ne la reproduis volontairement pas : elle change à chaque sortie de modèle et serait fausse en quelques semaines. **Décision à valider par l'équipe** — si le site veut une table nominative, il faut prévoir qui la maintient.
- **Les libellés affichés dans l'interface** pour ces paliers n'ont pas été relevés dans les fichiers de traduction. Le site doit reprendre les mots de l'application, pas les miens. À compléter.
- **Où se règle l'effort** — sélecteur dédié, menu du modèle, réglage global avec exception par conversation — n'est pas déterminé. J'ai écrit « par conversation, à côté du modèle » d'après la structure du code ; **à vérifier avant publication.**
- Le **palier Ultra** n'existe que pour de très rares modèles. Vérifier qu'il est réellement atteignable depuis l'interface, et pas seulement présent dans le code.
- Affichage à vérifier lors de la passe d'interface : présentation de la zone de raisonnement, repliée ou dépliée par défaut, et comportement pendant la génération.
