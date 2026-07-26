# Vue d'ensemble

Forecast sert à prévoir l'évolution future d'une donnée mesurable. Le module analyse l'historique, les tendances récentes et les variables de contexte pour produire une prédiction datée, avec une marge d'incertitude et des scénarios comparables.

## Définition simple du forecasting

Le forecasting consiste à observer des données passées et actuelles pour estimer ce qui peut arriver ensuite.

Exemples :

- prévoir les ventes des 30 prochains jours ;
- estimer le chiffre d'affaires du mois suivant ;
- anticiper la charge serveur des prochaines heures ;
- projeter le prix ou le volume d'un actif ;
- simuler l'effet d'un contexte futur connu.

Le modèle ne lit pas l'avenir. Il calcule une trajectoire probable à partir de schémas visibles dans les données.

## Ce que Forecast ajoute à un chat LLM

Un LLM peut lire un tableau et écrire une explication. Forecast ajoute un moteur spécialisé qui calcule réellement une série future.

La différence est importante :

| Chat LLM seul | Forecast |
| --- | --- |
| Explique un fichier | Calcule des points futurs datés |
| Peut raisonner qualitativement | Produit une courbe numérique |
| Peut inventer si les données sont floues | Utilise un contrat de données strict |
| Résume une tendance | Génère une prévision, des bornes et des scénarios |

Le LLM reste utile autour du moteur : il prépare les données, choisit les colonnes, peut chercher des informations sur le web, construit un dataset, lance Forecast, puis explique le résultat.

## Objet principal : la cible

La cible est la colonne que Forecast doit prédire.

Exemples :

- `ventes`
- `ca_total_eur`
- `commandes_total`
- `temperature`
- `stock_price`
- `incidents_count`

Toute la prévision tourne autour de cette cible : le modèle apprend son comportement passé, puis estime ses valeurs futures.

## Ce que contient un résultat Forecast

Un résultat Forecast contient :

- les valeurs historiques utilisées ;
- la prévision future point par point ;
- une plage d'incertitude ;
- les variables de contexte utilisées ;
- les scénarios créés à partir de cette prévision ;
- les métadonnées nécessaires pour relire, comparer et exporter le résultat.

Ce n'est pas un fichier passif. C'est un objet de travail complet pour comprendre ce qui est probable, ce qui est risqué, et ce qui change si le contexte évolue.

## Logique générale

Le workflow standard est :

1. fournir un dataset ;
2. choisir la date, la cible et éventuellement les séries ;
3. sélectionner les variables de contexte utiles ;
4. lancer un modèle de prévision ;
5. lire la courbe future et l'incertitude ;
6. créer des scénarios pour tester des hypothèses ;
7. demander au LLM d'expliquer les résultats ou de préparer de nouvelles données.
