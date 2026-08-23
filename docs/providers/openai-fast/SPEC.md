# Phase 2 — Mode Fast OpenAI

Statut : implémentée, validation provider partielle

Date de recherche : 22 août 2026, sources revérifiées le 23 août 2026

Périmètre : OpenAI par clé API et OpenAI/Codex par OAuth

## 1. Objectif

Ajouter un réglage **Rapide** aux modèles OpenAI qui prennent réellement en charge Fast, sans le confondre avec un niveau de raisonnement.

La préférence appartient à chaque session : une session peut activer Fast sans modifier les autres. Elle est sauvegardée avec la session, restaurée après fermeture et réouverture de Beaver, et reste active jusqu'à ce que l'utilisateur la désactive dans cette même session.

Le réglage doit fonctionner sur les deux transports OpenAI :

- clé API OpenAI ;
- connexion ChatGPT/Codex OAuth.

Cette phase n'ajoute aucun modèle, aucun prix au sélecteur et aucun multiplicateur de vitesse à l'interface.

## 2. Sources officielles et faits vérifiés

### 2.1 API OpenAI

Source principale : [Fast mode](https://developers.openai.com/api/docs/guides/fast-mode).

- Le nom public est **Fast** depuis le 30 juillet 2026. L'ancienne valeur `priority` reste acceptée par l'API.
- Fast est disponible sur Responses et Chat Completions.
- La requête recommandée utilise `service_tier: "fast"`.
- Pour désactiver Fast explicitement, la requête doit utiliser `service_tier: "default"`.
- Omettre `service_tier` signifie `auto`. Ce n'est pas un état désactivé fiable, car le projet OpenAI peut être configuré pour utiliser Fast par défaut.
- La réponse expose le tier réellement servi. Pour GPT-5.6 et les modèles antérieurs, OpenAI peut encore retourner `priority` même si la requête utilise `fast`.
- Fast conserve les limites de débit de base du modèle Standard.
- OpenAI peut rétrograder temporairement une requête vers `default` en cas de montée rapide du trafic. Cette rétrogradation n'annule pas la préférence de l'utilisateur.
- Les outils existants, le streaming, les entrées image et le long contexte GPT-5.6 restent utilisables.
- Fast ne couvre pas les modèles fine-tunés ni les embeddings.
- Les remises de cache d'entrée continuent de s'appliquer.
- Les règles existantes de résidence des données, Zero Data Retention, BAA, outils et éligibilité restent applicables.
- Fast est facturé séparément de Scale Tier.

La [référence Responses](https://developers.openai.com/api/reference/cli/resources/responses/methods/create) expose aussi `ultrafast`. Ce tier est distinct, soumis à un accès particulier, et ne doit jamais être produit par la bascule Rapide.

Décision de transport Beaver, validée le 23 août 2026 : OpenAI par clé API utilise Responses pour toutes les générations, Fast ou Standard. Chat Completions acceptait Fast sans raisonnement, mais rejetait `reasoning_effort` avec HTTP 400 sur GPT-5.5 et GPT-5.6; le même compte, le même modèle et `reasoning: { effort: "medium" }` ont réussi via Responses. Un transport unique évite qu'activer le raisonnement change silencieusement de protocole ou perde le choix de l'utilisateur.

La [grille tarifaire Fast](https://developers.openai.com/api/docs/pricing?latest-pricing=fast), ouverte sous l'onglet Fast le 23 août 2026, confirme dans le tableau principal `gpt-5.6-sol`, `gpt-5.6-terra` et `gpt-5.6-luna`; elle affiche aussi séparément certains modèles spécialisés. Elle ne confirme pas actuellement GPT-5.5 ni GPT-5.4 pour l'API par clé. Cette absence de preuve API ne signifie pas que ces modèles sont incompatibles dans Codex OAuth, où ils sont documentés et annoncés par le catalogue du compte. Cette SPEC n'ajoute pas de modèle absent du registre Beaver.

### 2.2 OAuth ChatGPT/Codex

Sources : [Codex — Speed](https://learn.chatgpt.com/docs/agent-configuration/speed) et [Codex config reference](https://learn.chatgpt.com/docs/config-file/config-reference).

- Fast est disponible avec une connexion ChatGPT dans Codex Desktop, CLI et IDE.
- Les modèles documentés sont GPT-5.6, GPT-5.5 et GPT-5.4.
- Le catalogue Codex annonce la capacité par modèle avec ses tiers de service.
- Dans la configuration Codex, `service_tier = "fast"` correspond sur le fil à la valeur `priority`.
- La référence de configuration décrit `service_tier` comme une chaîne choisie parmi les tiers annoncés par le modèle; `fast` est traduit vers `priority`. Elle ne définit pas une liste fermée limitée à `default`, `priority` et `flex`.
- Dans le client Codex officiel, `default` est un sentinelle de configuration interne. `service_tier_for_request` le filtre avant la sérialisation : Fast envoie `priority`, tandis que Fast coupé omet le champ.
- Le client Codex officiel ajoute aussi `x-codex-routing-hint`: `model=<slug>;tier=priority` pour Fast et `model=<slug>` quand aucun tier n'est envoyé. Cet en-tête est posé sur HTTP et WebSocket. Beaver doit reproduire ce routage à partir de la même capture que le corps, puis le vérifier par appel réel.
- `features.fast_mode` permet l'exposition du choix quand le modèle le déclare.
- Fast consomme davantage de crédits ChatGPT. Beaver ne doit afficher ni ratio ni estimation inventée.
- Spark est un modèle séparé, pas le mode Fast.

Preuves du client `openai/codex` branche `main`, relues le 23 août 2026 :

- `codex-rs/protocol/src/config_types.rs` : sentinelle `SERVICE_TIER_DEFAULT_REQUEST_VALUE` et traduction Fast → `priority` ;
- `codex-rs/protocol/src/openai_models.rs` : `service_tier_for_request` retire `default` et les tiers absents de `service_tiers` ;
- `codex-rs/core/src/client.rs` : construction et pose de `x-codex-routing-hint` sur HTTP et WebSocket ;
- `codex-rs/codex-api/src/common.rs` : omission serde de `service_tier` quand sa valeur est absente.

### 2.3 Catalogue local observé

Le cache Codex local observé le 22 août 2026 annonçait le tier Fast pour :

- `gpt-5.6-sol` ;
- `gpt-5.6-terra` ;
- `gpt-5.6-luna` ;
- `gpt-5.5` ;
- `gpt-5.4`.

Il ne l'annonçait pas pour `gpt-5.4-mini` ni `gpt-5.3-codex-spark`.

Cette observation confirme la forme du catalogue, mais ne remplace pas un test réel du compte OAuth Beaver. Le catalogue dynamique du compte reste l'autorité.

### 2.4 Icône et licence

L'asset fourni est `/Users/kevinh/Downloads/typcn--flash-outline.svg`.

- Icône : `typcn:flash-outline` de Stephen Hutchings, distribuée via Iconify.
- Licence : Creative Commons Attribution-ShareAlike 4.0 International.
- La capture de licence fournie est `/Users/kevinh/Downloads/typcn--flash-outline.png`.
- Source publique : [Typicons Flash Outline](https://icon-sets.iconify.design/typcn/flash-outline/).

Décision d'intégration : le dessin est converti en `FastModeIcon` au moyen de la primitive existante `InlineIcon`, avec `viewBox`, tracé et `currentColor` inchangés. La raison est de conserver une seule primitive pour les dessins qui suivent la couleur du texte et du thème; un asset à masque créerait une seconde mécanique. La conversion et l'absence de modification du tracé sont inscrites dans `THIRD_PARTY_NOTICES.md`. Le SVG et la capture PNG ne sont pas embarqués comme assets séparés.

## 3. Vocabulaire et invariants

### 3.1 Trois notions séparées

- `fast_mode_enabled` : préférence durable de la session.
- `supports_fast_mode` : capacité du couple provider/modèle.
- `fast_effective` : valeur utilisée pour une génération, calculée par `fast_mode_enabled && supports_fast_mode`.

Ces notions ne doivent jamais être fusionnées. La préférence ne prouve pas la capacité et la capacité n'active pas automatiquement la préférence.

### 3.2 Invariants obligatoires

1. La valeur par défaut est `false`.
2. Chaque session possède sa propre valeur.
3. La valeur est sauvegardée dans le fichier canonique de la session.
4. Une réouverture de Beaver restaure exactement cette valeur.
5. Un changement de modèle ou de provider ne modifie jamais la préférence.
6. Sur un modèle non déclaré compatible, la ligne Rapide est masquée et la préférence est conservée. OpenAI par clé envoie tout de même `default` pour neutraliser le réglage du projet; Codex OAuth et les autres providers n'envoient aucun tier.
7. Si la session revient ensuite sur un modèle compatible, Rapide redevient actif sans nouvelle action.
8. Une nouvelle session, un clone, un sous-agent, un heartbeat ou une session gateway commence à `false`.
9. La préférence ne se propage jamais d'une session parente vers une autre session.
10. Le réglage Fast est indépendant du niveau de raisonnement.
11. Le tier effectif d'une génération est figé au démarrage de cette génération.
12. Modifier la bascule pendant un flux en cours ne change que la génération suivante.
13. Pour toute requête OpenAI par clé où Fast n'est pas effectivement actif, aucune configuration de projet distante ne doit pouvoir réactiver Fast : Beaver envoie `default`, même si le modèle n'est pas déclaré compatible dans son registre.

## 4. Autorités de données

### 4.1 Préférence de session

L'unique autorité est le champ Rust suivant dans `AgentSession` :

```rust
fast_mode_enabled: bool
```

Exigences :

- `#[serde(default)]` assure la compatibilité des anciennes sessions ;
- la valeur est incluse dans la sérialisation, y compris quand elle vaut `false` ;
- `AgentSessionMeta` transporte la valeur pour les listes, mais elle est toujours dérivée de `AgentSession` ;
- aucune copie dans `localStorage`, `agent-settings.json` ou un nouveau fichier ;
- la mise à jour passe par une commande Rust étroite, verrouille la session, écrit atomiquement, puis renvoie la valeur confirmée ;
- en cas d'échec d'écriture, l'état visible revient à la valeur persistée et affiche une erreur i18n générique ;
- la commande générale de sauvegarde d'une session ne doit jamais écraser la valeur Rust avec un objet frontend périmé.

### 4.2 Capacité du modèle

L'unique autorité normalisée est :

```rust
supports_fast_mode: bool
```

dans `ModelInfo`. Les DTO frontend ne font que transporter cette valeur. Ils ne recalculent pas la capacité à partir du nom du modèle.

Pour OpenAI par clé API :

- la capacité est déclarée explicitement dans le registre des modèles ;
- les identifiants GPT-5.6 confirmés par la grille Fast, y compris leur alias enregistré, peuvent être marqués compatibles ;
- GPT-5.5 et GPT-5.4 ne sont pas déclarés incompatibles globalement : leur support OAuth est confirmé, mais leur capacité Fast API reste non confirmée tant qu'une source API officielle ou une preuve réelle datée ne l'établit pas ;
- aucune règle par préfixe telle que `starts_with("gpt-5")` ;
- ajouter un futur modèle exige une entrée explicite ou une source dynamique officielle.

Pour OAuth :

- le catalogue dynamique du compte est l'autorité ;
- un modèle est compatible seulement si `service_tiers[].id` contient `priority` ;
- `additional_speed_tiers` est obsolète et seulement informatif : il ne peut pas autoriser un tier que le chemin réseau Codex filtrerait ;
- le parseur reste borné, dédupliqué et validé comme en phase 1 ;
- les modèles de secours ont `supports_fast_mode: false`, car ils ne prouvent pas l'éligibilité du compte ;
- un catalogue invalide échoue fermé pour la capacité Fast.

Pour tous les autres providers : `supports_fast_mode` vaut `false`.

## 5. Cycle de vie de la session

### 5.1 Création

Le sélecteur de l'écran d'accueil peut conserver un brouillon local initialisé à `false`. Lors de la création, ce brouillon est transmis atomiquement à la nouvelle session. Après création, le prochain brouillon revient à `false`.

Toute autre voie de création initialise explicitement `fast_mode_enabled` à `false` :

- nouvelle conversation ;
- duplication/clone ;
- sous-agent ;
- heartbeat et réveil planifié ;
- gateway ;
- migration d'une ancienne session.

### 5.2 Modification

Le clic sur la bascule :

1. désactive temporairement le contrôle ;
2. appelle la commande Rust avec l'identifiant de session et la nouvelle valeur ;
3. attend la persistance atomique ;
4. affiche la valeur confirmée ;
5. en cas d'erreur, conserve ou restaure l'ancienne valeur.

Une collection ou file d'attente éventuelle de mutations doit rester bornée. La solution préférée est une seule mutation en vol par session. Tous les chemins qui font lecture → mutation → sauvegarde, y compris le renommage, le modèle, le raisonnement et la sauvegarde générale, passent par le même guichet verrouillé; une adoption partielle permettrait à un writer ancien d'effacer la préférence.

### 5.3 Rechargement

Au lancement et lors d'un rechargement de la liste, la valeur vient du fichier `agent-sessions/<id>.json`, via le backend. Le frontend ne doit pas reconstruire une valeur par défaut si le champ est présent.

La reconstruction de l'index doit conserver `true` comme `false`. Un test doit traverser la sérialisation réelle, la reconstruction de l'index et la réponse IPC.

### 5.4 Changement de modèle

La préférence n'est jamais normalisée à `false` quand le modèle devient incompatible.

Exemple normatif :

1. session A sur GPT-5.6, Fast activé ;
2. passage sur un modèle local, ligne masquée, préférence toujours vraie ;
3. fermeture et réouverture de Beaver ;
4. retour sur GPT-5.6 ;
5. ligne visible et bascule active.

## 6. Interface utilisateur

### 6.1 Emplacement et contenu

Pour un modèle compatible, la ligne Fast apparaît en tête de la liste des modes de raisonnement :

```text
[éclair] Rapide                                      [toggle]
```

Contraintes :

- icône à gauche ;
- libellé juste à droite ;
- composant `ToggleSwitch` existant complètement à droite ;
- aucune vitesse, aucun multiplicateur, aucun prix, aucun crédit, aucun sous-titre ;
- la ligne est absente pour un modèle incompatible ;
- la liste reste ouverte après activation pour que la bascule soit immédiatement vérifiable ;
- le choix d'un niveau de raisonnement n'altère pas Fast et inversement.

### 6.2 États

- désactivé par défaut ;
- activé ;
- mutation en cours : contrôle désactivé et état stable ;
- erreur : retour à l'état confirmé et message traduit générique ;
- génération en cours : la bascule peut être modifiée, mais l'effet commence à la génération suivante.

### 6.3 Accessibilité et thèmes

- conserver le rôle `switch` et l'état `aria-checked` du composant existant ;
- le nom accessible est la traduction de Rapide ;
- navigation et activation au clavier ;
- focus visible ;
- icône décorative masquée aux lecteurs d'écran ;
- `currentColor` et tokens de thème uniquement ;
- vérification manuelle en thèmes clair et sombre.

### 6.4 Traductions

La clé i18n doit exister dans les sept langues :

| Langue | Libellé |
| --- | --- |
| français | Rapide |
| anglais | Fast |
| espagnol | Rápido |
| allemand | Schnell |
| italien | Rapido |
| chinois | 快速 |
| japonais | 高速 |

Les messages d'échec de sauvegarde et d'indisponibilité du tier doivent aussi être traduits dans les sept langues.

## 7. Contrat réseau

### 7.1 Capture au début d'une génération

Au début d'une génération, le backend calcule une valeur immutable :

```text
fast_effective = session.fast_mode_enabled && model.supports_fast_mode
```

La même valeur accompagne :

- la requête initiale ;
- les continuations après outils ;
- les retries autorisés ;
- la compression automatique nécessaire à cette génération ;
- les chemins HTTP et WebSocket du transport Codex.

Une bascule modifiée en cours de flux ne peut donc pas produire une génération mélangeant Standard et Fast.

Les titres, résumés, diagnostics et autres appels internes qui ne sont pas une génération de la session ne doivent pas hériter implicitement de cette valeur.

### 7.2 OpenAI par clé API

Pour un modèle compatible :

| Préférence | Champ envoyé |
| --- | --- |
| activée | `"service_tier": "fast"` |
| désactivée | `"service_tier": "default"` |

`default` est obligatoire dès que Fast n'est pas effectivement actif afin de neutraliser un éventuel défaut Fast défini sur le projet OpenAI. Cela inclut un modèle que Beaver ne déclare pas compatible : la ligne reste masquée, mais la requête OpenAI par clé porte `default`.

Pour un autre provider, le champ est absent.

Beaver applique cette règle dans son payload Responses unique. Le raisonnement y est envoyé sous la forme imbriquée `reasoning.effort`; le champ Chat Completions `reasoning_effort` n'est jamais émis par ce chemin.

### 7.3 OpenAI/Codex OAuth

Pour un modèle dont le catalogue dynamique annonce Fast :

| Préférence | Champ envoyé sur le fil Codex |
| --- | --- |
| activée | `"service_tier": "priority"` |
| désactivée | champ absent |

Cette correspondance suit le client Codex officiel : `default` est un sentinelle de configuration filtré avant sérialisation, pas une valeur envoyée au backend ChatGPT.

Le champ doit vivre dans la structure canonique `CodexRequest`, partagée par les transports HTTP et WebSocket. Il ne doit pas être injecté séparément dans chaque transport.

Le même objet canonique produit l'en-tête non sensible `x-codex-routing-hint` :

- Fast : `model=<slug>;tier=priority` ;
- Standard ou modèle non compatible : `model=<slug>`.

La valeur est validée et dérivée du modèle et du tier déjà présents dans `CodexRequest`; HTTP et WebSocket ne la reconstruisent pas chacun de leur côté.

Pour un modèle non compatible, le champ est absent.

### 7.4 Valeurs interdites

La bascule Beaver ne produit jamais :

- `auto` ;
- `flex` ;
- `ultrafast` ;
- une valeur libre provenant du frontend.

Le frontend envoie uniquement un booléen. Le backend choisit la valeur réseau selon le provider et la capacité.

## 8. Réponse, diagnostic et observabilité

Beaver doit distinguer le tier demandé du tier réellement servi.

### 8.1 Extraction

- Chat Completions : lire le `service_tier` au niveau supérieur des chunks ou de la réponse finale.
- Responses HTTP/WebSocket : lire le `service_tier` de l'objet `response` final ou de l'événement final correspondant.
- `fast` et `priority` signifient que Fast a été servi.
- `default` signifie qu'OpenAI a servi Standard, y compris lors d'une rétrogradation légitime.
- champ absent ou inconnu : résultat indéterminé, sans inventer de succès.

### 8.2 Persistance du diagnostic

Le journal de diagnostic borné peut enregistrer :

- `fast_requested: bool` ;
- `service_tier_served: fast | default | unknown`.

Il ne doit jamais enregistrer de token OAuth, clé API, corps brut, en-tête sensible, URL privée ou identifiant de compte.

Une rétrogradation vers `default` ne désactive pas la préférence de la session et ne déclenche pas de notification répétitive. L'information reste disponible dans le diagnostic.

## 9. Erreurs et retries

### 9.1 Tier refusé

Si le provider refuse explicitement le paramètre `service_tier`, Beaver renvoie le code stable `service_tier_unavailable` avec un message utilisateur générique et traduit. Le champ structuré `param == "service_tier"` est la preuve principale. Un code tel que `unsupported_service_tier` reste une hypothèse de compatibilité tant qu'il n'a pas été observé dans une fixture réelle ou documenté officiellement; aucun texte libre du provider ne sert à classifier l'erreur.

Le backend ne doit pas réessayer silencieusement la même génération en Standard : cela pourrait doubler une requête facturée ou des effets d'outils.

### 9.2 Règles existantes préservées

- un `401` OAuth peut suivre l'unique refresh déjà autorisé ;
- un `429` suit la politique bornée existante et ne doit pas devenir un retry illimité sous prétexte de Fast ;
- une réponse partiellement diffusée ne doit jamais être rejouée silencieusement ;
- les corps provider restent filtrés par le mécanisme de journalisation existant ;
- toute erreur de lecture de capacité échoue fermé pour Fast.

## 10. Coûts, crédits et limites

- Aucun prix n'est affiché dans le sélecteur de modèle ou la ligne Rapide.
- Aucun ratio de vitesse ou de consommation n'est affiché.
- Avec une clé API, Fast suit la tarification API Fast, pas les multiplicateurs de crédits ChatGPT.
- Avec OAuth, Fast consomme les crédits ChatGPT selon les règles du compte.
- Beaver ne doit pas estimer un coût GPT-5.6 sans connaître le tier réellement servi et la tranche de contexte pertinente.
- Une réponse rétrogradée vers `default` ne doit pas être comptabilisée comme Fast par déduction.
- Les limites de débit, l'éligibilité et les éventuelles rétrogradations restent contrôlées par OpenAI.

## 11. Sécurité et confidentialité

- La capacité et le tier sont validés côté Rust ; le frontend ne choisit jamais une chaîne réseau.
- Les tokens OAuth continuent d'être envoyés uniquement vers le transport Codex autorisé ; Fast ne change aucune origine réseau.
- Les clés API restent dans le backend et sont zéroïsées selon le mécanisme existant.
- Aucun secret ou corps brut n'est ajouté aux logs.
- Les collections alimentées par le catalogue et les diagnostics restent bornées.
- Une erreur de catalogue, de persistance ou de sérialisation bloque l'activation effective au lieu de l'autoriser par défaut.
- Fast ne modifie ni les politiques de conservation des données ni l'usage des outils.

## 12. Fichiers et responsabilités prévues

Les chemins exacts peuvent évoluer si le découpage à 230 lignes l'exige, mais les responsabilités restent les suivantes :

### Backend

- `src-tauri/src/services/agent_local/types_session.rs` : préférence durable.
- `src-tauri/src/services/agent_local/session_index.rs` : projection fidèle dans les métadonnées.
- `src-tauri/src/services/agent_local/session_store_updates.rs` et commande IPC dédiée : mutation atomique.
- voies de création/clone/sous-agent/heartbeat/gateway : valeur initiale `false`.
- `src-tauri/src/services/llm/types.rs` : capacité normalisée du modèle.
- `src-tauri/resources/provider-models/openai.json` : capacité explicite de l'API.
- catalogue Codex : lecture bornée des tiers dynamiques.
- payload OpenAI générique : `fast` ou `default`.
- `src-tauri/src/services/codex_client/types.rs` : champ canonique OAuth partagé HTTP/WS.
- module Codex dédié : construction validée de `x-codex-routing-hint` à partir de `CodexRequest`.
- parseurs de flux : tier réellement servi et code d'erreur stable.

### Frontend

- `src/types/agent-session.ts` : transport de la préférence.
- DTO modèles : transport de `supports_fast_mode` sans calcul local.
- `src/components/agent-local/reasoning-selector.tsx` : ligne Rapide en tête.
- `src/components/ui/fast-mode-icon.tsx` : tracé tiers unique rendu par `InlineIcon`.
- `ToggleSwitch` existant : contrôle de la valeur.
- sept fichiers i18n : libellés et erreurs.

### Licence

- composant `FastModeIcon` dans le dossier d'icônes existant et test d'autorité étendu, sans nouvel asset ni seconde primitive ;
- `THIRD_PARTY_NOTICES.md` comme unique notice de distribution.

## 13. Tests automatisés obligatoires

### 13.1 Session et persistance

- ancienne session sans champ → `false` ;
- sérialisation de `true` et de `false` ;
- aller-retour fichier → index → IPC ;
- sessions A et B indépendantes ;
- fermeture/rechargement conserve la valeur ;
- nouvelle session, clone, sous-agent, heartbeat et gateway → `false` ;
- changement vers un modèle incompatible conserve la préférence ;
- retour sur un modèle compatible réactive l'état effectif ;
- sauvegarde générale avec objet frontend périmé ne l'écrase pas ;
- erreur d'écriture atomique conserve l'ancienne valeur ;
- changement pendant un flux ne s'applique qu'à la génération suivante.

### 13.2 Capacités

- registre API : chaque identifiant exact attendu et son alias ;
- absence de détection par préfixe ;
- catalogue OAuth : Fast annoncé, absent, inconnu et mal formé ;
- `additional_speed_tiers` seul ne publie pas la capacité ;
- catalogue borné et dédupliqué ;
- fallback OAuth toujours non compatible ;
- autres providers toujours non compatibles ;
- sérialisation réelle de `supports_fast_mode` jusqu'au frontend.

### 13.3 Payloads

- API compatible activé → `fast` ;
- API compatible désactivé → `default` ;
- API non déclarée compatible → `default` et ligne masquée ;
- OAuth compatible activé → `priority` ;
- OAuth compatible désactivé → champ absent ;
- OAuth non compatible/autre provider → champ absent ;
- HTTP et WebSocket Codex utilisent la même structure ;
- HTTP et WebSocket Codex envoient le même `x-codex-routing-hint` dérivé du corps ;
- continuations outils, retry et compression conservent la capture initiale ;
- aucune branche ne peut produire `auto`, `flex` ou `ultrafast`.

### 13.4 Réponses et erreurs

- parse `fast`, `priority`, `default`, absent et inconnu ;
- une rétrogradation conserve la préférence ;
- refus du tier → `service_tier_unavailable` ;
- aucun replay Standard automatique ;
- aucune fuite de corps ou de secret dans l'erreur et le diagnostic ;
- journal borné.

### 13.5 Frontend

- ligne en tête uniquement pour un modèle compatible ;
- icône, libellé et bascule, sans autre contenu ;
- état indépendant entre deux sessions ;
- restauration après rechargement ;
- passage incompatible puis retour compatible ;
- raisonnement et Fast indépendants ;
- mutation en cours et échec de persistance ;
- clavier, rôle switch, nom accessible et focus ;
- présence des sept traductions ;
- thème clair et sombre sans couleur codée en dur.

## 14. Mutations minimales de revue

Les tests doivent rougir si l'on :

1. retire `#[serde(default)]` ou la sérialisation de la préférence ;
2. partage une valeur Fast entre deux sessions ;
3. fait hériter un clone ou sous-agent ;
4. remet la préférence à `false` lors d'un changement de modèle ;
5. omet `default` sur une requête OpenAI API où Fast n'est pas actif, ou émet `default` sur le fil OAuth ;
6. retire le branchement du calcul effectif au payload ;
7. retire le branchement vers le transport HTTP ou WebSocket ;
8. remplace la capture de génération par une lecture mutable ;
9. accepte un modèle OAuth qui n'annonce pas Fast ;
10. active Fast sur un fallback OAuth ;
11. remplace les identifiants API exacts par un préfixe ;
12. transforme une rétrogradation `default` en désactivation de la préférence ;
13. ajoute un replay Standard après refus du tier ;
14. masque l'échec de persistance par un état optimiste ;
15. retire une traduction ou le nom accessible ;
16. ajoute un multiplicateur, un prix ou `ultrafast` à la bascule ;
17. retire le guichet verrouillé d'un writer de session, notamment `rename` ;
18. retire `x-codex-routing-hint` d'HTTP ou WebSocket, ou fait diverger son tier du corps.

## 15. Validation fonctionnelle avant activation

Les tests unitaires ne suffisent pas. L'activation fonctionnelle exige deux campagnes séparées, anonymisées et datées :

### 15.1 Clé API

- compte réellement éligible ;
- modèle compatible exact ;
- Fast activé puis désactivé ;
- tier demandé et tier retourné ;
- streaming ;
- appel outil et continuation ;
- second tour ;
- entrée image ;
- refus de tier et 429 sans double facturation ;
- redémarrage de Beaver et restauration de session.

### 15.2 OAuth

- catalogue réel du compte et métadonnées de tier ;
- Fast activé puis désactivé ;
- transport HTTP et, s'il est sélectionné, WebSocket ;
- tier demandé et tier servi avec l'en-tête `x-codex-routing-hint` officiel ; dans un worktree jetable, comparaison contrôlée sans cet en-tête afin de déterminer son rôle réel ;
- streaming ;
- appel outil et continuation ;
- second tour ;
- entrée image ;
- refresh 401 unique ;
- quota 429 sans boucle ;
- redémarrage de Beaver et restauration de session.

Chaque preuve suit la matrice provider P01–P13 de `docs/providers/plan-de-tests.md`. Les fixtures restent bornées, anonymisées et nommées `provider-modele-region-date`. Aucun support ne doit être déclaré fonctionnel sur la seule foi de la documentation.

### 15.3 Vérification visuelle

- application réellement ouverte ;
- modèles compatibles et incompatibles ;
- deux sessions côte à côte avec des états différents ;
- redémarrage réel ;
- thèmes clair et sombre ;
- clavier et VoiceOver ;
- aucune vitesse, aucun prix et aucun crédit visibles.

Les appels réels potentiellement facturés nécessitent l'accord explicite de l'utilisateur au moment de la validation.

## 16. Critères de sortie de la phase 2

La phase est terminée seulement si :

1. toutes les autorités décrites ci-dessus sont uniques ;
2. les tests Rust, TypeScript, frontend, lint, format et contrats sont verts ;
3. les mutations critiques rougissent ;
4. `graphify update .` a été exécuté après les changements de code ;
5. l'icône via `InlineIcon`, son autorité unique et sa notice sont présentes ;
6. les deux thèmes et la persistance après redémarrage ont été vus fonctionner ;
7. les campagnes API et OAuth sont consignées séparément, ou explicitement déclarées non exécutées ;
8. aucun résultat de coût, de vitesse ou de tier servi n'est inventé.

## 17. Hors périmètre

- ajout de nouveaux modèles OpenAI ;
- exposition de `ultrafast`, Flex ou Scale Tier ;
- réglage Fast global à toutes les sessions ;
- héritage par les clones ou sous-agents ;
- affichage de prix, crédits ou multiplicateurs ;
- modification des règles de raisonnement ;
- contournement des limites, de l'éligibilité ou des politiques OpenAI.
