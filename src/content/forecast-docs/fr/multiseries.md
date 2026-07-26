# Multi-séries

Le multi-séries permet de prédire plusieurs séries dans une même analyse. Au lieu de lancer une prévision séparée pour chaque objet, Forecast reçoit un seul dataset avec une colonne qui identifie chaque série.

## Exemple

Un fichier peut contenir les ventes de plusieurs magasins :

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Ici, `magasin` est la colonne série.

Forecast comprend qu'il doit prédire les ventes pour `paris` et pour `lyon`.

## À quoi ça sert

Le multi-séries est utile quand plusieurs séries ont une logique commune.

Exemples :

- ventes par magasin ;
- commandes par restaurant ;
- trafic par serveur ;
- prix par actif ;
- incidents par région.

Le modèle peut exploiter plus d'information qu'une prévision isolée, surtout si les séries se ressemblent ou partagent des variables de contexte.

## Horizon par série

Chaque série doit fournir une structure temporelle cohérente.

Si l'horizon est `31`, chaque série doit avoir 31 points futurs à prédire.

Exemple :

```text
paris -> 31 lignes futures
lyon  -> 31 lignes futures
```

Un horizon incohérent rend la comparaison difficile et peut bloquer le modèle.

## Lecture dans Forecast

Dans l'interface, l'utilisateur peut sélectionner la série affichée.

Les scénarios peuvent ensuite être lus :

- sur une série précise ;
- sur plusieurs séries ;
- en comparaison avec la prévision de base.

Le multi-séries ne change pas le principe de Forecast. Il ajoute simplement une dimension : "quelle série est en train d'être prédite ?"
