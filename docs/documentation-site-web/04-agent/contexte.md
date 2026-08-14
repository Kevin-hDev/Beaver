# Contexte

**Emplacement site** — Agent › Contexte
**Répond à** — « Pourquoi la conversation ralentit ou refuse de continuer, et qu'est-ce qui occupe la place ? »
**Sources** — `src-tauri/src/services/agent_local/context_budget.rs` (lignes 4-6, 122-132), `context_capacity_error.rs` (lignes 3-5), `context_usage_buckets.rs` (lignes 11-28, 175-186), `context_budget_prune.rs`, `context_budget_history.rs`, `context_usage_runtime.rs`, `src-tauri/src/commands/context_usage.rs`
**Vérification** — Vérifié dans le code : réserve de réponse, catégories d'usage, code d'erreur

**Ce fichier ne décrit pas la compression** — elle va être revue, son brief est gelé. Se contenter d'y renvoyer.

---

## Plan de page proposé

1. Ce qu'est le contexte
2. Ce qui l'occupe
3. La réserve de réponse
4. L'écran d'usage du contexte
5. Quand le contexte est saturé
6. Réduire l'occupation

---

## Contenu

### 1. Ce qu'est le contexte

Tout ce que le modèle a sous les yeux au moment de répondre : votre message, l'historique, les résultats d'outils, la mémoire, les instructions.

Sa taille est **fixée par le modèle**, pas par Beaver. Un modèle à petite fenêtre saturera vite, quel que soit le réglage.

**À ne pas confondre avec la mémoire** : la mémoire survit à la conversation, le contexte non. Une conversation qui sature son contexte ne perd pas la mémoire.

### 2. Ce qui l'occupe

Beaver répartit l'occupation en **sept catégories** :

| Catégorie | Contenu |
|---|---|
| **Messages** | Votre conversation et les résultats d'outils |
| **Outils système** | Les descriptions des outils disponibles |
| **Connecteurs MCP** | Les descriptions des outils de connecteurs |
| **Skills** | Les skills chargés |
| **Mémoire** | Les notes injectées |
| **Méta-contexte** | Le catalogue des skills disponibles |
| **Prompt système** | Les instructions de départ |

Cette répartition est l'information utile de la page : elle explique qu'un contexte peut être saturé **avant même** qu'on ait écrit un message, si trop de connecteurs sont actifs et trop de skills disponibles.

Détail : pour l'un des fournisseurs distants, le raisonnement du modèle n'est pas compté dans l'occupation. L'estimation peut donc différer d'un fournisseur à l'autre.

### 3. La réserve de réponse

Beaver ne remplit jamais la fenêtre entièrement : une part est **réservée à la réponse**.

| | Valeur |
|---|---|
| Part réservée | **15 %** de la fenêtre |
| Réserve minimale | **4 096 jetons** |
| Réserve maximale | **16 384 jetons** |

Sans cette réserve, le modèle serait coupé au milieu de sa réponse. C'est pour cette raison que la place utilisable est inférieure à la fenêtre annoncée par le fournisseur — et ça mérite d'être dit, sinon l'écart passe pour une erreur.

### 4. L'écran d'usage du contexte

Un écran dédié montre la répartition par catégorie pour la conversation en cours.

C'est l'outil de diagnostic à recommander : plutôt que de deviner ce qui remplit le contexte, on le lit.

### 5. Quand le contexte est saturé

L'erreur porte le code `context_capacity_exceeded`. Elle survient quand ce qui doit être envoyé ne tient pas dans la fenêtre du modèle, réserve de réponse déduite.

Beaver **élague l'historique** avant d'en arriver là. Quand l'élagage ne suffit pas, la compression entre en jeu — voir la page dédiée.

### 6. Réduire l'occupation

Les leviers, par ordre d'efficacité :

- **Changer de modèle** pour un modèle à plus grande fenêtre. C'est le levier le plus direct.
- **Désactiver les connecteurs MCP inutiles** dans la conversation : chaque connecteur actif ajoute la description de tous ses outils.
- **Désactiver les groupes d'outils inutiles** : même raisonnement.
- **Alléger la mémoire injectée** en réduisant son budget.
- **Alléger les instructions permanentes** — jusqu'à 200 Ko peuvent y passer.
- **Cloner la conversation** à un message antérieur pour repartir avec un contexte allégé.

Le dernier levier est le plus spécifique à Beaver et le moins évident : renvoyer vers *Cloner une conversation*.

---

## Tableaux

### Tableau — Réserve de réponse

| | Valeur |
|---|---|
| Part de la fenêtre réservée | 15 % |
| Plancher | 4 096 jetons |
| Plafond | 16 384 jetons |

### Tableau — Contexte, mémoire, instructions permanentes

| | Contexte | Mémoire | Instructions permanentes |
|---|---|---|---|
| Durée de vie | La conversation | Illimitée | Illimitée |
| Écrit par | L'échange en cours | L'agent ou vous | Vous |
| Occupe du contexte | Par nature | Oui, jusqu'à 3 000 jetons | Oui, jusqu'à 200 Ko |
| Limite fixée par | Le modèle | Réglage | Réglage |

---

## Encadrés

**Encadré « Contexte ou mémoire ? »**
> Le contexte est ce que le modèle voit maintenant. La mémoire survit à la conversation. Saturer son contexte ne fait rien perdre de sa mémoire.

**Encadré « La place utilisable est inférieure à la fenêtre annoncée »**
> Beaver réserve environ 15 % de la fenêtre du modèle pour sa réponse, entre 4 096 et 16 384 jetons. Sans cette réserve, la réponse serait coupée en cours de route.

**Encadré « Un contexte peut être plein avant d'avoir écrit »**
> Les descriptions des outils et des connecteurs actifs occupent du contexte en permanence. Avec beaucoup de connecteurs activés, la place disponible se réduit avant le premier message.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Capacité de contexte dépassée » | Le contenu ne tient pas dans la fenêtre du modèle | Changer de modèle, ou cloner à un message antérieur |
| La fenêtre annoncée n'est pas celle disponible | Réserve de réponse de 15 % | Comportement voulu |
| Le contexte se remplit vite sans raison apparente | Connecteurs et outils actifs, mémoire, instructions | Consulter l'écran d'usage du contexte |
| Les premiers messages semblent oubliés | Élagage de l'historique | Cloner en mode résumé pour conserver l'acquis |
| L'usage affiché diffère du décompte du fournisseur | Estimation, et raisonnement non compté chez un fournisseur | Écart normal |

---

## Renvois

- *Agent › Compression* — quand elle sera disponible
- *Agent › Mémoire* — le budget d'injection
- *Agent › Instructions permanentes* — la limite de 200 Ko
- *Interface › Cloner une conversation* — repartir avec un contexte allégé
- *Outils › Vue d'ensemble* — désactiver ce qui ne sert pas
- *Intégrations › Connecteurs MCP*

---

## Points à confirmer

- **Où se trouve l'écran d'usage du contexte** dans l'interface, et ce qu'il affiche exactement. Non relevé — c'est pourtant le cœur pratique de la page.
- **Quel fournisseur ne compte pas le raisonnement.** Identifié dans le code comme un fournisseur distant particulier ; à nommer, ou à formuler de façon générale.
- **La stratégie d'élagage.** Deux fichiers y sont consacrés. Savoir ce qui est écarté en premier — anciens messages, résultats d'outils volumineux ? — permettrait d'expliquer ce que l'utilisateur perd.
- **La fenêtre de contexte est-elle affichée par modèle** dans le sélecteur ? Ce serait l'endroit naturel.
- **Le comportement quand on change de modèle** vers une fenêtre plus petite que l'historique accumulé. Question déjà soulevée dans *Modèles locaux, clés API et comptes web*, toujours ouverte.
- **Le méta-contexte** — le catalogue des skills disponibles semble occuper une catégorie propre. Vérifier ce qu'il contient exactement et comment le réduire.
