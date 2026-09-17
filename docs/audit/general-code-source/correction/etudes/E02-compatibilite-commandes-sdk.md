# E02 — Compatibilité des commandes internes et du SDK

Date du relevé : 17 septembre 2026
Commit examiné : `aa8e1658`

## Décision

Les commandes Tauri non documentées restent une interface interne entre React et Rust. Elles peuvent être retirées lorsqu'elles n'ont aucun appelant dans Beaver, à condition de garder le parcours qui les remplace et ses tests. Le SDK `@beaver/sdk`, son guide et `EXTENSIONS.md` constituent en revanche une surface publique : l'absence de consommateur dans le dépôt ne permet pas de la réduire.

`beaver.unstable.call(...)` reste donc publié. Le contrat actuel ne lui donne aucune méthode avancée, mais la documentation promet explicitement cette entrée pour de futures méthodes. Son retrait attendra une politique de version du SDK ou une nouvelle version majeure ; aucune dépréciation n'est utile tant que Beaver conserve cette promesse.

## Inventaire

| Surface | Statut prouvé | Usages connus | Décision |
|---|---|---|---|
| `install_git_extension`, `install_npm_extension`, `update_extension` | IPC Tauri interne, absent du SDK et des guides d'extension | Enregistrement Rust et tests d'inventaire seulement. React et les tests de bout en bout utilisent les travaux `start_extension_install`. | **Retrait direct autorisé.** Garder `add_local_extension`, les travaux d'installation et leurs reprises. |
| `list_agent_tool_catalog`, `set_agent_tool_enabled` | IPC Tauri interne, absent du SDK et des guides d'extension | Doublures d'anciens tests seulement. L'écran utilise les groupes. | **Retrait direct autorisé.** Garder `list_agent_tool_groups` et `set_agent_tool_group_enabled`. |
| `get_registry_model` dans `commands/registry.rs` | IPC Tauri interne, absent du SDK et des guides d'extension | Aucun appelant React, Rust, Node ou test utile. Les autres commandes de recherche et de familles restent utilisées. | **Retrait direct autorisé.** Ne pas confondre avec `get_registry_model_details`, utilisé par les profils Ollama. |
| `beaver.unstable.call(...)` | API publique dans `@beaver/sdk`, `sdk/README.md` et `EXTENSIONS.md` | Aucun appel possible aujourd'hui : `ADVANCED_HOST_TO_CORE_REQUEST_METHODS` est vide. La documentation décrit ce point comme réservé aux futures méthodes avancées. | **Conserver.** Aucun retrait ni dépréciation dans ce plan. |
| Autres méthodes, événements, UI et remplacements du SDK | API publique versionnée, annoncée dans le README racine et les changelogs publiés | Extensions intégrées, fixtures et extensions tierces possibles | **Conserver intégralement.** Une absence d'usage interne ne constitue pas une preuve de non-usage externe. |

## Contrat de retrait des commandes internes

- Rechercher les noms dans tout le dépôt hors documents d'audit, dépendances et artefacts de build.
- Retirer ensemble l'enregistrement Tauri, le wrapper, ses variantes d'erreur devenues inaccessibles et les assertions d'inventaire correspondantes.
- Garder les services partagés encore appelés par les travaux d'installation, les groupes d'outils et les autres commandes du registre.
- Vérifier les parcours de remplacement, puis les suites Rust et frontend complètes.
- Ne pas présenter les commandes Tauri internes comme une API d'extension : les contributions standard ne reçoivent pas Tauri `invoke`, et le contrat public passe par `BeaverExtensionApi`.

## Conséquences pour le plan

- Le nettoyage interne différé par M09 et M19 peut retirer les six commandes sans migration publique.
- M26 peut régénérer les types et artefacts du SDK, mais doit conserver `unstable.call` et le type vide tant que le contrat ne publie aucune méthode avancée.
- M32 doit préserver la surface SDK et ses chemins d'export pendant les déplacements.
- Les grandes capacités sans consommateur intégré restent hors périmètre de suppression.

## Validation de l'étude

La recherche d'appelants a été refaite dans Rust, React, Node, les scripts et les tests. La publication du SDK est confirmée par `src-tauri/resources/extension-host/sdk/package.json`, `index.d.ts`, les deux guides d'extension, le README racine et les changelogs. Les anciennes commandes d'installation sont remplacées depuis le 6 septembre 2026 par les travaux durables ; l'API d'extensions est présente dans des versions publiées depuis la série 1.1 et son expansion est annoncée dans le changelog courant.
