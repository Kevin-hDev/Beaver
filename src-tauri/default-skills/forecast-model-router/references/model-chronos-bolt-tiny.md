# `chronos-bolt-tiny`

## Carte exacte

| Champ | Valeur Beaver |
| --- | --- |
| Famille | Chronos-Bolt |
| Paramètres | 9 M |
| Disque | 35 Mo |
| RAM estimée | 150 Mo |
| VRAM estimée | 60 Mo |
| Horizon maximal | 1 000 |
| Confiance | tout pourcentage entier de 50 % à 99 % |
| Exécution | locale, CPU ou GPU |

## Choisis-le

Choisis Tiny pour un test rapide, une machine contrainte ou une baseline neuronale probabiliste sur une série indépendante. Utilise-le aussi comme seuil de valeur face aux variantes plus grandes. Ne le prends pas comme premier candidat automatique lorsque la qualité maximale domine.

## Écarte-le

Écarte-le si des covariables, des dépendances conjointes ou un contexte supérieur à 2 048 sont obligatoires. Réduis sa priorité si le backtest révèle un sous-ajustement stable.

## Compare et décide

Compare-le à la baseline naïve et aux variantes prévues par le profil de priorité. En qualité maximale, inclus directement Base ou le candidat fort admissible dans ce tournoi. Garde Tiny s'il reste dans la bande d'équivalence. Enregistre durée et mémoire avec les métriques.

## Vérifie

Vérifie l'état `installed/ready`, la capacité renvoyée par `forecast_models` et la fiche [Amazon Chronos-Bolt Tiny](https://huggingface.co/amazon/chronos-bolt-tiny) si une donnée statique semble périmée.
