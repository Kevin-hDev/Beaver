# Trouver et retrouver un modèle

**Emplacement site** — Modèles › Catalogue et favoris
**Répond à** — « Comment je m'y retrouve parmi les centaines de modèles disponibles ? »
**Sources** — `services/favorite_models.rs`, `commands/favorite_models.rs`, `services/llm/litellm_catalog_search.rs`, `services/llm/provider_model_registry.rs`, `services/llm/openai_compat_models.rs`, `services/llm/model_metadata.rs`, `services/llm/tool_capable.rs`, `services/llm/vision.rs`
**Vérification** — Vérifié dans le code pour les mécanismes ; parcours d'interface à confirmer

---

## Plan de page proposé

1. Le problème
2. D'où vient la liste des modèles
3. Ce que Beaver sait d'un modèle
4. Les favoris
5. Choisir un modèle pour une conversation

---

## Contenu

### Le problème

Une fois deux ou trois fournisseurs configurés, plusieurs centaines de modèles deviennent accessibles. Un seul d'entre eux passe par un revendeur qui en propose à lui seul des centaines.

La liste brute est inutilisable. Beaver la rend praticable de deux façons : en enrichissant chaque modèle d'informations utiles, et en permettant d'épingler ceux qu'on utilise.

### D'où vient la liste des modèles

Beaver combine plusieurs sources, ce qui explique pourquoi certains modèles sont mieux documentés que d'autres :

1. **Ce que le fournisseur annonce.** Chaque fournisseur expose la liste des modèles accessibles au compte. C'est la source de vérité sur la disponibilité.
2. **Le registre interne de Beaver**, qui complète les informations manquantes pour les modèles connus.
3. **Un catalogue public**, téléchargé et tenu à jour, qui fournit les caractéristiques et les tarifs.

Conséquence à écrire sur le site : **un modèle très récent peut apparaître avec des informations incomplètes**, le temps que les catalogues le référencent. Il reste utilisable.

### Ce que Beaver sait d'un modèle

Les informations les plus utiles au moment de choisir :

| Information | Pourquoi elle compte |
|---|---|
| **Sait utiliser des outils** | **Décisif.** Sans cette capacité, le modèle ne peut ni lire de fichier ni lancer de commande |
| **Sait lire des images** | Nécessaire pour joindre une capture d'écran |
| **Sait raisonner** | Détermine si le réglage d'effort est disponible |
| **Longueur de contexte** | Combien il peut lire d'un coup |
| **Limite de sortie** | Longueur maximale d'une réponse |
| **Gratuit** | Certains modèles sont proposés sans frais chez certains fournisseurs |

**La capacité à utiliser des outils est le premier critère dans Beaver.** Un modèle qui ne l'a pas tient une conversation, mais reste incapable de la moindre action. C'est la principale cause de déception d'un utilisateur qui choisit un modèle sur sa réputation.

### Les favoris

Un modèle peut être épinglé. Les favoris sont enregistrés localement, dans un fichier propre, et identifient chaque modèle par son fournisseur **et** son nom — le même modèle chez deux fournisseurs compte donc pour deux entrées distinctes.

L'écriture est **atomique** : fichier temporaire puis renommage. Une interruption au mauvais moment ne peut pas laisser une liste de favoris corrompue.

Ajouter un favori déjà présent, ou en retirer un absent, ne provoque pas d'erreur : l'opération est simplement sans effet.

### Choisir un modèle pour une conversation

Le sélecteur se trouve **dans la barre de saisie**, sous le champ de message — pas en haut de la conversation.

Le choix vaut pour la conversation en cours. Changer de modèle en cours de route est possible : la suite de la conversation utilise le nouveau.

---

## Encadrés

> **Vérifiez d'abord que le modèle sait utiliser des outils.**
> C'est ce qui sépare un assistant capable d'agir d'un modèle qui se contente de répondre. Un excellent modèle sans cette capacité ne servira presque à rien dans Beaver.

> **Les favoris sont locaux.**
> Ils vivent dans un fichier de l'application, pas dans un compte. Changer de machine demande de les refaire.

> **Un modèle très récent peut être mal documenté.**
> Ses caractéristiques arrivent avec la mise à jour des catalogues. Il reste utilisable entre-temps.

> **Le même modèle chez deux fournisseurs est deux entrées.**
> Les prix, les limites et la disponibilité peuvent différer d'un fournisseur à l'autre pour un modèle identique.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent ne fait rien, il se contente de répondre » | Le modèle ne sait pas utiliser d'outils | En choisir un qui le sait |
| « Je ne peux pas joindre d'image » | Le modèle ne lit pas les images | En choisir un qui le sait |
| « Le réglage d'effort a disparu » | Le modèle ne raisonne pas | Comportement attendu |
| « Un modèle a disparu de la liste » | Le fournisseur ne le propose plus à ce compte | Vérifier chez le fournisseur |
| « Mes favoris ont disparu » | Fichier local, lié à la machine | Les refaire |
| « Deux fois le même modèle dans la liste » | Il est proposé par deux fournisseurs | Comportement attendu |
| « Les caractéristiques d'un modèle sont vides » | Modèle trop récent pour les catalogues | Il reste utilisable |

---

## Renvois

- `06-modeles/providers-api.md` — configurer un fournisseur pour accéder à ses modèles
- `06-modeles/raisonnement.md` — le réglage d'effort
- `06-modeles/usage-et-couts.md` — d'où viennent les tarifs affichés
- `06-modeles/ollama-modeles.md` — les modèles locaux
- `04-agent/pieces-jointes.md` — joindre une image
- `03-interface/vue-densemble.md` — où se trouve le sélecteur

---

## Points à confirmer

- **L'écran d'exploration des modèles** — filtres, recherche, groupement par famille, page de détail — n'a pas été reconstitué. Le fichier de suivi mentionne un « explorateur LLM » avec familles et détails ; le code correspondant existe mais je n'ai pas relié les composants au parcours. **À compléter avant rédaction du site** : c'est le cœur de la page.
- **Où s'affichent les favoris** — liste séparée, épingle dans le sélecteur, section en tête — reste à déterminer.
- **Le nombre maximal de favoris** n'est pas borné dans le code lu. À vérifier : une liste sans limite est un manquement au principe des collections bornées appliqué partout ailleurs dans le projet. **À signaler à l'équipe.**
- **Les modèles gratuits** sont détectés par leur tarif nul chez certains fournisseurs. Vérifier si l'interface les met en avant, ce qui serait très utile à documenter.
- **La liste des modèles capables d'utiliser des outils** est déterminée par un module dédié qui combine plusieurs sources. Je n'ai pas vérifié son degré de fiabilité sur les modèles récents ni ce qui se passe quand la capacité est inconnue.
