# Évaluation et comparaison

L'évaluation mesure un modèle sur des portions historiques qu'il n'a pas vues pendant son calcul. Elle permet de comparer des résultats sur les mêmes fenêtres temporelles au lieu de se fier au nom ou à la taille d'un modèle.

## Backtest temporel glissant

Forecast découpe l'historique en plusieurs fenêtres. Pour chaque fenêtre, le modèle apprend uniquement sur le passé puis prévoit la période suivante.

Ce principe évite d'utiliser accidentellement des informations futures. Le nombre de fenêtres et l'horizon peuvent être réduits lorsque l'historique est trop court. L'interface affiche alors un avertissement.

## Modèles de référence

Chaque évaluation tente d'inclure des méthodes simples :

| Référence | Principe |
| --- | --- |
| Naive | Répète la dernière valeur connue |
| Naive saisonnier | Répète la valeur de la période comparable précédente |
| Drift | Prolonge la tendance moyenne observée |
| ETS | Modélise niveau, tendance et saisonnalité lorsque les données le permettent |

Un modèle avancé n'est utile que s'il apporte un gain réel par rapport à ces références.

## Métriques affichées

| Mesure | Lecture |
| --- | --- |
| MASE | Erreur comparée à une prévision naïve ; plus faible est meilleur |
| sMAPE | Erreur relative symétrique ; plus faible est meilleur |
| MAE | Écart absolu moyen dans l'unité de la cible |
| Couverture | Part des valeurs réelles situées dans l'intervalle annoncé |
| Durée | Temps observé pendant l'évaluation |
| Mémoire | Ressource maximale observée lorsqu'elle est disponible |

La couverture mesurée doit être comparée au niveau théorique demandé. Un intervalle à 80 % qui ne contient que 40 % des valeurs observées est mal calibré.

## Onglets Évaluation et Comparaison

Évaluation lance le backtest et affiche les résultats détaillés. Comparaison reprend uniquement les résultats homogènes et permet de voir les compromis entre précision, couverture, vitesse et ressources.

Un résultat peut être complet, partiel ou indisponible. Une évaluation partielle ne doit jamais être présentée comme une validation complète.

## Désigner un meilleur modèle

Forecast ne qualifie un modèle de meilleur que si :

- les modèles ont été testés sur les mêmes fenêtres ;
- le résultat est complet et exploitable ;
- les métriques pertinentes sont meilleures ;
- le modèle bat au moins une référence crédible ;
- les contraintes de l'utilisateur restent respectées.

Sans backtest comparable, parlez uniquement de modèle compatible ou recommandé selon ses capacités.

## Ensemble de modèles

Après un backtest multi-modèles réussi, l'onglet Comparaison peut créer un ensemble à partir de deux à quatre modèles valides. Forecast pondère leurs prévisions selon l'inverse du MASE.

L'ensemble est clairement marqué comme non évalué indépendamment. Il doit être backtesté séparément avant d'être présenté comme supérieur.
