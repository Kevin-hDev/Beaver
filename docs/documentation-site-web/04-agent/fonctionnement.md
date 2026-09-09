# Comment l'agent travaille

**Emplacement site** — Agent › Fonctionnement (première page de la section Agent)
**Répond à** — « Que se passe-t-il quand j'envoie un message, et pourquoi ça s'arrête parfois tout seul ? »
**Sources** — `src-tauri/src/services/agent_local/agent_loop.rs` (lignes 69, 92-100), `agent_loop_limits.rs` (ligne 1), `circuit_breaker.rs` (lignes 1, 21-25), `agent_loop_support.rs` (lignes 23-36), `agent_loop_thinking_retry.rs`, `agent_loop_completion.rs`, `eager_dispatch.rs` (lignes 11, 71-78), `tool_executor_parallel_batch.rs` (ligne 17), `ollama_wire.rs`, `agent_chat_queue.rs`
**Vérification** — Vérifié dans le code, revérifié le 9 septembre 2026 : la boucle, les limites, les conditions d'arrêt, le pré-dispatch et la libération de la mémoire vidéo

---

## Plan de page proposé

1. La boucle
2. Les conditions d'arrêt
3. Le garde-fou anti-boucle
4. Les outils en parallèle
5. Le raisonnement
6. Interrompre, mettre en file d'attente
7. Ce qui se passe quand la connexion lâche
8. Le modèle local et la mémoire vidéo

---

## Contenu

### 1. La boucle

C'est le mécanisme central du produit, et il vaut la peine d'être décrit simplement.

À chaque tour :

1. Beaver envoie au modèle la conversation, les instructions et la liste des outils disponibles.
2. Le modèle répond : du texte, des appels d'outils, ou les deux.
3. Beaver exécute les outils demandés — en demandant confirmation si nécessaire.
4. Les résultats sont ajoutés à la conversation.
5. On recommence.

**La boucle s'arrête quand le modèle ne demande plus d'outil** : il a fini, il répond.

C'est ce cycle qui distingue un agent d'un chat. Un chat s'arrête à l'étape 2.

**La boucle est la même en mode Chatbot.** Elle n'est pas ramenée à un seul tour : c'est le catalogue d'outils qui est réduit à la recherche web et à la lecture d'une page. Un tour de Chatbot peut donc enchaîner une recherche, la lecture d'une page, puis la réponse. Voir *Agent › Modes de permission*.

### 2. Les conditions d'arrêt

Cinq façons pour un tour de conversation de se terminer :

| Cause | Ce qui se passe |
|---|---|
| **Le modèle a fini** | Plus d'appel d'outil : c'est la sortie normale |
| **Vous avez annulé** | L'annulation est vérifiée à chaque tour et avant chaque outil |
| **200 tours atteints** | Limite dure ; un avertissement précède le dernier tour |
| **Boucle détectée** | Voir section 3 |
| **Erreur bloquante** | Contexte saturé, modèle indisponible, échec réseau non récupérable |

La limite de **200 tours** est généreuse : elle sert de filet contre une boucle infinie, pas de plafond de travail. Un tour supplémentaire est signalé au modèle avant d'y arriver, ce qui lui permet de conclure proprement plutôt que d'être coupé net.

Détail à mentionner : si des sous-agents travaillent encore, la boucle peut se poursuivre au-delà du moment où le modèle n'appelle plus d'outil — le temps qu'ils rendent leur résultat.

### 3. Le garde-fou anti-boucle

**Six appels d'outils identiques consécutifs** arrêtent la conversation.

Un modèle qui tourne en rond — même outil, mêmes arguments, encore et encore — ne progressera pas au septième essai. Mieux vaut s'arrêter et vous rendre la main que consommer des jetons.

C'est un comportement à documenter, parce qu'il surprend : la conversation s'interrompt avec un message alors que rien n'a échoué.

**Le message affiché est écrit en dur dans le code**, sous la forme « Circuit breaker : N appels identiques consécutifs détectés. Boucle probable, arrêt. » Deux conséquences pour la page :

- **ne pas le citer tel quel.** Le terme *circuit breaker* n'est compréhensible pour personne sans explication : décrire le comportement, et donner le message seulement comme repère de reconnaissance.
- **ce message n'est pas traduit** : ce n'est pas une clé de traduction, un utilisateur en espagnol ou en japonais lira ce texte franco-anglais. **Manquement à la règle i18n du projet, à remonter à l'équipe avant publication.**

### 4. Les outils en parallèle

Quand le modèle demande plusieurs outils de lecture d'un coup, Beaver les exécute **en parallèle**, par lots de **dix**.

Sur une exploration qui lit quinze fichiers, le gain est net : deux vagues au lieu de quinze attentes successives.

Les outils qui modifient ne suivent pas ce chemin — ils s'exécutent l'un après l'autre, pour rester prévisibles.

**Mieux : les outils de lecture démarrent avant que le modèle ait fini d'écrire.** Dès qu'un appel d'outil en lecture seule apparaît dans le flux de réponse, Beaver le lance sans attendre la fin du message. Le même plafond de **dix** appels simultanés s'applique. Un outil qui demande une confirmation n'entre jamais dans ce mécanisme.

Le gain se voit sur une exploration : la lecture des fichiers a commencé pendant que le modèle rédigeait encore la suite de sa réponse. C'est un point de performance qui mérite d'être écrit, parce qu'il explique une réactivité que le simple traitement par lots ne suffit pas à justifier.

### 5. Le raisonnement

Certains modèles réfléchissent avant de répondre. Beaver affiche ce raisonnement, séparé de la réponse.

Un mécanisme de reprise existe quand un modèle local produit un raisonnement mal formé. À défaut, l'affichage serait pollué par des balises internes.

Renvoyer vers *Modèles › Raisonnement* pour le réglage de l'intensité.

### 6. Interrompre, mettre en file d'attente

- **Interrompre** est possible à tout moment. L'annulation est vérifiée à chaque tour et avant chaque outil : elle prend effet vite, sans attendre la fin du tour.
- **Mettre un message en file d'attente** pendant que l'agent travaille : il sera traité une fois le tour terminé, sans qu'il faille attendre devant l'écran.

### 7. Ce qui se passe quand la connexion lâche

Si le flux de réponse est coupé en cours de route, Beaver **conserve ce qui a été reçu** et poursuit plutôt que de perdre le tour.

Les échecs de flux sont enregistrés dans la conversation — utile pour diagnostiquer un fournisseur instable. Ils ne sont pas repris dans un clone.

### 8. Le modèle local et la mémoire vidéo

Question fréquente sur les modèles locaux : le modèle reste-t-il chargé dans la carte graphique entre deux messages ?

**Oui, pendant un temps réglable — et non, ce n'est pas lié à la fin de la boucle.** Aucun déchargement n'est déclenché quand l'agent termine son travail. Beaver joint à **chaque requête** envoyée à Ollama une durée de maintien en mémoire, lue dans les réglages avancés :

| Valeur du réglage | Effet |
|---|---|
| **5 minutes** (valeur par défaut) | Le modèle est libéré après cinq minutes sans requête |
| Une autre durée | Le modèle est libéré après cette durée sans requête |
| **Toujours** | Traduit en `-1m` : Ollama ne décharge jamais le modèle |

Le compromis est à expliquer, parce qu'il se ressent : un modèle libéré rend sa mémoire vidéo aux autres applications, mais le message suivant paie le temps de rechargement. Un modèle maintenu répond tout de suite et occupe la carte en permanence.

Ce réglage ne concerne que les modèles locaux servis par Ollama. Les modèles cloud ne passent pas par ce mécanisme.

---

## Tableaux

### Tableau — Les limites de la boucle

| Limite | Valeur |
|---|---|
| Tours par message | **200** |
| Appels d'outils identiques consécutifs | **6** |
| Outils de lecture en parallèle | **10** par vague |
| Outils de lecture démarrés pendant le flux de réponse | **10** au maximum |
| Maintien du modèle local en mémoire vidéo | **5 minutes** par défaut |

---

## Encadrés

**Encadré « Pourquoi ça s'arrête parfois tout seul »**
> Si le modèle demande six fois de suite le même outil avec les mêmes arguments, Beaver interrompt la conversation. Un modèle qui tourne en rond ne progressera pas au septième essai.

**Encadré « Vous n'êtes pas obligé d'attendre »**
> Un message envoyé pendant que l'agent travaille est mis en file d'attente et traité au tour suivant.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| La conversation s'arrête sans erreur apparente | Six appels d'outils identiques détectés | Reformuler, ou changer de modèle |
| L'agent s'arrête après un long travail | 200 tours atteints | Découper la tâche, ou relancer |
| L'annulation ne semble pas immédiate | Elle prend effet entre deux étapes | Elle est vérifiée à chaque tour et avant chaque outil |
| L'agent continue après avoir répondu | Des sous-agents travaillent encore | Attendre leur résultat |
| Des balises de raisonnement apparaissent dans la réponse | Modèle local produisant un format inattendu | Un mécanisme de reprise existe ; essayer un autre modèle |
| Une réponse s'arrête au milieu | Flux coupé | Ce qui a été reçu est conservé ; relancer |
| Le premier message après une pause est lent avec un modèle local | Le modèle a été libéré de la mémoire vidéo après le délai de maintien | Augmenter ce délai dans les réglages avancés, ou le mettre sur « toujours » |
| Un modèle local occupe la carte graphique en permanence | Délai de maintien réglé sur « toujours » | Choisir une durée finie |

---

## Renvois

- *Agent › Permissions* — ce qui déclenche une confirmation pendant la boucle
- *Agent › Contexte* — la limite qui arrête tout
- *Agent › Sous-agents* — pourquoi la boucle peut se prolonger
- *Modèles › Raisonnement*
- *Agent › Diagnostics et erreurs*

---

## Points à confirmer

- **Le message affiché à 200 tours.** Aucun message dédié n'a été trouvé dans le code : la boucle est un simple parcours de 200 tours, et l'approche de la fin est signalée au modèle avant le dernier tour. À vérifier à l'écran avec le reste de la passe d'interface.
- **La taille de la file d'attente de messages.**
- **Le libellé exact du réglage de maintien en mémoire vidéo** dans l'écran des réglages avancés, et les valeurs proposées à l'utilisateur. Le code accepte une durée libre et la valeur « toujours » ; l'écran n'a pas été ouvert.
- **Le message du garde-fou anti-boucle n'est pas traduit.** Ce n'est pas une incertitude mais un défaut : à corriger côté produit, ou à assumer explicitement, avant que le site ne le cite.
