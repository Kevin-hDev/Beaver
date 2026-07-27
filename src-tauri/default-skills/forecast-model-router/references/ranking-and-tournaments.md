# Classement, tournois et tailles

## Utilise le classement actuel

Avec `selection_basis=rolling_backtest`, tu respectes :

1. baseline battue ;
2. MASE ;
3. perte quantile ;
4. écart de couverture ;
5. sMAPE ;
6. MAE ;
7. RMSE ;
8. biais absolu ;
9. stabilité ;
10. mémoire ;
11. durée.

Tu ne recomposes pas un score sur 100.

## Superpose la décision métier

Pour une tâche critique ou une perte asymétrique :

1. Tu utilises le classement backend pour présélectionner des candidats techniquement solides.
2. Tu conserves la perte métier et le quantile décisionnel définis avant les résultats.
3. Tu backtestes la chaîne `prévision -> action -> coût ou regret`.
4. Tu refuses l'expression `meilleur modèle` si cette évaluation manque.
5. Tu peux retenir un candidat moins bien classé en MASE uniquement si la perte métier comparable démontre un avantage.

Tu documentes séparément le classement technique et la décision finale.

## Construis un tournoi comparable

Tu imposes :

- même empreinte de données ;
- mêmes origines ;
- même horizon ;
- mêmes informations disponibles ;
- mêmes transformations ;
- mêmes séries ;
- mêmes quantiles ;
- mêmes limites de temps et mémoire ;
- échecs conservés ;
- test final intact.

## Borne le tournoi

| Complexité | Candidats |
| --- | --- |
| simple | baselines + deux modèles adaptés à l'objectif |
| avancée | baselines + trois à cinq familles |
| complexe | trois candidats capables + sensibilités |
| critique | tournoi avancé + test final + validation indépendante |

Tu utilises une course par élimination : tu soumets d'abord tous les finalistes à un test court comparable, puis tu accordes plus de plis aux survivants.

## Choisis entre tailles

1. Tu fixes d'abord la priorité : qualité maximale, équilibre ou rapidité.
2. En qualité maximale, tu inclus immédiatement la variante compatible ayant la plus forte capacité ou les meilleures preuves.
3. En équilibre, tu compares cette variante à un candidat moins coûteux.
4. En rapidité, tu commences par une variante compacte et tu ajoutes une taille supérieure si nécessaire.
5. Tu compares sur les mêmes plis, puis tu mesures gain moyen et variabilité.
6. Tu définis un seuil de gain pratique avant de voir le classement final.
7. Tu choisis le meilleur résultat utile et stable ; tu sélectionnes le moins coûteux seulement en cas d'équivalence.

Tu appliques particulièrement cette règle à Chronos-Bolt, Toto et Kairos. Tu ne transfères pas automatiquement un ordre de taille entre deux architectures ou générations différentes.

## Traite l'absence de backtest

Sans preuve locale :

- Tu diversifies les familles.
- Tu inclus un candidat fort compatible lorsque la qualité maximale est prioritaire.
- Tu préfères une variante équilibrée lorsque l'utilisateur ne précise rien.
- Tu préfères une variante compacte uniquement lorsque la rapidité, la mémoire ou le coût domine.
- Tu utilises les ressources du matériel courant.
- Tu présentes le choix comme présélection.
- Tu proposes un backtest avant une décision importante.

## Compare les baselines

Tu vérifies Naive, Seasonal Naive, Drift et ETS lorsque disponibles. Tu conserves la prévision métier si elle existe.

Tu ne retiens pas un modèle avancé qui échoue à battre la baseline avec un gain matériel et stable.

## Construis un ensemble

Tu n'ensembles que deux à quatre modèles ayant réussi le même backtest. Tu compares à la moyenne simple. Tu vérifies calibration, coût et repli.

Tu signales qu'un ensemble créé après le backtest n'est pas automatiquement évalué comme un nouveau candidat.

## Présente la portée

Tu dis `meilleur sur ces données, ces fenêtres, cet horizon et cette perte`. Tu ne généralises pas à tous les domaines.
