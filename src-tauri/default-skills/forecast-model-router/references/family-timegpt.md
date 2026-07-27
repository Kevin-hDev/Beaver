# Famille TimeGPT

## Positionnement

Utilise TimeGPT comme famille cloud gérée lorsque l'utilisateur autorise explicitement l'envoi des données et dispose d'un accès Nixtla valide.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | Nixtla |
| Exécution | cloud |
| Poids et ressources | non publiés ; ne les invente pas |
| Séries | série unique et panel |
| Covariables | passées et futures câblées |
| Multivarié joint | uniquement TimeGPT 2.1 avec séries alignées |
| Horizon catalogue | jusqu'à 5 000 points |
| Incertitude | confiance entière de 50 % à 99 % |
| Coût et quotas | vérifie le compte et le contrat courants |

## Applique la barrière cloud

Demande ou confirme le consentement cloud avant toute exécution. Vérifie la politique de données, la confidentialité, la région, le coût et les quotas. Ne bascule jamais depuis un modèle local sans autorisation.

## Route les variantes

- Utilise Mini comme candidat économique ou rapide si le compte l'expose.
- Traite Standard comme l'alias Beaver de l'API `timegpt-2`.
- Utilise Pro seulement si l'accès existe et si le backtest justifie son coût.
- Utilise 2.1 lorsque les dépendances conjointes entre séries alignées sont nécessaires.

Ne déduis aucune hiérarchie universelle de qualité à partir des noms Mini, Standard ou Pro.

## Vérifie le contrat API

Contrôle au moment de l'appel les modèles disponibles, l'horizon, les niveaux, les covariables, les quotas et les options autorisées. Traite toute réponse partielle, quota épuisé ou erreur réseau comme un échec fermé.

## Compare

Compare le résultat cloud aux meilleures options locales sur les mêmes plis. Intègre le coût, la latence, la confidentialité et la disponibilité au choix final.

## Sources vivantes

Vérifie la [documentation officielle TimeGPT](https://www.nixtla.io/docs) et les conditions du compte actif. Considère le registre Beaver et la réponse API comme plus frais que cette fiche.
