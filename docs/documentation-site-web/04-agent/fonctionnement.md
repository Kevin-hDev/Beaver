# Comment l'agent travaille

**Emplacement site** — Agent › Fonctionnement (première page de la section Agent)
**Répond à** — « Que se passe-t-il quand j'envoie un message, et pourquoi ça s'arrête parfois tout seul ? »
**Sources** — `src-tauri/src/services/agent_local/agent_loop.rs` (lignes 46-145), `agent_loop_limits.rs` (ligne 1), `circuit_breaker.rs` (ligne 1), `agent_loop_support.rs`, `agent_loop_thinking_retry.rs`, `agent_loop_completion.rs`, `eager_dispatch.rs`, `tool_executor_parallel_batch.rs` (ligne 14), `agent_chat_queue.rs`
**Vérification** — Vérifié dans le code : la boucle, les limites, les conditions d'arrêt

---

## Plan de page proposé

1. La boucle
2. Les conditions d'arrêt
3. Le garde-fou anti-boucle
4. Les outils en parallèle
5. Le raisonnement
6. Interrompre, mettre en file d'attente
7. Ce qui se passe quand la connexion lâche

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

### 4. Les outils en parallèle

Quand le modèle demande plusieurs outils de lecture d'un coup, Beaver les exécute **en parallèle**, par lots de **dix**.

Sur une exploration qui lit quinze fichiers, le gain est net : deux vagues au lieu de quinze attentes successives.

Les outils qui modifient ne suivent pas ce chemin — ils s'exécutent l'un après l'autre, pour rester prévisibles.

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

---

## Tableaux

### Tableau — Les limites de la boucle

| Limite | Valeur |
|---|---|
| Tours par message | **200** |
| Appels d'outils identiques consécutifs | **6** |
| Outils de lecture en parallèle | **10** par vague |

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

---

## Renvois

- *Agent › Permissions* — ce qui déclenche une confirmation pendant la boucle
- *Agent › Contexte* — la limite qui arrête tout
- *Agent › Sous-agents* — pourquoi la boucle peut se prolonger
- *Modèles › Raisonnement*
- *Agent › Diagnostics et erreurs*

---

## Points à confirmer

- **Le message affiché quand le garde-fou anti-boucle se déclenche.** Le texte exact n'a pas été relevé ; il doit figurer dans la page pour être reconnaissable.
- **Le message affiché à 200 tours.**
- **Le mécanisme de pré-dispatch.** Un fichier lui est consacré : il semble lancer certains outils avant la fin de la réponse du modèle, pour gagner du temps. À élucider — si c'est le cas, c'est un point de performance intéressant à documenter.
- **La décharge du GPU en fin de boucle.** Une fonction s'en occupe pour les modèles locaux. Comprendre son effet : le modèle est-il déchargé de la mémoire vidéo, avec un temps de rechargement au message suivant ?
- **La taille de la file d'attente de messages.**
- **Le comportement en mode Chatbot** : la boucle se réduit-elle à un seul tour ?
