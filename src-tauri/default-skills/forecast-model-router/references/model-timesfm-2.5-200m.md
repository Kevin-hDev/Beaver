# `timesfm-2.5-200m`

## Carte exacte

| Champ | Valeur Beaver |
| --- | --- |
| Famille | TimesFM 2.5 |
| Paramètres affichés | 200 M |
| Disque | 925 Mo |
| RAM estimée | 4 200 Mo |
| VRAM estimée | 1 800 Mo |
| Contexte | jusqu'à 16 384 |
| Horizon maximal | 1 000 |
| Confiance | 60 % ou 80 % uniquement |
| Séries | unique ou multiples indépendantes |
| Covariables | passées et futures numériques |

## Choisis-le

Choisis-le lorsque le contexte long ou les covariables numériques justifient son coût mémoire, et seulement pour une confiance compatible.

## Écarte-le

Écarte-le pour une confiance différente de 60 % ou 80 %, une dépendance jointe, des covariables non numériques ou une machine sans marge.

## Compare et décide

Compare-le à Chronos-2 avec une ablation des covariables, puis à une baseline saisonnière. Garde-le seulement si son gain est stable et compense mémoire et durée.

## Vérifie

Vérifie le runtime courant et la [fiche Google TimesFM 2.5](https://huggingface.co/google/timesfm-2.5-200m-pytorch).
