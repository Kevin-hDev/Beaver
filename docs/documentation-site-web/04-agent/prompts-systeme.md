# Prompts système

**Emplacement site** — Extensions › Réécrire le prompt système dans le mockup ; à placer plutôt dans Agent › Prompts système
**Répond à** — « Quelles instructions Beaver donne-t-il au modèle avant mon premier message, et puis-je les changer ? »
**Sources** — `src-tauri/src/services/agent_local/system_prompt_types.rs`, `system_prompt_resolver.rs`, `system_prompt_store.rs`, `chat_prompts.rs` (lignes 57-101), `model_size.rs`, `ollama_native_prompts.rs`, `ollama_native_prompt_store.rs`, `src-tauri/src/commands/system_prompts.rs`, `src/components/system-prompts/`
**Vérification** — Vérifié dans le code : modes, niveaux, cibles, états et provenance des prompts

---

## Plan de page proposé

1. Ce qu'est un prompt système
2. Deux modes, deux niveaux
3. Le niveau est choisi automatiquement
4. Deux portées : global ou par modèle
5. Les quatre états possibles
6. Les prompts natifs Ollama
7. Modifier, désactiver, restaurer

---

## Contenu

### 1. Ce qu'est un prompt système

Les instructions données au modèle **avant votre premier message**. Elles définissent son rôle, la façon d'employer les outils, le format attendu de ses réponses.

Beaver en fournit par défaut. Ils sont **consultables, modifiables, remplaçables et désactivables** — ce qui n'est pas si courant, et mérite d'être présenté comme un choix de transparence.

À distinguer de vos propres instructions : le prompt système vient de Beaver, `AGENTS.md` vient de vous. Renvoyer vers *Instructions permanentes*.

### 2. Deux modes, deux niveaux

**Deux modes**, selon la façon de travailler :

| Mode | Usage |
|---|---|
| **Chatbot** | Conversation sans outils |
| **Agentique** | Travail avec outils |

**Deux niveaux de détail** pour chaque mode :

| Niveau | Usage |
|---|---|
| **Compact** | Version resserrée |
| **Détaillé** | Version complète |

Soit **quatre prompts** par portée. Chacun se règle indépendamment.

### 3. Le niveau est choisi automatiquement

Point important, et non évident : **le niveau dépend du modèle employé**, pas d'un réglage manuel.

Le choix se fait à partir de la taille du modèle. Un petit modèle reçoit la version compacte, un grand la version détaillée. La raison est solide : un prompt long occupe une part importante du contexte d'un petit modèle, et le noie plutôt qu'il ne le guide.

Voir *Points à confirmer* pour le seuil exact.

### 4. Deux portées : global ou par modèle

| Portée | Effet |
|---|---|
| **Globale** | S'applique à tous les modèles |
| **Par modèle Ollama** | Ne s'applique qu'à ce modèle |

**Le réglage par modèle prime sur le réglage global.**

Cette portée par modèle n'existe que pour les modèles Ollama — ce sont les seuls dont Beaver gère le cycle de vie complet.

### 5. Les quatre états possibles

Ce qu'affiche l'application pour chaque prompt :

| État | Signification |
|---|---|
| **Par défaut** | Aucun réglage, le prompt fourni est employé |
| **Beaver** | Le prompt de Beaver est explicitement retenu |
| **Personnalisé** | Vous avez écrit le vôtre |
| **Désactivé** | Aucun prompt système n'est envoyé |

Et **trois provenances** possibles pour le texte réellement employé : celui de Beaver, celui d'Ollama, ou le vôtre.

L'état **Désactivé** mérite un avertissement : sans prompt système, le modèle ne sait pas comment employer les outils ni quel format adopter. C'est utilisable pour un usage précis, pas comme réglage courant.

Détail relevé dans le code : **un prompt personnalisé vide est traité comme désactivé**, pas comme un retour au défaut. C'est volontaire — vider le champ est un moyen explicite de couper le prompt.

### 6. Les prompts natifs Ollama

Un modèle Ollama peut embarquer son propre prompt système, défini par son auteur.

- Beaver **préserve ce prompt natif** avant toute personnalisation, dans un stockage dédié.
- L'application indique si un prompt natif est **disponible** pour le modèle en cours.
- On peut donc choisir explicitement entre le comportement voulu par l'auteur du modèle et celui de Beaver.

C'est un point de respect du travail des autres qui vaut d'être mentionné : personnaliser un modèle ne détruit pas ce qu'il portait.

### 7. Modifier, désactiver, restaurer

Trois opérations, dans **Réglages › Prompt système** :

- **Enregistrer** un contenu personnalisé ;
- **Désactiver** le prompt ;
- **Restaurer** le prompt d'origine.

Deux garde-fous relevés dans le code :

- un **avertissement avant de remplacer** un prompt déjà personnalisé ;
- une **copie dans le presse-papier en un clic** avant que le texte soit perdu.

Ces deux détails valent d'être documentés : ils signalent que le produit anticipe la perte de travail.

---

## Tableaux

### Tableau — Récapitulatif des combinaisons

| Portée | Mode | Niveau | Réglable |
|---|---|---|---|
| Globale | Chatbot | Compact | Oui |
| Globale | Chatbot | Détaillé | Oui |
| Globale | Agentique | Compact | Oui |
| Globale | Agentique | Détaillé | Oui |
| Par modèle Ollama | Les quatre mêmes combinaisons | | Oui, et prime sur le global |

### Tableau — Prompt système et instructions permanentes

| | Prompt système | AGENTS.md et personnalité |
|---|---|---|
| Écrit par | Beaver, ou vous en remplacement | Vous |
| Contenu | Rôle, usage des outils, format | Conventions, commandes, interdits |
| Désactivable | Oui | Il suffit de ne rien écrire |
| Portée par modèle | Oui, pour Ollama | Non |

---

## Encadrés

**Encadré « Le niveau est automatique »**
> Beaver choisit la version compacte ou détaillée selon la taille du modèle. Un petit modèle reçoit un prompt plus court, pour ne pas saturer son contexte.

**Encadré « Désactiver le prompt système »** — avertissement.
> Sans prompt système, le modèle ne sait ni comment employer les outils, ni quel format adopter. Réservez ce réglage à un usage précis.

**Encadré « Un champ vide désactive »**
> Enregistrer un prompt personnalisé vide ne restaure pas le prompt d'origine : cela désactive le prompt système. Utilisez « Restaurer » pour revenir au défaut.

**Encadré « Les prompts natifs sont préservés »**
> Si un modèle Ollama embarque son propre prompt système, Beaver le conserve avant toute personnalisation. Vous pouvez y revenir.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Un réglage global semble ignoré | Un réglage par modèle prime | Vérifier l'onglet du modèle concerné |
| Le prompt affiché n'est pas celui attendu | Le niveau dépend de la taille du modèle | Vérifier les deux niveaux |
| Le modèle n'emploie plus ses outils | Prompt système désactivé | Restaurer, ou réécrire un prompt qui décrit les outils |
| Un prompt personnalisé a disparu | Champ vidé, donc désactivé | Le récupérer depuis le presse-papier si la copie a été faite |
| Le comportement du modèle change après personnalisation | Passage du prompt natif Ollama à celui de Beaver | Choisir explicitement la provenance |

---

## Renvois

- *Agent › Instructions permanentes* — vos propres consignes
- *Modèles › Ollama — personnalisation* — les modelfiles et paramètres
- *Réglages › Agent*
- *Extensions › Réécrire le prompt système* — quand cette section sera disponible

---

## Points à confirmer

- **Le seuil de bascule entre Compact et Détaillé.** Un fichier est dédié au calcul de la taille du modèle ; la valeur exacte n'a pas été relevée. Utile : elle explique pourquoi tel modèle reçoit tel prompt.
- **Le contenu des prompts par défaut.** Ils sont consultables dans l'application. Décider si le site en publie des extraits — l'argument de transparence gagnerait à montrer au moins la structure.
- **Le niveau est-il réellement non réglable ?** Les commandes exposent le niveau en paramètre, ce qui laisse penser qu'on consulte les deux mais que le choix reste automatique. À confirmer.
- **La portée par modèle existe-t-elle pour les fournisseurs distants**, ou seulement pour Ollama ? Le code ne prévoit que `Global` et `Ollama`, mais confirmer côté interface.
- **Ce que devient un prompt personnalisé** quand le modèle Ollama concerné est supprimé.
- **L'articulation avec les extensions.** Une extension peut réécrire le prompt système. La précédence entre réglage utilisateur, réglage par modèle et extension est à établir — sujet gelé avec les extensions, à traiter ensemble.
