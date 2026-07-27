# `sundial-128m`

## Carte exacte

| Champ | Valeur Beaver |
| --- | --- |
| Famille | Sundial |
| Paramètres | 128 M |
| Disque | 513 Mo |
| RAM estimée | 2 400 Mo |
| VRAM estimée | 1 000 Mo |
| Contexte | jusqu'à 2 880 |
| Horizon direct / maximal | 720 / 1 024 |
| Confiance | tout pourcentage entier de 50 % à 99 % |
| Sortie | trajectoires probabilistes |
| Covariables / joint | non / non |

## Choisis-le

Choisis Sundial lorsque des trajectoires futures et la forme de la distribution sont utiles. Utilise une graine et un nombre d'échantillons enregistrés.

## Écarte-le

Écarte-le pour des covariables, des dépendances jointes ou si une simple prévision ponctuelle suffit avec beaucoup moins de coût.

## Compare et décide

Compare-le à un modèle à quantiles continus et à une baseline. Mesure calibration, stabilité entre graines, durée et mémoire. Valide toute extension au-delà de 720 pas.

## Vérifie

Vérifie le runtime épinglé et la [fiche THUML Sundial 128M](https://huggingface.co/thuml/sundial-base-128m).
