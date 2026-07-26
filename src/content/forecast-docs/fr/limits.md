# Limites

Forecast prédit une trajectoire probable. Il ne garantit pas que le futur arrivera exactement comme prévu.

## Ce qu'il faut retenir

Une prévision dépend toujours :

- de la qualité de l'historique ;
- de la stabilité du phénomène observé ;
- des variables de contexte disponibles ;
- de l'horizon demandé ;
- du modèle choisi.

Si les données sont faibles, incohérentes ou incomplètes, la prévision sera fragile.

## Ce que Forecast ne peut pas garantir

Forecast ne garantit pas :

- un résultat futur certain ;
- une causalité réelle entre deux variables ;
- une bonne prédiction après une rupture brutale ;
- une interprétation métier automatique ;
- une fiabilité identique sur tous les domaines.

Exemple : si une crise, une panne, une guerre de prix ou un événement exceptionnel arrive sans être représenté dans les données, le modèle peut sous-estimer le changement.

## Modèles locaux et cloud

Les modèles locaux gardent le calcul sur la machine.

Les modèles cloud nécessitent d'envoyer les données utiles au provider configuré. Il faut donc éviter d'envoyer des données sensibles si ce n'est pas acceptable pour le cas d'usage.

## Bonne pratique

Une prévision doit être utilisée comme aide à la décision.

La bonne lecture est :

- tendance probable ;
- fourchette de risque ;
- variables qui peuvent expliquer le mouvement ;
- scénarios alternatifs ;
- décision humaine finale.
