# Famille TiRex

## Positionnement

Utilise TiRex 35M comme candidat local compact et probabiliste pour des séries indépendantes, sous réserve des conditions de licence et de plateforme.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | NX-AI |
| Exécution | locale, CPU ou GPU |
| Licence | vérifie les conditions courantes |
| Paramètres | 35 M |
| Séries | une série ou plusieurs séries indépendantes |
| Covariables | non câblées |
| Dépendances jointes | non |
| Contexte | jusqu'à 2 048 points |
| Horizon catalogue | jusqu'à 1 024 points |
| Incertitude | intervalles centraux fixes de 60 % ou 80 % |

## Route la famille

Choisis-la lorsque le budget est limité, que 60 % ou 80 % convient et que le runtime est validé sur l'OS courant. Écarte-la pour des covariables, une confiance à 90 % ou des dépendances jointes.

## Vérifie la plateforme

Contrôle l'état réel du runtime et le smoke test avant de sélectionner. Ne généralise pas un support annoncé sur macOS ou Linux à Windows. Sur Apple Silicon, compare CPU et accélération disponible : un petit lot peut être plus rapide sur CPU.

## Compare

Compare TiRex à Chronos-Bolt, Kairos, MOIRAI si sa licence convient, et à une baseline. Évalue la calibration séparément de l'erreur de point.

## Sources vivantes

Vérifie le [dépôt officiel TiRex](https://github.com/NX-AI/TiRex) et la [fiche du modèle](https://huggingface.co/NX-AI/TiRex). Laisse le test de préparation Beaver confirmer la plateforme.
