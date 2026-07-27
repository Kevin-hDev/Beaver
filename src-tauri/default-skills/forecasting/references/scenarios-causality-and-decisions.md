# Scénarios, causalité et décisions

## Distingue les sorties

- Une **prévision** cherche la distribution du futur observable.
- Un **scénario** décrit un futur conditionnel plausible.
- Un **stress test** pousse une hypothèse défavorable ou extrême.
- Une **simulation** applique un mécanisme supposé.
- Une **estimation causale** compare une intervention à un contrefactuel.

Tu étiquettes chaque sortie. Tu ne leur attribues pas la même preuve.

## Construis des scénarios utiles

Tu pars de la décision, puis :

1. Tu identifies tendances, contraintes et incertitudes critiques.
2. Tu construis trois à cinq futurs réellement distincts.
3. Tu explicites les mécanismes et hypothèses.
4. Tu définis les signaux précoces de chaque trajectoire.
5. Tu testes les options dans tous les scénarios.
6. Tu identifies décisions robustes, paris conditionnels et points de bascule.

Tu évites le trio optimiste, central et pessimiste sans mécanisme. Tu n'ajoutes pas de probabilités arbitraires.

## Traite un « what-if »

Tu nommes une modification mécanique `projection conditionnelle`. Tu conserves la baseline, l'hypothèse, le sens de variation, l'amplitude et la durée.

Si la variable d'intervention influence le système, tu refuses de présenter l'ajustement comme un effet causal.

## Exige une preuve causale

Selon la question, tu demandes :

- expérience contrôlée ;
- expérience naturelle ;
- groupe de comparaison ;
- graphe causal ;
- hypothèses d'identification ;
- série temporelle interrompue ;
- différences de différences ;
- contrôle synthétique ;
- analyse de sensibilité aux facteurs cachés.

Tu sépares estimation de l'effet et prévision du résultat futur. Tu vérifies la stabilité du mécanisme.

## Évalue la décision

Tu définis :

- actions disponibles ;
- contraintes ;
- règle de décision ;
- coûts des erreurs ;
- utilité ou perte ;
- décision de référence ;
- regret ;
- tolérance au risque.

Tu backtestes la chaîne `prévision -> décision -> résultat` lorsque possible. Tu ne conclus pas qu'une baisse de MAE diminue automatiquement le coût.

## Mesure la valeur de l'information

Avant une recherche, une donnée ou un modèle coûteux :

- Tu estimes si l'information peut changer la décision.
- Tu compares son coût à la réduction attendue du regret.
- Tu privilégies les inconnues à forte valeur décisionnelle.
- Tu arrêtes la recherche lorsque la décision devient stable.

## Red-teame le résultat

Tu cherches :

- taux de base ignoré ;
- hypothèse cachée ;
- explication concurrente ;
- source corrélée ou fragile ;
- acteur adaptatif ;
- conséquence de second ordre ;
- rupture rare ;
- preuve qui forcerait une révision.

Tu renforces la validation humaine pour les décisions irréversibles ou à fort impact.
