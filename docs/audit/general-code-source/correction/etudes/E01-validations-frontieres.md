# E01 — Validations aux frontières Rust, React et Node

Date du relevé : 17 septembre 2026
Commit examiné : `afa3df9f`

## Décision

Une validation reste nécessaire à chaque entrée contrôlable par un utilisateur, un fichier persistant ou une extension tierce. Une sortie que Rust vient de construire et de sérialiser vers React doit en revanche être décrite par un type généré, sans recopier tout son validateur en TypeScript. Les contrôles de routage d'événements et les limites d'erreur d'affichage restent utiles : ils ne valident pas le même contrat.

La génération d'un type ne retire aucun contrôle de portée, permission, quota, révocation, interception ou taille sur une entrée tierce.

## Inventaire et arbitrage

| Domaine et contrôle actuel | Origine réelle de la donnée | Décision | Raison et preuve à conserver |
|---|---|---|---|
| Terminal : `tab_store::validate_document` à la lecture du disque | Document persistant modifiable hors de Beaver | **Conservé** | Le fichier peut être ancien, corrompu ou remplacé. Rust garde lecture bornée, migration et récupération. Tester fichier malformé, surdimensionné et ancien format. |
| Terminal : `tab_store::validate_document` à `save_terminal_tabs` | Objet reçu de React | **Conservé** | React est une frontière d'entrée. Rust doit refuser version, groupes, labels et volumes invalides avant écriture. Tester chaque limite et un caractère de contrôle. |
| Terminal : `isTerminalTabsDocument` après `load_terminal_tabs` | Objet que Rust vient de valider puis sérialiser | **Supprimable après M26** | Le contrôle recopie exactement la forme et cinq limites Rust. Générer le type et les constantes ; un test de contrat remplace le second parseur. |
| Terminal : `normalizeTerminalLabel` avant envoi | Texte saisi ou calculé dans l'interface | **Conservé pour l'expérience** | Il donne un retour immédiat et normalise le texte. Il ne remplace jamais le refus Rust. Ses plafonds doivent être générés par M26. |
| Terminal : `MAX_LIVE_TERMINALS` utilisé pour désactiver l'action | État d'interface | **Conservé, valeur générée** | Il évite une action vouée au refus ; l'admission Rust reste l'autorité de sécurité. |
| Navigateur : `validate_browser_url`, identifiants, session et scan local en Rust | Saisie utilisateur, disque et moteur CEF | **Conservé** | Ce sont les frontières d'entrée et la source native. Tester URL avec identifiants, contrôle, dépassement, doublon d'onglet et faux site local. |
| Navigateur : `parseBrowserSession`, `parseTabCreation`, `parseLocalSiteScan` | Commandes Tauri construites par Rust | **Supprimable après M26** | Types, titres, URL, nombres et limites sont déjà validés côté Rust. Générer types et constantes, puis typer les retours `invoke`. |
| Navigateur : parseurs d'enveloppes d'événements | Bus Tauri global | **Partiellement conservés** | Garder `eventVersion` et `conversationId`, qui routent l'événement vers la bonne vue. La revalidation complète de `session`, tab, URL et génération devient inutile quand le payload est généré. |
| Navigateur : `normalizeBrowserUrl` dans la barre d'adresse | Saisie utilisateur | **Conservé** | C'est une validation d'entrée et une normalisation visible ; Rust revalide ensuite. |
| Navigateur : signature PNG dans `parseFaviconSnapshot` | Événement Rust après validation CEF | **Supprimable après type généré** | Rust borne et valide le PNG. React garde version, conversation ciblée et limite d'erreur du rendu. Tester qu'un callback CEF malformé est refusé par Rust. |
| Extension : validation dans l'hôte Node | Valeurs produites par du code d'extension tiers | **Conservée** | Elle donne une erreur locale bornée à l'auteur et empêche l'hôte de collecter une valeur illimitée. Elle n'est pas une autorité de sécurité. |
| Extension : revalidation Rust du protocole, contributions, vues, actions et résultats | Processus Node non fiable | **Conservée** | Rust est la frontière de sécurité. Conserver portée, identité, permission, quotas, révocation, interception, tailles et séquences. Tester payload tiers malformé malgré un hôte simulé. |
| Extension : parseurs React de records et vues standard reçus de Rust | Projection Rust de données tierces déjà validées | **Générables ; parseurs structurels supprimables après M26** | Le type doit venir du contrat commun. Garder les limites d'erreur par contribution et les règles propres au rendu, car elles isolent un composant défaillant. |
| Extension : actions, champs et événements envoyés depuis React | Interaction utilisateur vers Rust/Node | **Conservés aux frontières Rust et Node** | L'interface peut vérifier pour l'expérience, mais les deux processus receveurs gardent leurs bornes et leur contrôle de propriétaire. |
| Extension : `contract.mjs` et constantes Node/Rust | Contrat embarqué livré avec Beaver | **Générés** | `resources/extension-host/contract.json` fait foi. Le module Node et les artefacts Rust/TypeScript/SDK doivent être reproduits par le générateur et vérifiés par `contracts:check`. |

## Tests requis lors de M26

| Scénario malformé | Résultat attendu |
|---|---|
| Document terminal disque surdimensionné ou ancien | Rust récupère ou migre sans faire confiance à React. |
| Objet terminal invalide envoyé par React | Rust refuse sans écrire. |
| Faux retour `load_terminal_tabs` dans un test frontend | Le test de doublure peut échouer au niveau du contrat de test ; aucun parseur de production dupliqué n'est requis. |
| URL ou identifiant navigateur invalide à l'entrée | Rust refuse avant création ou navigation. |
| Événement navigateur d'une autre conversation ou mauvaise version | React l'ignore. |
| Callback CEF contenant favicon ou état invalide | Rust le refuse avant publication. |
| Extension tierce contourne la validation Node | Rust refuse le message, ne produit aucun effet et garde une erreur générique. |
| Vue d'extension valide mais composant React fautif | La limite d'erreur isole la contribution sans casser Beaver. |
| Artefact généré modifié à la main | `npm run contracts:check` échoue. |

## Conséquences pour le plan

- M26 peut générer les types et constantes du terminal et du navigateur, puis retirer uniquement les parseurs de sorties internes recensés ci-dessus.
- M26 peut étendre les artefacts d'extensions, mais ne retire aucune validation Node ou Rust portant sur une donnée tierce.
- Les contrôles de routage React (`conversationId`, version d'événement), les limites d'erreur et les validations de saisie restent distincts du contrat généré.
- Aucun autre domaine ne doit appliquer une règle générale du type « Rust a validé, donc toute validation disparaît ».

## Validation de l'étude

Les producteurs et consommateurs ont été relus dans `terminal/tab_store.rs`, `terminal-persistence.ts`, `browser/session_types.rs`, `browser-types.ts`, `browser-events.ts`, `browser-favicon-events.ts`, `resources/extension-host/contract.mjs`, les validateurs Node, les validateurs Rust d'extensions et les parseurs React. Les suites complètes vertes du commit précédent constituent le point de référence avant M26 : 5 861 tests Rust et 3 506 tests frontend.
