# Famille TabPFN-TS

## Positionnement

Utilise TabPFN-TS-3 comme candidat expérimental puissant seulement lorsque la licence, les ressources et le runtime sont compatibles.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | Prior Labs |
| Exécution | locale, CPU ou GPU |
| Licence | restrictions possibles ; vérifie l'usage exact |
| Modèle visible | `tabpfn-ts-3` |
| Alias historique | `tabpfn-ts`, non proposé comme doublon |
| Séries | une série ou plusieurs séries indépendantes |
| Covariables | non câblées dans Beaver |
| Dépendances jointes | non |
| Horizon catalogue | jusqu'à 1 024 points |
| Incertitude | intervalles centraux fixes de 60 % ou 80 % |
| Mémoire catalogue | 8 Go de RAM et 8 Go de VRAM |

## Applique la contrainte de licence

Vérifie les conditions actuelles des poids, de la bibliothèque et de l'usage commercial. Bloque le candidat si l'autorisation nécessaire manque. Ne considère jamais une disponibilité technique comme une permission juridique.

## Route la famille

Choisis-la uniquement si le backend renvoie `tabpfn-ts-3`, si le niveau vaut 60 % ou 80 % et si la machine possède une marge mémoire réelle. Écarte-la lorsque des covariables ou une modélisation jointe sont obligatoires.

## Traite les dates

Fournis les dates réelles lorsque leur calendrier compte. Si le runtime reconstruit un calendrier quotidien, vérifie que cette hypothèse ne change pas la tâche.

## Compare

Compare-le à un candidat compact et à une baseline. Inclue la mémoire de pointe et la durée dans la décision. N'utilise pas l'alias historique comme second candidat.

## Sources vivantes

Vérifie le [dépôt officiel TabPFN Time Series](https://github.com/PriorLabs/tabpfn-time-series), la [fiche TabPFN 3](https://huggingface.co/Prior-Labs/tabpfn_3) et les licences publiées par Prior Labs.
