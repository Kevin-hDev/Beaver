# Famille MOIRAI 2.0

## Positionnement

Utilise MOIRAI 2.0 R Small comme candidat local compact à confronter aux autres modèles généralistes, avec une vigilance renforcée sur la licence et les transformations du runtime.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | Salesforce |
| Exécution | locale, CPU ou GPU |
| Poids | licence CC-BY-NC-4.0 à vérifier |
| Séries | une série ou plusieurs séries indépendantes |
| Covariables | non câblées dans Beaver |
| Dépendances jointes | non |
| Contexte adaptateur | plafonné à 1 680 points |
| Horizon catalogue | jusqu'à 1 024 points |
| Incertitude | confiance entière de 50 % à 99 % |

## Applique la contrainte de licence

Traite un usage commercial comme non admissible sans autorisation séparée. Distingue la licence du code et celle des poids. En cas de doute, bloque le routage vers MOIRAI.

## Route la famille

Choisis-la pour un essai local non commercial, une série indépendante et un budget modeste. Écarte-la lorsqu'une fréquence autre que celle transformée par l'adaptateur doit être préservée strictement, lorsque des covariables sont nécessaires ou lorsque l'usage est commercial.

## Vérifie la transformation temporelle

L'adaptateur courant impose une représentation quotidienne. Contrôle que cette transformation respecte réellement le calendrier métier. Exige un backtest au même rythme et refuse toute conclusion si la transformation change le sens de la série.

## Compare

Compare MOIRAI à Chronos-Bolt, Kairos et TiRex sur les mêmes plis. Ne privilégie pas sa compacité si la licence, la fréquence ou la calibration ne convient pas.

## Sources vivantes

Vérifie le [dépôt officiel Uni2TS](https://github.com/SalesforceAIResearch/uni2ts) et la [fiche MOIRAI 2.0 R Small](https://huggingface.co/Salesforce/moirai-2.0-R-small).
