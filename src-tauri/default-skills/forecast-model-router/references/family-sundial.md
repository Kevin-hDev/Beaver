# Famille Sundial

## Positionnement

Utilise Sundial comme modèle local génératif lorsque des trajectoires futures échantillonnées sont utiles pour étudier des distributions ou des scénarios.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | THUML |
| Exécution | locale, CPU ou GPU |
| Licence à vérifier | Apache-2.0 |
| Paramètres | 128 M |
| Séries | une série ou plusieurs séries indépendantes |
| Covariables | non câblées |
| Dépendances jointes | non |
| Contexte | jusqu'à 2 880 points |
| Horizon direct | 720 points |
| Horizon catalogue | jusqu'à 1 024 points |
| Incertitude | trajectoires et confiance entière de 50 % à 99 % |

## Route la famille

Choisis-la lorsque la forme de la distribution future ou les trajectoires importent plus qu'une simple valeur centrale. Écarte-la si des covariables ou des dépendances conjointes sont obligatoires.

## Contrôle l'échantillonnage

Traite le nombre de trajectoires comme un compromis entre stabilité et coût. Enregistre la graine, le nombre d'échantillons et les paramètres. Vérifie que les quantiles restent stables lorsque tu augmentes le nombre d'échantillons.

## Contrôle l'horizon

Valide par backtest toute extension au-delà de l'horizon direct. Ne confonds pas une limite catalogue avec une qualité garantie.

## Sécurise le runtime

Utilise uniquement le code distant épinglé et installé par Beaver. Ne télécharge ni n'exécute une révision flottante pendant la prévision.

## Compare

Compare Sundial à un modèle quantile continu et à une baseline. Mesure la calibration, la stabilité entre graines, la durée et la mémoire.

## Sources vivantes

Vérifie le [dépôt officiel Sundial](https://github.com/thuml/Sundial) et la [fiche Sundial 128M](https://huggingface.co/thuml/sundial-base-128m).
