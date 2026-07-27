# Famille TimesFM

## Positionnement

Utilise TimesFM 2.5 comme candidat local à long contexte, notamment lorsque des covariables numériques compatibles doivent être prises en compte.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | Google |
| Exécution | locale, CPU ou GPU |
| Licence à vérifier | Apache-2.0 |
| Taille affichée | 200 M |
| Séries | une série ou plusieurs séries indépendantes |
| Covariables | passées et futures numériques câblées |
| Dépendances jointes | non |
| Contexte | jusqu'à 16 384 points |
| Horizon catalogue | jusqu'à 1 000 points |
| Incertitude | intervalles centraux fixes de 60 % ou 80 % |

## Route la famille

Choisis-la lorsque :

- l'historique long contient une information utile ;
- le niveau de confiance demandé vaut exactement 60 % ou 80 % ;
- les covariables numériques sont disponibles et alignées ;
- la machine dispose d'une marge mémoire suffisante.

Écarte-la pour une confiance à 90 %, 92 % ou toute autre valeur non exposée. Écarte-la aussi si l'utilisateur demande une dépendance multivariée jointe.

## Valide les covariables

Vérifie la forme, le type numérique, le calendrier et la disponibilité future. Compare avec une version sans covariables. Refuse toute variable calculée avec des données postérieures au cutoff.

## Compare

Compare TimesFM à Chronos-2 pour les covariables et le contexte long. Compare aussi à une baseline saisonnière et à un modèle compact afin de mesurer si le coût mémoire apporte un gain réel.

## Sources vivantes

Vérifie le [dépôt officiel TimesFM](https://github.com/google-research/timesfm) et la [fiche officielle 2.5](https://huggingface.co/google/timesfm-2.5-200m-pytorch). N'extrapole pas une capacité amont que l'adaptateur Beaver ne déclare pas.
