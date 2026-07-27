# `tirex-35m`

## Carte exacte

| Champ | Valeur Beaver |
| --- | --- |
| Famille | TiRex |
| Paramètres | 35 M |
| Disque | 141 Mo |
| RAM estimée | 900 Mo |
| VRAM estimée | 420 Mo |
| Contexte | jusqu'à 2 048 |
| Horizon maximal | 1 024 |
| Confiance | 60 % ou 80 % uniquement |
| Covariables / joint | non / non |

## Choisis-le

Choisis TiRex comme candidat probabiliste compact si la licence, l'OS et le runtime courant sont validés.

## Écarte-le

Écarte-le pour une confiance non supportée, des covariables, des dépendances jointes ou un smoke test invalide.

## Compare et décide

Compare-le à Chronos-Bolt, Kairos et une baseline. Sur Apple Silicon et petit lot, mesure CPU et accélération disponible au lieu de présumer le GPU supérieur.

## Vérifie

Vérifie la plateforme, la licence et la [fiche NX-AI TiRex](https://huggingface.co/NX-AI/TiRex).
