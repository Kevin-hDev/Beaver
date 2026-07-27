# Capacités, preuve et fraîcheur

## Utilise le vocabulaire exact

Une capacité reçoit un état :

- **annoncée** : l'auteur la décrit ;
- **présente en amont** : le checkpoint et l'API l'exposent ;
- **câblée** : l'adapter Beaver transmet les données ;
- **testée** : un test vérifie le comportement ;
- **mesurée** : un backtest local démontre l'utilité ;
- **non disponible** : l'intégration ne l'utilise pas ;
- **inconnue** : la preuve manque.

Tu filtres sur `testée` quand la capacité est obligatoire.

## Distingue la maturité

| État | Signification |
| --- | --- |
| C0 | catalogué |
| C1 | adapter présent |
| C2 | runtime préparé |
| C3 | smoke test réussi |
| C4 | flux complet validé |
| C5 | backtesté sur le profil courant |
| C6 | surveillé en production |

Tu ne confonds pas `Prêt` avec C5.

## Classe les sources

| Niveau | Source |
| --- | --- |
| S0 | non vérifiable |
| S1 | auteur ou model card |
| S2 | prépublication ou benchmark fournisseur |
| S3 | article évalué, benchmark indépendant ou standard |
| S4 | reproduction indépendante ou convergence S3 |

Tu utilises S pour la source et C pour Beaver. Tu ne fusionnes pas les deux.

## Sépare statique et vivant

Les fiches statiques décrivent :

- architecture ;
- contexte et horizon amont ;
- sorties ;
- licence ;
- usages plausibles ;
- limites connues ;
- sources.

Le backend vivant décrit :

- installation ;
- disponibilité ;
- confiance exacte ;
- capacité câblée ;
- matériel ;
- ticket ;
- mesures locales ;
- incidents.

Tu donnes toujours priorité au backend vivant.

## Invalide une preuve

Tu réexamines après :

- changement de dataset ou horizon ;
- nouvelle version du checkpoint ;
- changement d'adapter ;
- changement du runtime ;
- nouvelle licence ;
- nouvel OS ou accélérateur ;
- dérive de performance ;
- sélection expirée.

## Maintiens les fiches

Tu vérifies la date et les sources avant une affirmation dynamique. Tu ne copies pas éternellement un hash, un prix, un quota ou un leaderboard dans le raisonnement.

Tu n'écris jamais `X est le meilleur`. Tu écris des conditions et une méthode de vérification.
