# `flowstate-r1`

## Carte exacte

| Champ | Valeur Beaver |
| --- | --- |
| Famille | FlowState |
| Paramètres | 9 M |
| Disque | 36 Mo |
| RAM estimée | 650 Mo |
| VRAM estimée | 260 Mo |
| Contexte | jusqu'à 2 048 |
| Horizon maximal | 4 096 |
| Confiance | 60 % ou 80 % uniquement |
| Covariables / joint | non / non |

## Choisis-le

Choisis R1 comme premier FlowState lorsque la saisonnalité est connue et que 2 048 points de contexte suffisent.

## Écarte-le

Écarte-le si la période saisonnière est spéculative, si la confiance diffère de 60 % ou 80 %, ou si des covariables ou dépendances jointes sont requises.

## Compare et décide

Compare-le à R1.1, à une baseline saisonnière et à une autre famille compacte. Garde R1 si le contexte supplémentaire de R1.1 n'apporte pas de gain utile.

## Vérifie

Vérifie la révision installée et la [fiche IBM FlowState R1](https://huggingface.co/ibm-granite/granite-timeseries-flowstate-r1).
