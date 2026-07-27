# Famille Kairos

## Positionnement

Utilise Kairos comme famille locale compacte pour des séries indépendantes. Considère ses trois tailles comme un tournoi, pas comme une hiérarchie automatique.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | Foundation Model Research |
| Exécution | locale, CPU ou GPU |
| Licence à vérifier | Apache-2.0 |
| Tailles | 10 M, 23 M, 50 M |
| Séries | une série ou plusieurs séries indépendantes |
| Covariables | non câblées |
| Dépendances jointes | non |
| Contexte | jusqu'à 2 048 points |
| Horizon direct | 128 points |
| Horizon catalogue | jusqu'à 1 024 points par extension |
| Incertitude | intervalles centraux fixes de 60 % ou 80 % |

## Route la famille

Choisis-la pour une exécution locale légère, un niveau de 60 % ou 80 % et une tâche sans covariables. En qualité maximale, inclus 50 M dès le premier tour. En équilibre, compare 23 M et 50 M. En rapidité, commence par 10 M ou 23 M.

## Contrôle l'horizon

Traite toute prévision au-delà de 128 points comme une extension récursive de l'adaptateur. Backteste exactement cette profondeur. Surveille la dérive, le lissage excessif et l'élargissement des intervalles.

## Encadre les options

Enregistre les options de génération, de positivité et de symétrisation. Active `preserve_positivity` seulement si la cible ne peut jamais être négative. Ne change qu'une option à la fois dans une comparaison.

## Compare

Compare les tailles entre elles, à Chronos-Bolt et à TiRex. Garde la variante la plus qualitative lorsque son gain est utile et stable. Garde la moins coûteuse uniquement dans la bande d'équivalence.

## Sources vivantes

Vérifie le [dépôt officiel Kairos](https://github.com/foundation-model-research/Kairos) et la fiche Hugging Face de la taille exacte.
