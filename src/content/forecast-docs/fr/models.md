# Modèles

Un modèle est le moteur qui calcule la prévision. Forecast propose plusieurs familles locales et une famille cloud, puis vérifie leurs capacités, leur état et les ressources disponibles avant chaque exécution.

## Familles disponibles

| Famille | Éditeur | Usage principal |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Prévisions locales rapides et probabilistes |
| TimesFM | Google | Prévision généraliste de séries temporelles |
| Toto 2.0 | Datadog | Métriques et séries de monitoring |
| MOIRAI 2.0 | Salesforce | Multi-séries et variables de contexte |
| FlowState | IBM | Prévision locale probabiliste |
| TabPFN-TS | PriorLabs | Prévision locale expérimentale |
| TiRex | NX-AI | Prévision locale expérimentale |
| Kairos | Foundation Model Research | Prévision locale expérimentale |
| Sundial | THUML | Prévision locale probabiliste |
| TimeGPT | Nixtla | Prévision cloud via une clé API |

Les capacités exactes dépendent de la variante choisie. Le catalogue de l'application reste la source de vérité pour les fréquences, l'horizon, les covariables, le multi-séries et les intervalles supportés.

## Mode Manuel

En mode Manuel, vous choisissez le modèle dans le sélecteur. Forecast impose ce choix au LLM.

Le LLM vérifie tout de même que le modèle est prêt et qu'il accepte les données et le niveau de confiance exact. En cas d'incompatibilité, il doit demander de choisir un autre modèle ou un autre niveau de confiance. Il ne remplace pas silencieusement votre sélection.

## Mode Auto

En mode Auto, le LLM doit sélectionner un modèle parmi une courte liste déjà filtrée par Forecast.

Le backend exclut les modèles :

- non installés ou non préparés ;
- incompatibles avec la fréquence, l'horizon ou les séries ;
- incapables d'utiliser les covariables nécessaires ;
- incompatibles avec le niveau de confiance exact ;
- trop lourds pour les ressources disponibles ;
- cloud lorsque le cloud n'est pas autorisé.

Le LLM reçoit uniquement les informations matérielles utiles pendant cette opération Forecast. Ces informations ne sont pas ajoutées au reste de la conversation.

Avant tout backtest comparable, Auto indique seulement qu'un modèle est compatible ou recommandé selon ses capacités. Il ne le présente pas comme le meilleur.

## Installation et préparation

Depuis les réglages Forecast, le bouton Préparer télécharge le modèle, installe le moteur nécessaire et effectue une vérification réelle. Ces étapes ont lieu pendant la préparation, pas au lancement de la première prévision.

Plusieurs préparations peuvent être ajoutées à une file d'attente. Une famille peut partager son moteur entre plusieurs variantes, ce qui évite de réinstaller les mêmes dépendances.

Les états principaux sont :

| État | Signification |
| --- | --- |
| Non installé | Les fichiers du modèle ne sont pas présents |
| Mise à jour requise | Les fichiers existent, mais le moteur ou la validation doit être actualisé |
| Invalide | L'installation est incomplète ou ne passe pas les contrôles |
| Prêt | Le modèle et son moteur ont été vérifiés |
| Provider requis | Le modèle cloud attend une clé API configurée |

Un modèle local n'apparaît comme sélectionnable que lorsqu'il est prêt. Sa désinstallation retire ses fichiers et supprime le moteur partagé seulement lorsqu'aucun autre modèle de la famille n'en a besoin.

## Modèles cloud

Un modèle cloud envoie les données nécessaires au fournisseur configuré. Auto ne l'utilise que si le cloud a été autorisé, si le fournisseur est prêt et si la politique de données permet l'envoi externe.

Forecast ne bascule jamais silencieusement d'un modèle local vers le cloud.
