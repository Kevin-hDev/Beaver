# Personnaliser un modèle local

**Emplacement site** — Modèles › Personnalisation
**Répond à** — « Puis-je régler la créativité d'un modèle, lui donner des instructions permanentes, en créer une version à moi ? »
**Sources** — `src/components/ollama/model-parameter-catalog.ts`, `parameter-editor-state.ts`, `modelfile-editor.tsx`, `services/agent_local/ollama_modelfile_create.rs`, `ollama_modelfile_parameters.rs`, `services/agent_local/model_customizations.rs`, `system_prompt_types.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Trois niveaux de personnalisation
2. Les paramètres de génération
3. Le prompt système par modèle
4. Créer son propre modèle
5. Ce qui est vérifié

---

## Contenu

### Trois niveaux de personnalisation

Du plus simple au plus engageant :

| Niveau | Ce que ça change | Réversible |
|---|---|---|
| **Paramètres** | Le comportement de génération | Immédiatement |
| **Prompt système** | Les instructions permanentes du modèle | Immédiatement |
| **Modèle personnalisé** | Un nouveau modèle dérivé d'un autre | En le supprimant |

### Les paramètres de génération

Onze paramètres, regroupés par thème. Le tableau complet est plus bas ; ce qu'il faut expliquer sur le site :

**La longueur de contexte** décide de combien le modèle peut lire d'un coup. Par défaut, Beaver la calcule d'après la mémoire de la machine — c'est le réglage à laisser tel quel dans la quasi-totalité des cas.

**La créativité** — le paramètre de température — est le seul que la plupart des utilisateurs auront envie de toucher. Bas, le modèle est prévisible et répétitif ; haut, il est inventif et instable. Pour du code, on descend ; pour de la rédaction, on peut monter.

**La graine** rend les réponses reproductibles : à graine fixée et question identique, le modèle répond la même chose. Utile pour comparer deux configurations, sans intérêt au quotidien.

**Les pénalités de répétition** empêchent le modèle de tourner en boucle sur les mêmes formulations.

**Les paramètres d'échantillonnage** affinent la façon dont le modèle choisit chaque mot. Ils sont réservés à qui sait ce qu'il fait — les valeurs par défaut conviennent presque toujours.

**Les séquences d'arrêt** indiquent au modèle où s'interrompre.

### Le prompt système par modèle

Un prompt système est un texte d'instructions que le modèle reçoit avant tout message, à chaque conversation.

Beaver permet d'en définir un **par modèle**. Le mécanisme mérite une explication, car il est plus soigné qu'il n'y paraît :

- **Le prompt d'origine du modèle est capturé et conservé** avant toute personnalisation. Il est donc toujours possible de revenir à l'état initial — la modification n'est jamais destructrice.
- Quatre états sont possibles : le prompt par défaut du modèle, celui de Beaver, un prompt personnalisé, ou aucun prompt.

Le détail du système de prompts — modes Chatbot et Agentique, variantes courte et détaillée — est traité dans `04-agent/prompts-systeme.md`. Cette page ne couvre que la partie propre aux modèles locaux.

### Créer son propre modèle

Le niveau le plus avancé : partir d'un modèle existant et en dériver une version qui embarque ses propres réglages et instructions.

Concrètement, on écrit une petite recette qui déclare le modèle de départ, puis ce qu'on veut changer. Beaver la transmet au moteur, qui construit le nouveau modèle.

Ce que ça apporte : un modèle prêt à l'emploi, avec ses réglages intégrés, sélectionnable comme n'importe quel autre — sans avoir à refaire les réglages à chaque fois.

Points de comportement vérifiés :

- La recette est limitée à **2 Mo** — largement suffisant, la plupart tiennent en quelques lignes.
- La création est abandonnée au bout de **10 minutes**.
- **Mettre à jour le modèle de base ne détruit pas le reste.** Beaver ne remplace que la ligne qui désigne le modèle de départ, en conservant toutes les autres instructions.

### Ce qui est vérifié

- **Le nom du modèle est validé** avant toute création.
- **La recette est validée** : ni vide, ni trop volumineuse, ni contenant de caractères interdits.
- Le moteur est appelé **sans passer par un interpréteur de commandes**, et la recette est transmise par un fichier temporaire plutôt que sur la ligne de commande. Un nom ou un contenu malveillant ne peut pas s'exécuter comme une commande.

---

## Tableaux

### Les onze paramètres

| Thème | Paramètre | Type | Défaut |
|---|---|---|---|
| **Contexte** | Longueur de contexte | Entier | **Automatique** |
| **Longueur** | Nombre maximal de jetons produits | Entier | Sans limite |
| **Longueur** | Jetons de prédiction anticipée | Entier | 4 |
| **Créativité** | Température | Décimal | 0,8 |
| **Créativité** | Graine aléatoire | Entier | 0 |
| **Répétition** | Fenêtre de répétition | Entier | 64 |
| **Répétition** | Pénalité de répétition | Décimal | 1,1 |
| **Échantillonnage** | Nombre de candidats | Entier | 40 |
| **Échantillonnage** | Masse de probabilité | Décimal | 0,9 |
| **Échantillonnage** | Probabilité minimale | Décimal | 0,0 |
| **Arrêt** | Séquences d'arrêt | Texte | Aucune |

### Quel paramètre pour quel effet

| Envie | Réglage |
|---|---|
| Réponses plus prévisibles, moins d'inventions | Baisser la température |
| Réponses plus variées | Monter la température |
| Réponses reproductibles à l'identique | Fixer la graine |
| Le modèle se répète | Monter la pénalité de répétition |
| Réponses trop longues | Limiter le nombre de jetons produits |
| Le modèle « oublie » le début de la conversation | Augmenter la longueur de contexte, si la mémoire le permet |

---

## Encadrés

> **Les valeurs par défaut conviennent dans la quasi-totalité des cas.**
> Les paramètres d'échantillonnage en particulier se règlent mal à l'intuition : les modifier sans mesurer dégrade généralement les réponses.

> **Le prompt d'origine est conservé avant toute personnalisation.**
> Il est toujours possible de revenir à l'état initial. La modification n'écrase rien définitivement.

> **Augmenter la longueur de contexte consomme de la mémoire.**
> Un contexte doublé demande davantage de mémoire vidéo. Sur une machine juste, cela peut empêcher le modèle de se charger.

> **Un modèle personnalisé est un vrai modèle.**
> Il occupe sa place, apparaît dans la liste, et se supprime comme les autres.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le modèle ne se charge plus après mon réglage » | Longueur de contexte trop élevée pour la mémoire | Revenir à Automatique |
| « Les réponses sont devenues incohérentes » | Température trop haute, ou échantillonnage modifié | Revenir aux valeurs par défaut |
| « Le modèle répète la même phrase » | Pénalité de répétition trop basse | La remonter |
| « Le modèle s'arrête au milieu d'une phrase » | Nombre de jetons produits trop limité, ou séquence d'arrêt mal choisie | Vérifier les deux |
| « Mon prompt personnalisé n'a aucun effet » | Un prompt de niveau supérieur s'applique | Voir `04-agent/prompts-systeme.md` |
| « La création de mon modèle échoue » | Recette invalide, ou modèle de départ absent | Vérifier que le modèle de base est installé |
| « La création dépasse le délai » | Modèle de base volumineux | Limite à 10 minutes |

---

## Renvois

- `04-agent/prompts-systeme.md` — le système complet de prompts
- `06-modeles/ollama-modeles.md` — installer le modèle de base
- `06-modeles/materiel-et-vram.md` — l'effet du contexte sur la mémoire
- `06-modeles/ollama-runtime.md` — les réglages du moteur
- `10-reglages/modeles.md`

---

## Points à confirmer

- **Les libellés affichés pour les onze paramètres** viennent des fichiers de traduction et n'ont pas été relevés. J'ai donné des noms explicites en français ; **le site doit reprendre ceux de l'application**. À compléter.
- **Où se trouve l'éditeur de paramètres** dans l'interface, et s'il s'applique par modèle ou par conversation, reste à déterminer. Le code suggère par modèle.
- **La description exacte des séquences d'arrêt** et de la prédiction anticipée mérite une relecture technique : ce sont les deux paramètres que j'explique le moins bien, faute d'avoir lu leur usage réel.
- **L'articulation entre le prompt système par modèle et les autres niveaux de prompt** est traitée dans `04-agent/prompts-systeme.md`, écrit avant cette fiche. **Les deux pages doivent être relues ensemble** pour vérifier qu'elles ne se contredisent pas.
- Je n'ai **pas vérifié** si l'interface propose une remise à zéro des paramètres, ni si elle avertit quand un réglage risque d'empêcher le chargement du modèle.
