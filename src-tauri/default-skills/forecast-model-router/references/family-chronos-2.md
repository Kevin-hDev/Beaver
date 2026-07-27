# Famille Chronos-2

## Positionnement

Utilise Chronos-2 comme candidat local polyvalent lorsque le besoin inclut un contexte long, plusieurs séries indépendantes ou des covariables passées et futures.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | Amazon |
| Exécution | locale, CPU ou GPU |
| Licence à vérifier | Apache-2.0 |
| Paramètres | 120 M |
| Séries | une série, panel ou lot indépendant |
| Covariables | passées et futures câblées |
| Dépendances jointes | non garanties par Beaver |
| Contexte amont | jusqu'à 8 192 points selon le runtime |
| Horizon catalogue | jusqu'à 1 024 points |
| Incertitude | confiance entière de 50 % à 99 % |

## Route la famille

Choisis-la lorsque :

- des covariables locales sont nécessaires ;
- l'historique utile est long ;
- plusieurs séries doivent être prévues avec une interface de panel ;
- tu veux un candidat probabiliste local de taille intermédiaire.

Écarte-la si le besoin exige des dépendances multivariées conjointes. Ne confonds pas l'acceptation de plusieurs séries avec une modélisation de leurs interactions.

## Vérifie les covariables

Contrôle l'alignement temporel, la disponibilité au cutoff et la disponibilité future réelle. N'utilise jamais une valeur future reconstruite à partir de la cible. Lance une ablation sans covariables ; conserve-les seulement si le gain hors échantillon est stable et utile.

## Compare

Compare Chronos-2 à une baseline naïve, à Chronos-Bolt et à TimesFM si les deux sont admissibles. Ajoute Toto uniquement si les dépendances conjointes sont le besoin central. Mesure la qualité, la calibration, la mémoire et la durée sur les mêmes plis.

## Sources vivantes

Vérifie le [dépôt officiel Chronos](https://github.com/amazon-science/chronos-forecasting) et la [fiche Chronos-2](https://huggingface.co/amazon/chronos-2). Laisse le catalogue et le registre Beaver trancher les capacités présentes.
