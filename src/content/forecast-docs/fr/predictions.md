# Prévisions

Une prévision est le résultat calculé par Forecast pour estimer les prochaines valeurs d'une cible. Elle répond à une question concrète : "si les données passées et le contexte connu restent cohérents, quelles valeurs peut-on attendre ensuite ?"

## Ce qu'une prévision représente

Une prévision contient une suite de points futurs.

Chaque point correspond à une date ou une période :

```text
2026-06-01 -> 142 commandes prévues
2026-06-02 -> 151 commandes prévues
2026-06-03 -> 149 commandes prévues
```

Ces valeurs ne sont pas une certitude. Elles représentent l'estimation du modèle.

## Entrées nécessaires

Pour lancer une prévision, Forecast a besoin de :

| Élément | Rôle |
| --- | --- |
| Date | Situe chaque ligne dans le temps |
| Cible | Valeur à prédire |
| Fréquence | Rythme des données : jour, heure, mois, etc. |
| Horizon | Nombre de points futurs à prédire |
| Modèle | Moteur utilisé pour calculer la trajectoire |

Les variables de contexte et le multi-séries ne sont pas obligatoires, mais ils deviennent importants dès qu'on veut expliquer ou simuler le futur.

## Horizon

L'horizon indique la profondeur de la prévision.

Exemples :

- horizon `24` avec fréquence horaire : prévoir les 24 prochaines heures ;
- horizon `31` avec fréquence quotidienne : prévoir les 31 prochains jours ;
- horizon `12` avec fréquence mensuelle : prévoir les 12 prochains mois.

Plus l'horizon est long, plus l'incertitude augmente généralement.

## Résultat et identifiant

Chaque lancement produit un identifiant `analysis_id`.

Cet identifiant ne signifie pas "fichier sauvegardé". Il sert à retrouver le résultat calculé : courbe future, incertitude, paramètres, variables, scénarios et annotations.

L'application l'utilise pour :

- rouvrir une prévision ;
- afficher le graphe ;
- comparer plusieurs résultats ;
- créer ou relancer des scénarios ;
- permettre à un agent LLM de relire le résultat.

## Interprétation correcte

Une prévision doit être lue avec trois questions :

- la tendance monte, baisse ou reste stable ?
- l'incertitude est-elle faible ou large ?
- quelles variables de contexte peuvent expliquer le mouvement ?

Une courbe seule ne suffit pas. Forecast devient utile quand la prévision est reliée à son contexte.
