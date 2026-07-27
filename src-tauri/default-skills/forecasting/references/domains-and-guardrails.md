# Domaines et garde-fous

## Adapte le protocole

| Domaine | Méthodes ou références utiles | Risques |
| --- | --- | --- |
| demande et retail | saisonnalité, intermittent, hiérarchie, promotions, coût stock | demande censurée, nouveaux produits |
| supply chain | demande + délai, simulation, scénarios | bullwhip, dépendances fournisseurs |
| énergie | charge, prix, météo, quantiles, scénarios | extrêmes, prix négatifs, déséquilibre |
| économie | facteurs, BVAR, enquêtes, nowcast | révisions, politiques, vintages |
| finance | random walk, volatilité, quantiles, stress | fuite, coûts, sur-optimisation |
| météo | ensembles physiques et post-traitement | extrêmes et fausses alertes |
| climat | modèles physiques et trajectoires conditionnelles | scénarios non probabilisés |
| santé | nowcast, mécaniste, statistique, ensembles | retards, révisions, impact public |
| géopolitique | taux de base, foule, événements, scénarios | désinformation, résolution ambiguë |
| agriculture | météo, culture, télédétection | données locales et alertes |
| opérations | saisonnalité, anomalies résiduelles, capacité | changements de système |
| catastrophes | impacts, détection, scénarios | alerte officielle et pertes humaines |

## Renforce les usages sensibles

Pour santé, finance, droit, sûreté, infrastructures critiques et catastrophes :

- Tu imposes une validation humaine.
- Tu empêches toute décision autonome irréversible.
- Tu conserves toutes les versions et preuves.
- Tu définis un plan de repli.
- Tu communiques le périmètre réglementaire.
- Tu distingues aide à la décision et conseil professionnel.

## Retail et stock

Tu vérifies ruptures de stock, cannibalisation, changements d'assortiment, promotions réellement connues, retours et hiérarchies. Tu mesures coût de rupture, surstock et niveau de service en plus de l'erreur.

## Finance

Tu utilises random walk comme baseline. Tu sépares prévision de rendement, volatilité et risque. Tu évalues après coûts, glissement et contraintes. Tu interdis fuite, biais de survivants et optimisation répétée sur le test.

## Énergie et météo

Tu versionnes les prévisions météo disponibles au cutoff. Tu évalues les extrêmes séparément. Tu ne remplaces jamais une alerte d'une autorité compétente par une sortie Beaver.

## Santé

Tu utilises les dates de publication et révision. Tu contrôles couverture, timing du pic, retard de déclaration et changement de comportement. Tu ne présentes jamais une prévision comme diagnostic ou instruction médicale.

Tu sépares la demande sanitaire de la capacité nécessaire. Pour passer d'admissions prévues à une décision de capacité, tu explicites durée de séjour, occupation, files, transferts, personnel, délai d'activation et contraintes opérationnelles. Tu évalues cette chaîne séparément et tu fais valider hypothèses, seuils d'escalade et autorité décisionnelle. Tu interdis toute action automatique irréversible.

## Géopolitique

Tu définis une règle de résolution publique. Tu recherches désaccords, sources indépendantes et taux de base. Tu limites les probabilités extrêmes et tu surveilles la désinformation.

## Choisis les sources par spécialité

Tu privilégies :

- autorités statistiques et archives de vintages ;
- services météo et climat officiels ;
- organismes sanitaires et hubs ouverts ;
- banques centrales et rapports d'évaluation ;
- articles et benchmarks reproductibles ;
- experts avec historique scoré.

Tu ne classes pas une source par pays ou prestige. Tu la classes par fraîcheur, transparence, protocole et preuve.
