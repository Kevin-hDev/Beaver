# Modèles

Un modèle est le moteur qui calcule la prévision. Forecast propose plusieurs familles de modèles, locales (le calcul reste sur la machine) ou cloud (les données utiles sont envoyées au provider configuré).

## Familles locales

| Famille | Éditeur | Détail |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Modèle local rapide, bon pour un premier test ou une cible simple |
| TimesFM | Google | Modèle local de prévision de séries temporelles |
| Toto 2.0 | Datadog | Modèle local orienté monitoring et métriques |
| MOIRAI 2.0 | Salesforce | Modèle local, gère le multi-séries et les covariables |
| FlowState | IBM | Modèle local pour séries temporelles |
| TabPFN-TS | PriorLabs | Modèle local expérimental |
| TiRex | NX-AI | Modèle local expérimental |
| Kairos | Foundation Model Research | Modèle local expérimental |
| Sundial | THUML | Modèle local expérimental |

## Famille cloud

| Famille | Éditeur | Détail |
| --- | --- | --- |
| TimeGPT-2 / TimeGPT-2.1 | Nixtla | Moteur cloud spécialisé en séries temporelles. Nécessite une clé API et envoie les données utiles au provider. |

Les modèles cloud peuvent être plus puissants, mais impliquent une dépendance externe et un envoi de données. Pour des données sensibles, préférer un modèle local.

## Choisir un modèle

Le choix dépend surtout du dataset et du cas d'usage :

- **Test rapide, cible simple** : Chronos-Bolt.
- **Données sensibles, calcul local** : n'importe quelle famille locale.
- **Covariables et contexte futur** : un modèle qui gère les variables de contexte (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Multi-séries** : un modèle qui gère plusieurs séries (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Qualité cloud avancée** : TimeGPT, en acceptant l'envoi des données.

Un modèle avancé ne compense pas des données mal structurées. Avant de changer de modèle, vérifier la qualité du dataset, la fréquence, l'horizon et les variables de contexte.

## Installer un modèle local

Les modèles locaux doivent être installés depuis le gestionnaire de modèles (Settings → Forecast) ou via l'onglet modèles de l'espace Forecast. Ils sont téléchargés depuis Hugging Face ou GitHub selon la famille, puis stockés localement dans `~/.local/share/cl-go-dash/forecast-models/`.
