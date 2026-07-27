# Cadrage et contrat d'une prévision

## Sépare les problèmes

Tu classes la demande avant de choisir une méthode :

- **prévision** : estime une valeur ou une probabilité future ;
- **projection conditionnelle** : prolonge sous des hypothèses annoncées ;
- **nowcast** : estime un présent encore incomplet ;
- **scénario** : explore un futur plausible sans prétendre donner sa probabilité ;
- **simulation** : calcule les conséquences d'un mécanisme supposé ;
- **causalité** : estime l'effet d'une intervention par rapport à un contrefactuel.

Tu refuses les substitutions suivantes :

- Tu ne présentes pas une corrélation comme une cause.
- Tu ne présentes pas un ajustement `+10 %` comme l'effet réel d'une intervention.
- Tu ne présentes pas un scénario comme la trajectoire la plus probable sans méthode probabiliste.
- Tu ne présentes pas une narration du LLM comme une preuve quantitative.

## Formule une question vérifiable

Tu définis :

1. la cible exacte ;
2. l'unité ;
3. la population, la série ou la zone ;
4. l'origine de prévision ;
5. la dernière information autorisée ;
6. l'horizon ;
7. la fréquence ou les issues possibles ;
8. la règle et la source de résolution ;
9. la décision aidée ;
10. le coût relatif des erreurs.

Pour un événement, tu précises aussi les cas ambigus, reports, annulations et date de fermeture. Tu ne modifies jamais la règle après avoir observé le résultat.

## Choisis la valeur centrale selon la perte

| Perte ou décision | Cible centrale habituelle |
| --- | --- |
| erreur quadratique | moyenne conditionnelle |
| erreur absolue | médiane conditionnelle |
| sous-prévision plus coûteuse | quantile supérieur adapté au ratio de coûts |
| sur-prévision plus coûteuse | quantile inférieur adapté au ratio de coûts |

Tu ne choisis pas la métrique après avoir regardé quel modèle gagne.

## Utilise un contrat persistant

Tu conserves au minimum :

```yaml
forecast_id: identifiant sûr
question: formulation figée
forecast_type: time_series | event | nowcast | scenario | causal
target: cible
unit: unité
population_or_series: périmètre
origin_time: instant de production
data_cutoff: dernière information autorisée
horizon: durée ou pas futurs
resolution: granularité ou issues
resolution_rule: vérité et source finales
decision_context: décision aidée
loss_or_cost: fonction de perte
data_snapshot: empreinte ou version
sources: sources datées
assumptions: hypothèses vérifiables
constraints: bornes, cohérence, confidentialité et ressources
status: draft | published | superseded | resolved
```

Après calcul, tu ajoutes modèle, baseline, protocole, résultats, calibration, limites, déclencheurs de mise à jour et provenance.

Après résolution, tu ajoutes vérité observée, erreurs, scores et valeur ajoutée des interventions humaines ou LLM.

## Pose peu de questions

Tu demandes une précision uniquement si elle change :

- le type de tâche ;
- la cible ou l'unité ;
- l'horizon ou la fréquence ;
- la disponibilité d'une covariable ;
- la fonction de perte ;
- la permission cloud ;
- le niveau de risque.

Sinon, tu avances avec une hypothèse clairement annoncée.
