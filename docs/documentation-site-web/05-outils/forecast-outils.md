# Forecast depuis une conversation — les sept outils

**Emplacement site** — Outils › Forecast
**Répond à** — « Puis-je demander une prévision à l'agent en langage naturel, sans passer par l'espace Forecast ? »
**Sources** — `tool_definitions_forecast.rs`, `tool_definitions_forecast_run.rs`, `tool_definitions_forecast_audit.rs`, `tool_definitions_forecast_data.rs`, `tool_definitions_forecast_evaluation.rs`, `forecast/limits.rs`, `forecast/selection_policy.rs`, `permission_gate.rs`, `tool_catalog.rs`
**Vérification** — Vérifié dans le code, côté définition des outils. Le moteur de prévision lui-même n'a pas été lu — voir la section 8.

> Cette page décrit **l'usage de Forecast par l'agent, depuis une conversation**. La section 8 couvre l'espace Forecast complet : import de données, modèles, évaluation, scénarios, rapports, exports.

---

## Plan de page proposé

1. Deux façons d'utiliser Forecast
2. Les sept outils
3. L'enchaînement obligatoire
4. Ce qui est audité avant toute prévision
5. Le choix du modèle
6. Valider une prévision
7. Enrichir une analyse
8. Ce que l'agent n'a pas le droit de faire

---

## Contenu

### Deux façons d'utiliser Forecast

Forecast est d'abord un **espace dédié** dans l'application, avec ses écrans, ses graphiques et ses exports.

Ces sept outils ouvrent une seconde porte : **piloter Forecast en langage naturel depuis une conversation**. « Prévois-moi les ventes des trois prochains mois à partir de ce fichier » devient une suite d'appels d'outils, et le résultat est une analyse enregistrée, consultable ensuite dans l'espace Forecast.

Ils forment le groupe **Forecast**, optionnel et **éteint par défaut**.

### Les sept outils

| Outil | Rôle | Approbation |
|---|---|---|
| `forecast_data_audit` | Vérifie les données avant toute prévision | **Oui** |
| `forecast_models` | Consulte les modèles utilisables et la politique de sélection | Non |
| `forecast_run` | Lance la prévision | **Oui** |
| `forecast_read` | Relit une analyse enregistrée | Non |
| `forecast_backtest` | Valide un modèle sur l'historique | **Oui** |
| `forecast_compare_models` | Lit le classement des modèles évalués | Non |
| `forecast_analyze` | Ajoute des notes et des scénarios | Non |

Les trois outils approuvés sont les trois qui **consomment des ressources réelles** : ils font tourner des modèles, parfois pendant plusieurs minutes, parfois dans le nuage donc à un coût.

### L'enchaînement obligatoire

C'est le point structurant de la page, et il est inhabituel : **l'ordre des appels est imposé**.

```
Auditer les données  →  Choisir un modèle  →  Prévoir  →  Lire les résultats
                                              ↓
                                     Valider sur l'historique  →  Comparer
```

Aucune prévision ne peut être lancée sans un audit préalable. L'audit produit un **identifiant de profil de données** qui sert ensuite de laissez-passer : c'est cet identifiant qui est transmis à la prévision, **pas les données elles-mêmes**.

Ce mécanisme a trois conséquences pratiques :

1. les données ne transitent qu'une fois, quel que soit le nombre de prévisions ;
2. **le niveau de confiance demandé est lié au profil** — il ne peut plus être modifié en cours de route ;
3. les problèmes de données sont détectés **avant** qu'un modèle ne tourne, pas après.

### Ce qui est audité avant toute prévision

L'audit vérifie une douzaine de points, et c'est ce qui distingue Forecast d'un simple appel à un modèle :

- validité des dates et **ordre chronologique** ;
- **doublons** et **périodes manquantes** ;
- cohérence de la **fréquence** annoncée ;
- **longueur de l'historique** — assez de données pour prévoir ;
- **nombre de séries** ;
- présence de **lignes futures** ;
- validité des **valeurs numériques** ;
- **valeurs aberrantes** ;
- **budget de prédiction** — le volume demandé reste-t-il tenable.

Un audit qui échoue empêche la prévision. L'agent ne peut pas passer outre.

### Le choix du modèle

Deux politiques, réglées par l'utilisateur dans l'espace Forecast, et **la définition même des outils change selon la politique active** — l'agent ne voit pas les mêmes paramètres dans les deux cas.

**En mode manuel** — l'utilisateur a désigné un modèle. L'agent doit vérifier que ce modèle **supporte le niveau de confiance demandé**, et s'y tenir. Si aucun modèle n'est désigné, l'agent a pour consigne de **demander à l'utilisateur de choisir** avant toute prévision.

**En mode automatique** — l'agent consulte la liste des modèles utilisables pour ce profil de données, en choisit **un seul**, et transmet son choix avec :

- un **identifiant de sélection** à durée de vie courte, délivré au moment de la consultation ;
- l'origine du choix : décision de l'agent, ou demande explicite de l'utilisateur ;
- des **motifs structurés**, choisis dans une liste fermée : meilleur résultat en validation, dépasse les références, précision demandée, rapidité demandée, exécution locale requise, nuage autorisé, demandé par l'utilisateur, adapté au matériel.

Ces motifs ne sont pas décoratifs : ils rendent le choix du modèle **traçable**. L'utilisateur peut savoir pourquoi tel modèle a été retenu, sans avoir à croire une justification rédigée librement.

Une contrainte revient partout dans les consignes : **ne jamais arrondir le niveau de confiance pour qu'il rentre dans les capacités d'un modèle**. Si un modèle ne sait pas produire l'intervalle demandé, il n'est pas candidat — on ne dégrade pas la demande de l'utilisateur pour l'y faire entrer.

### Valider une prévision

Une prévision seule ne dit pas si elle est bonne. La validation par **fenêtres glissantes** répond à cette question : on rejoue le modèle sur des périodes passées dont on connaît le résultat.

- Le modèle est comparé à **quatre méthodes de référence** : la valeur précédente, la valeur de la même période de la saison précédente, une tendance simple, et un lissage exponentiel.
- Jusqu'à **5 modèles** évalués sur les **mêmes fenêtres**, **3 fenêtres** par défaut, **5** au maximum.
- Les résultats sont enregistrés sur l'analyse.

Les consignes données à l'agent sont particulièrement strictes ici, et le site gagne à les reprendre :

- **ne jamais présenter une validation partielle comme complète** — l'agent doit inspecter les échecs de modèles ;
- **ne jamais qualifier un modèle de « meilleur » s'il ne bat pas les méthodes de référence.**

Ce second point est le cœur de l'honnêteté statistique de Forecast : un modèle sophistiqué qui ne fait pas mieux que « demain sera comme aujourd'hui » n'est pas un bon modèle, quelle que soit sa réputation.

La comparaison renvoie les mesures d'exactitude, la **couverture réelle des intervalles**, la durée et la mémoire observée — sans jamais renvoyer les données brutes.

### Enrichir une analyse

Sur une analyse enregistrée, l'agent peut :

- **annoter** — ajouter une note datée ;
- **créer un scénario** de deux formes :
  - un **ajustement en pourcentage** — « et si tout augmentait de 15 % ? », qui dérive une courbe sans relancer le modèle ;
  - un **ajustement contextuel** — modifier une variable explicative connue pour l'avenir et **relancer réellement le modèle** ;
- **modifier ou supprimer** un scénario ;
- **construire un ensemble** à partir de plusieurs modèles validés — de 2 à 4 modèles, pondérés par leur performance.

Détail remarquable : un ensemble est **explicitement marqué comme non validé indépendamment**. Beaver refuse de laisser croire qu'une combinaison de modèles validés est elle-même validée.

### Ce que l'agent n'a pas le droit de faire

Plusieurs analyses avancées — décomposition de la série, anomalies résiduelles, importance des variables, dérive — sont **calculées automatiquement** par Forecast. La consigne donnée à l'agent est sans ambiguïté : **les lire, jamais les fabriquer**.

De même, quand une analyse est indisponible ou peu fiable, l'agent doit le dire tel quel — et non la remplacer par un calcul approximatif de son cru.

C'est une conception à mettre en avant sur le site : **Forecast est construit pour empêcher le modèle d'inventer des statistiques**. Les chiffres viennent du moteur, pas du modèle de langage.

---

## Tableaux

### Les limites

| Limite | Valeur |
|---|---|
| Lignes de données en entrée | **5 000** |
| Colonnes | **256** |
| Séries simultanées | **256** |
| Variables explicatives | **64** |
| Horizon de prévision | **5 000** périodes |
| Prédictions au total | **100 000** |
| Données transmises directement | **5 Mo** |
| Fichier tableur en source | **50 Mo** |
| Profils de données conservés | **20** |
| Analyses enregistrées | **500** |
| Prédictions par page de lecture | **100** par défaut, **200** au maximum |
| Modèles par validation | **5** |
| Fenêtres de validation | **3** par défaut, **5** au maximum |
| Validations simultanées | **1** |
| Modèles dans un ensemble | **2 à 4** |
| Modèles candidats en mode automatique | **5** |

### Les outils et leur moment

| Étape | Outil | Obligatoire |
|---|---|---|
| 1. Vérifier les données | `forecast_data_audit` | **Oui** |
| 2. Choisir un modèle | `forecast_models` | **Oui** en mode automatique |
| 3. Prévoir | `forecast_run` | — |
| 4. Lire | `forecast_read` | — |
| 5. Valider | `forecast_backtest` | Recommandé |
| 6. Comparer | `forecast_compare_models` | Après validation |
| 7. Annoter, simuler | `forecast_analyze` | — |

---

## Encadrés

> **Aucune prévision sans audit.**
> Les données sont vérifiées avant qu'un modèle ne tourne : dates, doublons, trous, fréquence, longueur d'historique, valeurs aberrantes. Un audit qui échoue bloque la prévision.

> **Le niveau de confiance n'est jamais arrondi.**
> Si un modèle ne sait pas produire l'intervalle demandé, il est écarté. La demande de l'utilisateur n'est pas dégradée pour faire entrer un modèle dans le cadre.

> **Un modèle n'est « meilleur » que s'il bat les méthodes de référence.**
> Quatre méthodes simples servent d'étalon. Un modèle avancé qui ne fait pas mieux n'est pas présenté comme le bon choix.

> **Les statistiques viennent du moteur, jamais du modèle de langage.**
> Décomposition, anomalies, importance des variables, dérive : l'agent a pour consigne de les lire et non de les produire. Une analyse indisponible est annoncée comme indisponible.

> **Un ensemble de modèles n'est pas validé.**
> Combiner quatre modèles validés ne produit pas un modèle validé, et Forecast le marque explicitement.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent ne sait pas faire de prévisions » | Groupe Forecast éteint par défaut | L'activer dans les réglages |
| « L'agent demande de choisir un modèle » | Mode manuel sans modèle désigné | En choisir un dans l'espace Forecast, ou passer en automatique |
| « L'audit refuse mes données » | Dates désordonnées, trous, doublons, historique trop court | Corriger le fichier — le détail est dans le résultat de l'audit |
| « L'agent refuse mon niveau de confiance » | Aucun modèle disponible ne le supporte | Comportement voulu ; changer de niveau ou de modèle |
| « La validation n'a évalué que deux modèles sur cinq » | Échecs signalés dans le résultat | L'agent doit les mentionner ; ne pas lire le classement comme complet |
| « Le résultat n'est pas dans l'espace Forecast » | L'analyse est enregistrée et devrait y apparaître | Vérifier la liste des analyses |
| « La prévision est très lente » | Modèle lourd, ou historique long | Consulter les besoins matériels des modèles |

---

## Renvois

- `08-forecast/vue-densemble.md` — l'espace Forecast complet
- `08-forecast/donnees-et-audit.md` — le détail de l'audit
- `08-forecast/selection-du-modele.md` — les politiques manuelle et automatique
- `08-forecast/evaluation-et-comparaison.md` — validation, mesures, méthodes de référence
- `08-forecast/scenarios-notes-rapports.md` — scénarios et annotations
- `05-outils/vue-densemble.md` — activer le groupe

---

## Points à confirmer

- **Cette page décrit les outils, pas le moteur.** Tout ce qui est écrit ici vient des définitions d'outils et des limites, qui font autorité sur ce que l'agent peut demander. Le comportement réel du moteur de prévision — modèles disponibles, calcul des mesures, exécution locale ou distante — n'a **pas** été lu. La section 8 devra le faire, et pourra contredire cette page sur des détails d'exécution.
- **La définition des outils change selon la politique de sélection active.** C'est une conception peu commune : l'agent ne voit pas les mêmes paramètres en mode manuel et en mode automatique. Le site doit décider s'il l'explique — c'est éclairant pour un lecteur technique, superflu pour les autres.
- Les **mesures d'exactitude** ne sont pas nommées dans cette page. Elles sont listées dans le fichier de suivi de la section 8. À harmoniser pour éviter deux vocabulaires.
- Je n'ai **pas vérifié à l'écran** ce que voit l'utilisateur pendant qu'une prévision lancée depuis une conversation s'exécute : barre de progression, résultat inséré dans la conversation, lien vers l'espace Forecast.
- Le comportement quand l'**identifiant de sélection expire** en cours de tâche est décrit dans les consignes (l'agent doit reconsulter les modèles) mais n'a pas été vérifié dans le code.
