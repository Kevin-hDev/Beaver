# Vue d'ensemble

Forecast est directement lié à la conversation active. Le LLM prépare ou recherche les données, lance les calculs et explique les résultats. L'utilisateur garde le chat comme centre de commande et dispose de deux surfaces complémentaires pour lire et approfondir une analyse.

## Parcours principal

Le parcours normal suit cet ordre :

1. l'utilisateur décrit ce qu'il veut prévoir dans le chat ;
2. le LLM lit, crée ou enrichit les données nécessaires ;
3. Forecast contrôle la qualité des données ;
4. le modèle est imposé en mode Manuel ou sélectionné parmi les candidats sûrs en mode Auto ;
5. Forecast calcule et sauvegarde la prévision ;
6. le panneau affiche immédiatement le résultat principal ;
7. l'utilisateur poursuit la discussion ou ouvre l'espace Forecast pour aller plus loin.

Il n'existe pas de chat Forecast séparé. Pour demander une explication, une comparaison ou une nouvelle prévision, écrivez simplement un message dans la conversation.

## Deux surfaces complémentaires

| Surface | Rôle |
| --- | --- |
| Chat | Préparer les données, piloter le LLM et demander des explications |
| Panneau Forecast | Lire rapidement la courbe, les indicateurs et les avertissements essentiels |
| Espace Forecast | Explorer les données, les graphiques, les évaluations, les scénarios, les notes et le rapport |

Le panneau reste volontairement compact. L'espace Forecast s'ouvre dans une fenêtre dédiée, sans masquer ni remplacer la conversation.

## Espace Forecast

L'espace Forecast reste lié à la session et à l'analyse actives. Si vous choisissez une autre analyse dans le panneau, la fenêtre ouverte se met à jour automatiquement.

Ses sections sont :

| Section | Contenu |
| --- | --- |
| Données | Résumé du dataset, mapping, qualité et aperçu des lignes |
| Prévision | Courbe principale, incertitude, saisonnalité, filtres et tableau des points |
| Évaluation | Backtest temporel, modèles de référence et fiabilité des intervalles |
| Comparaison | Classement comparable des modèles et création éventuelle d'un ensemble |
| Scénarios | Création et modification d'hypothèses |
| Notes | Contexte, risques, décisions et annotations |
| Rapport | Analyse détaillée et exports |

## Ce que sauvegarde une analyse

Une analyse conserve notamment :

- les colonnes et paramètres réellement utilisés ;
- le profil de qualité des données ;
- le modèle et la source de sa sélection ;
- la prévision centrale et ses intervalles ;
- les scénarios, notes et annotations ;
- les résultats de backtest lorsqu'ils existent ;
- les informations de provenance nécessaires à la reproductibilité.

## Ce qu'il faut retenir

Forecast aide à produire une estimation structurée, pas une certitude. Une courbe convaincante doit toujours être lue avec la qualité des données, l'incertitude, les modèles de référence et les limites du contexte disponible.
