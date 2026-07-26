# Scénarios

Un scénario sert à tester une hypothèse sur le futur. Il compare la prévision de base avec une trajectoire modifiée.

## Pourquoi créer un scénario

La prévision de base répond à : "que prévoit le modèle avec les données actuelles ?"

Un scénario répond à : "que se passe-t-il si le contexte change ?"

Exemples :

- que deviennent les ventes si une promotion est lancée ?
- que devient le chiffre d'affaires si le trafic baisse ?
- que devient un actif si le VIX augmente ?
- que devient la demande si un événement local est ajouté ?

Le scénario transforme Forecast en outil de simulation, pas seulement en graphe de prévision.

## Ajustement global

L'ajustement global applique une variation simple à la courbe.

Exemple :

```text
Prévision de base : 100, 110, 120
Scénario +10 %   : 110, 121, 132
```

Ce mode est rapide et lisible. Il ne relance pas le modèle, donc il ne comprend pas les relations entre variables.

## Scénario contextuel

Le scénario contextuel modifie les variables futures, puis relance le modèle.

Exemple :

```text
Hypothèse : vix_close +20 %
Effet attendu : le modèle recalcule la cible avec ce contexte de marché plus stressé.
```

Ce mode est plus important pour Chronos-2 et TimeGPT, car il utilise les covariables comme vrais signaux de prédiction.

## Variables modifiables

Les variables disponibles dépendent du dataset.

Elles peuvent représenter :

- environnement : météo, trafic, événements ;
- finance : volatilité, taux, indices, news score ;
- calendrier : weekend, jour férié, fin de mois ;
- métier : promo, stock, budget, campagne ;
- risque : alerte, incident, pression concurrentielle.

Chaque modification doit avoir un sens métier. Modifier une variable au hasard produit un scénario difficile à interpréter.

## Lecture dans le graphe

Quand un scénario est sélectionné, le graphe doit permettre de comparer :

- historique réel ;
- prévision de base ;
- prévision du scénario ;
- variables de contexte affichées ;
- différence entre base et scénario.

Schéma typique d'une comparaison :

```text
valeur
  ^
  |              ╭───── scenar (VIX +20 %)
  |           ╭──╯
  |       · ·─·      ← prévision de base
  |     ·
  |   ·
  | ·
  ──────────────────────────────> temps
       historique    │   futur
                     │
                horizon
```

La question principale n'est pas "la courbe est-elle différente ?", mais "quelle hypothèse a déplacé la trajectoire, à quelle date, et de combien ?"

## Bon usage

Un bon scénario doit être nommé clairement.

Exemples :

- `VIX +20% pendant 30 jours`
- `Promo week-end active`
- `Pluie forte semaine 2`
- `Trafic -15% après incident`

Un nom vague comme `test` ou `crash` rend la comparaison inutile quand plusieurs scénarios existent.
