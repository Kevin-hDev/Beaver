# Incertitude

Une prévision n'est jamais une vérité absolue. Forecast affiche donc une valeur centrale et une plage d'incertitude pour montrer le risque autour de la trajectoire.

## Valeur centrale

La valeur centrale est l'estimation principale du modèle.

Exemple :

```text
2026-06-01 -> 142 commandes prévues
```

Elle représente le scénario le plus probable selon les données utilisées.

## Bornes d'incertitude

Les bornes indiquent une zone probable autour de la valeur centrale.

Exemple :

```text
Prévision : 142
Borne basse : 128
Borne haute : 157
```

Lecture simple : le modèle estime 142, mais considère qu'une valeur autour de 128 à 157 reste plausible.

## Quantiles

Les modèles peuvent retourner des quantiles.

| Champ | Sens |
| --- | --- |
| q10 | Valeur basse probable |
| q50 | Valeur centrale ou médiane |
| q90 | Valeur haute probable |

Plus l'écart entre q10 et q90 est large, plus le modèle est incertain.

## Pourquoi l'incertitude augmente

L'incertitude peut augmenter quand :

- l'historique est court ;
- la cible varie fortement ;
- l'horizon est long ;
- les variables de contexte sont absentes ;
- une rupture récente apparaît dans les données ;
- plusieurs scénarios futurs sont possibles.

## Bon usage

La valeur centrale sert à lire la tendance.

La plage d'incertitude sert à lire le risque.

Une décision sérieuse doit regarder les deux.
