# Écrire sa propre extension

**Emplacement site** — Intégrations › Extensions › Écrire sa propre extension
**Répond à** — « Comment j'écris ma première extension, de zéro à un outil qui marche dans Beaver ? »
**Sources** — `EXTENSIONS.md` (racine du dépôt) ; `src-tauri/resources/extension-host/` (`loader.mjs`, `extension-api.mjs`, `module-loader.mjs`, `host.mjs`, `protocol.mjs`, `diagnostics.mjs`, `ui-api.mjs`, `contract.json`, `sdk/index.mjs`, `sdk/index.d.ts`) ; `src-tauri/resources/extension-ui/contract.json` ; `src-tauri/src/services/extensions/` (`manifest.rs`, `manifest_source.rs`, `validation.rs`, `types.rs`, `storage.rs`, `managed_store.rs`, `source_validation.rs`, `git_source.rs`, `host_process_spawn.rs`, `process_environment.rs`, `host_identity.rs`, `ui_startup.rs`) ; `src-tauri/src/services/agent_local/` (`permission_policy.rs`, `tool_plan_guard.rs`, `subagent_tool_profile.rs`) ; `src-tauri/src/services/mcp_bridge/` (`schema_definition.rs`, `schema_limits.rs`) ; `src-tauri/src/commands/agent_chat_task/session_events.rs` ; `src/i18n/fr.json` ; `src/features/extension-ui/core-occupants.tsx`
**Vérification** — Vérifié dans le code, sauf mentions contraires signalées ligne par ligne. Aucun écran n'a été observé : les libellés cités viennent des clés de traduction, pas d'une capture.

---

## Plan de page proposé

1. Ce que vous allez construire
2. Ce que vous acceptez avant d'écrire une ligne
3. Le projet minimal
4. Le manifeste, champ par champ
5. Le squelette du code
6. Votre premier outil
7. La classe d'effet : ce que Beaver fera sans vous demander
8. Écouter un événement
9. Ajouter une vue dans l'interface
10. Les niveaux d'API, et quand chacun s'impose
11. Installer, approuver, recharger
12. Distribuer depuis un dépôt Git
13. Déboguer : ce que vous voyez quand ça plante
14. Ce qu'une extension ne peut pas faire

---

## Contenu

### 1. Ce que vous allez construire

Une extension Beaver est un module JavaScript ou TypeScript que Beaver charge dans un processus Node.js qui lui appartient. Elle peut ajouter des outils à l'Agent, réagir à des événements et poser des éléments dans l'interface.

- Pas de serveur à écrire, pas de port à ouvrir, pas de jeton d'appairage : Beaver lance le point d'entrée et lui passe directement l'objet `beaver` (`EXTENSIONS.md` § Choisir rapidement, l. 47-49 ; confirmé par `loader.mjs:116-126`). *Vérifié dans le code.*
- Le code de l'extension est du **JavaScript ou du TypeScript**, pas de Python, Rust, Go, Java ni C# (`EXTENSIONS.md` § Limites actuelles). Une extension peut en revanche lancer un programme écrit dans un autre langage, puisqu'elle a accès complet à Node.js. *Issu de EXTENSIONS.md, cohérent avec l'accès Node complet vérifié.*
- Une extension personnalisée est de **catégorie « Local »** dans le code, même quand elle vient de Git ou de npm : seuls les plugins livrés avec Beaver sont « Builtin » (`types.rs:14-18`, `host_identity.rs:12-18`). *Vérifié dans le code.*

### 2. Ce que vous acceptez avant d'écrire une ligne

À dire **avant** le premier bout de code sur le site, pas en note de bas de page.

- Le code d'une extension s'exécute avec **les droits du compte utilisateur**. Ce n'est pas un bac à sable (`EXTENSIONS.md` § À retenir avant de commencer ; § Ce que Beaver isole — et ce qu'il n'isole pas). *Issu de EXTENSIONS.md.*
- Ce que l'isolation apporte réellement : chaque extension tierce activée a **son propre processus Node.js**, avec sa propre identité attribuée par Rust (`host_identity.rs:6-18`, `host_process_spawn.rs:93-118`). Une extension qui plante n'arrête pas les autres. Elle n'est pas empêchée de lire vos fichiers pour autant. *Vérifié dans le code.*
- L'environnement du processus est **vidé** avant le lancement : `env_clear()`, puis seulement `PATH`, `TMPDIR`, `TMP`, `TEMP`, et `SystemRoot` sous Windows (`process_environment.rs:34-56`). Conséquence directe pour l'auteur : **aucune de vos variables d'environnement habituelles n'existe dans l'extension**. Ni `HOME`, ni `USER`, ni vos clés. *Vérifié dans le code.*
- Une extension explicitement approuvée peut demander les secrets du coffre par l'API du SDK — clé de fournisseur, jeton OAuth MCP, valeur d'environnement MCP, jeton de canal (`extension-api.mjs:150-166`). L'approbation porte sur **tout son code et toutes ses dépendances**, sans permission par clé (`EXTENSIONS.md` § Isolation et confiance). *Vérifié dans le code pour les méthodes ; le caractère global de l'approbation est issu de EXTENSIONS.md.*

### 3. Le projet minimal

Deux formes possibles, et le choix a des conséquences.

**Forme 1 — un fichier isolé.** Vous ajoutez directement `mon-extension.ts`. Beaver fabrique un manifeste implicite :

- `id` = `local.<nom-du-fichier-nettoyé>.<8 caractères hexadécimaux tirés d'un UUID>` ;
- `name` = nom du fichier sans son extension ; `version` = **`0.0.0`** ; `runtime` = `node` ; `access` = `full` ; `essential` = `false` ;
- `apiLevel` = **`advanced`**, pas `stable` (`manifest_source.rs:33-56`). *Vérifié dans le code.*

Conséquence à écrire noir sur blanc : un fichier isolé reçoit le niveau d'API le plus permissif et une identité qui change à chaque ajout. C'est bon pour un essai, mauvais pour tout le reste.

**Forme 2 — un dossier avec manifeste**, la seule forme recommandée :

```text
hello-beaver/
├── beaver-extension.json
├── index.ts
├── package.json
└── package-lock.json
```

(`EXTENSIONS.md` § Structure recommandée. `package.json` et son verrou ne servent que si l'extension a des dépendances npm.)

- Beaver cherche le manifeste dans le dossier sous **trois noms, dans cet ordre** : `beaver-extension.json`, puis `beaver.json`, puis `package.json`. **Le premier trouvé gagne**, les suivants ne sont pas lus (`manifest_source.rs:4-5`, `manifest.rs:107-117`). *Vérifié dans le code.*
- Le point d'entrée doit porter une extension de fichier reconnue : `js`, `mjs`, `cjs`, `jsx`, `ts`, `mts`, `cts`, `tsx`, `mtsx`, `ctsx` (`manifest_source.rs:7-9`, appliqué par `manifest.rs:168-179`). *Vérifié dans le code.*
- Le chemin de `main` est canonicalisé puis vérifié comme restant **sous la racine de l'extension** : un `..` ou un lien symbolique qui sort du dossier est refusé (`manifest.rs:168-201`). *Vérifié dans le code.*

**Où poser le dossier.** N'importe où sur votre disque : Beaver enregistre le chemin canonique et lit la source sur place, sans la copier (`manifest.rs:66-86`). Ce qu'il écrit chez lui :

| Ce que Beaver écrit | Chemin |
|---|---|
| Registre des extensions | `~/.local/share/cl-go-dash/extensions.json` (`storage.rs:8, 34-36`) |
| Copies gérées Git et npm | `~/.local/share/cl-go-dash/extension-installs/` (`managed_store.rs:5, 49-51`) |

*Vérifié dans le code.* Le chemin de données est le même sur les trois systèmes (règle projet `data_dir()`).

**Le SDK.** N'embarquez jamais `@beaver/sdk` dans le paquet distribué : au chargement, l'Hôte résout cet import vers **sa propre copie**, par un alias Jiti pointant sur `sdk/index.mjs` (`module-loader.mjs:5-12`). Utilisez le SDK uniquement comme dépendance de développement, pour l'autocomplétion (`EXTENSIONS.md` § Préparer l'environnement de développement). *Vérifié dans le code.*

**Faut-il compiler ?** Non pour une extension simple.

- Les fichiers `.js`, `.cjs`, `.mjs` passent par l'`import()` natif de Node ;
- tous les autres, dont `.ts` et `.tsx`, passent par **Jiti** ;
- et un `.js` qui échoue sur une extension inconnue ou sur `@beaver/sdk` **retombe sur Jiti** (`module-loader.mjs:13-32`). *Vérifié dans le code.*

> Précision utile pour le site : `EXTENSIONS.md` (l. 202) attribue tout le chargement à Jiti. Le code n'y recourt que pour les fichiers non natifs et en repli. Le résultat pour l'auteur est le même — aucune compilation obligatoire — mais la phrase du site doit être exacte.

### 4. Le manifeste, champ par champ

Exemple minimal de `beaver-extension.json` (repris de `EXTENSIONS.md` § Créer le manifeste) :

```json
{
  "id": "com.example.hello",
  "name": "Hello Beaver",
  "version": "1.0.0",
  "description": "Ajoute un outil de salutation.",
  "author": "Example",
  "beaverApi": "1",
  "runtime": "node",
  "main": "./index.ts",
  "access": "full",
  "apiLevel": "stable",
  "essential": false
}
```

Le tableau complet des champs est en section « Tableaux ». Les règles qui bloquent réellement un débutant :

- **`beaverApi` doit valoir exactement la chaîne `"1"`.** La comparaison est stricte, avant toute autre validation, et l'échec produit l'erreur `extensions_api_incompatible` (`manifest.rs:49-54`, `validation.rs:37-39`, valeur `apiVersion: "1"` dans `extension-host/contract.json`). *Vérifié dans le code.*
- **`runtime` ne peut valoir que `node`** pour une extension personnalisée. La valeur `builtin` existe mais est réservée aux plugins livrés avec Beaver (`validation.rs:40-42`, `validation.rs:20-25`). *Vérifié dans le code.*
- **`access` doit valoir `full` dès que `runtime` vaut `node`** — c'est vérifié explicitement, avec le message « Une extension Node.js possède un accès complet. » (`validation.rs:43-48`). Le champ est facultatif, sa valeur par défaut est `full` (`types.rs:57-59`). Écrire `"access": "core"` refuse le manifeste. *Vérifié dans le code.*
- **`apiLevel` n'accepte que `stable` ou `advanced`**, et vaut `stable` par défaut dans un manifeste (`types.rs:48-55`). *Vérifié dans le code.*
- **`id`** : non vide, **96 caractères maximum**, uniquement des lettres et chiffres ASCII plus `.`, `_` et `-`, et il doit commencer par une lettre ou un chiffre (`validation.rs:120-131`, `MAX_IDENTIFIER_CHARS` = 96 dans `contract.json`). Un nom de domaine inversé — `com.example.hello` — reste la convention recommandée par `EXTENSIONS.md`. *Vérifié dans le code.*
- **`name`** : 100 caractères maximum ; **`version`** : 64 caractères maximum ; `author`, `homepage` et `description` : 2 000 caractères chacun (`validation.rs:35-36, 86-95`, limites `maxExtensionNameChars: 100` et `maxExtensionTextChars: 2000` dans `contract.json`). *Vérifié dans le code.*

**Le même manifeste dans `package.json`.** Le bloc `beaver` porte les champs Beaver, et Beaver **recopie depuis la racine du package** les champs `name`, `version`, `description` et `main` s'ils manquent dans le bloc (`manifest.rs:137-166`). C'est pour cela que l'exemple ci-dessous fonctionne alors que le bloc `beaver` ne contient ni `name`, ni `version`, ni `main` :

```json
{
  "name": "hello-beaver",
  "version": "1.0.0",
  "type": "module",
  "main": "./index.ts",
  "beaver": {
    "id": "com.example.hello",
    "beaverApi": "1",
    "runtime": "node",
    "access": "full",
    "apiLevel": "stable",
    "essential": false
  }
}
```

(`EXTENSIONS.md` § Créer le manifeste ; mécanisme de recopie vérifié dans `manifest.rs:151-156`.) L'`id`, lui, n'est **jamais** déduit du nom npm : il doit être écrit dans le bloc `beaver`.

### 5. Le squelette du code

Ce que fait Beaver au chargement, dans l'ordre, avec les trois étapes que vous verrez dans les diagnostics — `import`, `activate`, `register` (`contract.json` → `loadStages` ; `loader.mjs:108-140`) :

1. **`import`** — l'Hôte importe le module d'entrée.
2. **`activate`** — il cherche la fonction d'activation et l'appelle avec l'objet `beaver`. **Deux formes sont acceptées** : le module par défaut *est* une fonction, ou il expose une méthode `activate` (`loader.mjs:119-126`). Si ni l'une ni l'autre : erreur `activate_missing`.
3. **`register`** — il vérifie l'unicité des noms d'outils, encode l'instantané des contributions, contrôle qu'il tient dans le message du protocole, **et seulement alors** publie les outils (`loader.mjs:129-148`).

Point important pour l'auteur : les outils ne sont publiés **qu'après** la validation complète. Une extension qui échoue en étape `register` ne laisse aucun outil derrière elle.

- **`defineExtension` ne fait rien à l'exécution** : elle retourne son argument tel quel (`sdk/index.mjs:1-3`). Son seul rôle est de faire vérifier la forme de votre objet par TypeScript. C'est une aide d'écriture, pas un mécanisme. *Vérifié dans le code.*
- **`deactivate()`** est lu sur le module et appelé quand l'Hôte est remis à zéro (`loader.mjs:173-186`). Une exception levée dedans est avalée : le nettoyage des autres extensions continue. Vous y arrêtez les minuteries, fermez les fichiers, retirez les écouteurs (`EXTENSIONS.md` § Cycle de vie du module).
- Les fonctions rendues par `beaver.on`, `beaver.ui.register` et `beaver.ui.onAction` sont **idempotentes** : les appeler deux fois ne casse rien (`extension-api.mjs:85-93`, `ui-api.mjs:55-64, 79-84`). *Vérifié dans le code.*

### 6. Votre premier outil

**L'exemple canonique**, à reprendre tel quel sur le site (`EXTENSIONS.md` § Coder un premier outil, l. 292-329) :

```ts
import { defineExtension } from "@beaver/sdk";

let unsubscribe = () => {};

export default defineExtension({
  activate(beaver) {
    beaver.registerTool({
      name: "hello",
      description: "Retourne une salutation pour la personne demandée.",
      parameters: {
        type: "object",
        properties: {
          name: { type: "string", description: "Nom de la personne." },
        },
        required: ["name"],
        additionalProperties: false,
      },
      effect: "read-only",
      async execute({ name }, context) {
        return {
          content: `Bonjour ${String(name)} depuis ${context.workingDirectory}`,
          displaySummary: "Salutation créée",
        };
      },
    });

    unsubscribe = beaver.on("session.turn.started", async (event) => {
      // Réagir au début d’un tour sans bloquer les autres extensions.
      void event;
    });
  },

  deactivate() {
    unsubscribe();
  },
});
```

**Le contrat de `registerTool`, champ par champ.**

- **`name`** — Beaver **préfixe automatiquement** le nom par l'identifiant de l'extension, sauf s'il commence déjà par ce préfixe (`extension-api.mjs:43-48`). L'outil déclaré `hello` dans l'extension `com.example.hello` s'appelle donc `com.example.hello.hello` pour le modèle. Le nom public complet doit tenir dans **96 caractères** et respecter le motif `^[a-zA-Z0-9](?:[a-zA-Z0-9._-]*[a-zA-Z0-9])?$` (`extension-api.mjs:21-25, 57-58`). Comptez le préfixe : c'est lui qui vous rapproche de la limite. *Vérifié dans le code.*
- **`description`** — obligatoire, non vide une fois les espaces retirés, **2 000 scalaires Unicode maximum** (`extension-api.mjs:59-60`). Une description vide fait échouer l'enregistrement, donc toute l'activation.
- **`parameters`** — doit être un objet JSON, pas un tableau ; absent, il devient `{ type: "object" }` (`extension-api.mjs:52, 61-63`). Puis Rust revalide, et c'est là que « schéma JSON strict » devient concret :
  - la racine **doit** porter `"type": "object"`, sinon « Le schéma d'un outil doit décrire un objet. » (`validation.rs:186-197`) ;
  - seuls **38 mots-clés** sont acceptés, à tous les niveaux : `$id`, `$schema`, `title`, `description`, `default`, `examples`, `deprecated`, `readOnly`, `writeOnly`, `format`, `type`, `properties`, `required`, `additionalProperties`, `minProperties`, `maxProperties`, `items`, `minItems`, `maxItems`, `uniqueItems`, `minLength`, `maxLength`, `pattern`, `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`, `multipleOf`, `enum`, `const`, `allOf`, `anyOf`, `oneOf`, `not` (`schema_definition.rs:8-41`). **`$ref` et `$defs` n'y sont pas** : un schéma qui référence une définition est refusé ;
  - profondeur maximale **16**, et **256 nœuds** au total (`schema_definition.rs:5-6`, `schema_limits.rs:3-4`). *Vérifié dans le code.*
- **`effect`** — voir la section suivante. Une valeur absente ou inconnue devient `unknown`, pas une erreur (`extension-api.mjs:54`).
- **`execute`** — obligatoire, et doit être une fonction, sinon `invalid_tool` (`extension-api.mjs:38-41`).

**Ce que `execute` reçoit et rend.**

- Deuxième argument, le contexte : un objet **gelé qui ne contient qu'un champ**, `workingDirectory` (`loader.mjs:36-47`, `sdk/index.d.ts:55-57`). Il est refusé s'il est absent, vide, dépasse **1 024 caractères** ou contient un octet nul. Il n'y a ni identifiant de session, ni langue, ni modèle : ne les cherchez pas.
- Retour accepté : une chaîne, ou un objet `{ content, isError?, displaySummary?, truncated? }` (`sdk/index.d.ts:24-35`).
- `content` en chaîne est **tronqué à 1 048 576 unités** ; `displaySummary` est **tronqué à 1 024 caractères** — une valeur écrite en dur dans le code, absente de la table des limites de `EXTENSIONS.md` (`loader.mjs:49-62`). La troncature est silencieuse : elle lève le drapeau `truncated`, elle n'échoue pas. *Vérifié dans le code.*
- **Délai d'exécution : 55 secondes.** Au-delà, l'appel est rejeté avec `tool_timeout` (`loader.mjs:23-32`, `toolCallTimeoutMs: 55000`). Attention : la course rejette la promesse, **elle n'interrompt pas votre code** — votre travail continue en arrière-plan sans que personne n'en lise le résultat. *Vérifié dans le code.*

**Ce qui fait échouer l'enregistrement.** `registerTool` **lève une exception**, qui remonte dans `activate` : toute l'extension échoue et rien n'est publié (`loader.mjs:151-157`). Causes : plus de 64 outils, `execute` absent, nom invalide, description vide, `parameters` invalide (`extension-api.mjs:35-66`), ou nom d'outil déjà pris dans l'Hôte, ou 256 outils atteints au total (`loader.mjs:160-171`).

### 7. La classe d'effet : ce que Beaver fera sans vous demander

C'est la déclaration qui a le plus de conséquences dans tout le fichier, et c'est un simple mot dans un objet.

Le tableau complet est en section « Tableaux ». Ce qu'il faut retenir :

- L'unique classe qui **n'affiche pas de demande d'approbation** en mode « Demande d'approbation » est `read-only`, et c'est aussi la seule autorisée en mode Plan (`permission_policy.rs:13-41`, `tool_plan_guard.rs:34-48`). *Vérifié dans le code.*
- Une valeur absente ou mal orthographiée n'est **pas** refusée : elle devient `unknown`, la classe la plus restrictive (`extension-api.mjs:54`). Votre outil marchera, mais il demandera confirmation à chaque appel et sera interdit en mode Plan. Un `effect: "readonly"` au lieu de `"read-only"` produit exactement cela, sans message.
- Le mode « Accès complet » contourne la garde de permission pour toutes les classes (`permission_policy.rs:43-45`). Votre classe d'effet ne protège donc que les utilisateurs des modes plus stricts — raison de plus pour la déclarer juste.
- Deux conséquences documentées nulle part ailleurs, et utiles à un auteur :
  - **exécution en parallèle** : seuls `read-only` et `external-read` peuvent être appelés en parallèle avec d'autres lectures (`parallel_read`, `permission_policy.rs:16-27`) ;
  - **mémorisation de l'accord** : `external-read`, `local-write` et `external-write` peuvent voir leur approbation retenue pour la session ; `process`, `secret` et `unknown` **redemandent à chaque appel** (`allow_session_cache`, `permission_policy.rs:22-39`). *Vérifié dans le code.*
- **Sous-agents** : un sous-agent de profil « explorateur » ne reçoit que les outils d'extension déclarés `read-only` ; le profil « codeur » les reçoit tous (`subagent_tool_profile.rs:10-12`). Si votre outil doit servir aux sous-agents d'exploration, il doit être honnêtement `read-only`. *Vérifié dans le code.*

### 8. Écouter un événement

- **Un seul événement public existe aujourd'hui** : `session.turn.started` (`contract.json` → `events`, `extension-api.mjs:78`). *Vérifié dans le code.*
- Sa charge utile contient exactement **deux champs** : `sessionId` et `mode` (`session_events.rs:1-9`). *Vérifié dans le code* — et c'est bien la valeur annoncée par `EXTENSIONS.md`.
- `beaver.on` rend une fonction de désabonnement à conserver et à appeler dans `deactivate()` (`extension-api.mjs:85-93`).
- **64 gestionnaires maximum par extension**, au-delà : `invalid_event_handler` (`extension-api.mjs:70-75`, `maxEventsPerExtension: 64`). Cette limite ne figure pas dans la table de `EXTENSIONS.md`.
- **Budget : 5 secondes** (`eventHandlerTimeoutMs: 5000`). Nuance qui compte : à l'expiration, Beaver **cesse d'attendre** mais **n'interrompt pas** le gestionnaire — la course se résout, elle ne rejette pas (`extension-api.mjs:198-217`). Votre code continue de tourner sans que rien ne le surveille. *Vérifié dans le code.*
- Une exception dans un gestionnaire **ne bloque pas la livraison aux autres extensions** : chaque livraison est isolée (`loader.mjs:88-98`).
- Au-delà de **64 gestionnaires en vol** simultanés, la livraison est refusée avec `too_many_event_handlers_running` (`extension-api.mjs:198-201`).

### 9. Ajouter une vue dans l'interface

**Il faut d'abord le déclarer dans le manifeste.** Sans le bloc `ui`, chaque appel à `beaver.ui.register` est refusé (`ui-api.mjs:19-22`) :

```json
{
  "ui": {
    "apiVersion": "1",
    "mode": "standard"
  }
}
```

(`EXTENSIONS.md` § Mode standard.) Le bloc `ui` est validé en mode **strict** : `apiVersion`, `mode`, `entry` — tout autre champ refuse le manifeste (`types.rs:68-75`, `deny_unknown_fields`). En mode `standard`, `entry` est interdit ; en mode `advanced`, `entry` est obligatoire **et** `apiLevel` doit valoir `advanced` (`validation.rs:57-77`). *Vérifié dans le code.*

**Ce qu'on peut poser, en mode standard.**

- Quatre types de contribution : `tab`, `settingsTab`, `action`, `theme` (`extension-ui/contract.json` → `contributionTypes`).
- Huit emplacements publics, et un seul est limité à la conversation Agent — `agent.composer.leading`, qui porte en plus `thirdPartyChatAllowed: false` (`extension-ui/contract.json` → `placements`). La liste complète est en section « Tableaux ».
- Cinq opérations de placement : `before`, `after`, `replace`, `move`, `remove` (`extension-ui/contract.json` → `placementOperations`).
- Onze primitives de vue : `stack`, `row`, `heading`, `text`, `badge`, `separator`, `textField`, `numberField`, `select`, `toggle`, `button`. Vingt icônes nommées. Quarante-sept jetons de thème redéfinissables. Sept langues plus `default` (`extension-ui/contract.json`). Vous ne pouvez ni écrire de CSS, ni de JSX : vous décrivez des données, Beaver dessine. *Vérifié dans le code.*

**Les occupants `beaver.*` utiles pour se placer.** Ce sont les identifiants des éléments que Beaver publie lui-même : `beaver.agent-local`, `beaver.heartbeat`, `beaver.personality`, `beaver.settings`, les onglets de réglages `beaver.tools`, `beaver.providers`, `beaver.extensions`, et les actions de barre préfixées `beaver.toolbar.` (`EXTENSIONS.md` § Emplacements et composition modulaire). **L'autorité courante est `src/features/extension-ui/core-occupants.tsx`** — le fichier existe et doit être relu avant de publier une extension qui vise un occupant précis. *Issu de EXTENSIONS.md ; l'existence du fichier est vérifiée, son contenu n'a pas été relu ligne à ligne.*

Exemple à reprendre — ajouter un onglet juste après Réveils, sans toucher au composant de Réveils (`EXTENSIONS.md` § Emplacements et composition modulaire) :

```ts
const unregisterTab = beaver.ui.register({
  type: "tab",
  id: "dashboard",
  placement: "app.navigation.primary",
  order: 15,
  operation: "after",
  targetId: "beaver.heartbeat",
  label: {
    default: "Dashboard",
    fr: "Tableau de bord",
  },
  icon: "activity",
  detail: {
    type: "stack",
    children: [
      {
        type: "heading",
        text: { default: "Dashboard", fr: "Tableau de bord" },
      },
    ],
  },
});
```

**Deux différences de comportement à ne pas rater.**

- `registerTool` **lève** en cas de refus. `beaver.ui.register` et `beaver.ui.onAction`, eux, **ne lèvent pas** : ils rendent une fonction vide et ajoutent un diagnostic (`ui-api.mjs:22-64, 87-91`). Votre extension se charge, ses outils marchent, et l'interface est simplement absente. Il faut aller lire le diagnostic pour le savoir. *Vérifié dans le code.*
- Deux occupants sont **protégés** contre `remove` et `replace` : `beaver.settings` dans la navigation principale, et `beaver.extensions` dans les réglages Intégrations (`extension-ui/contract.json` → `protectedOccupants`). C'est ce qui garantit qu'un utilisateur peut toujours revenir désactiver votre extension. *Vérifié dans le code.*

### 10. Les niveaux d'API, et quand chacun s'impose

Attention au vocabulaire : le manifeste porte **deux champs différents**, avec deux jeux de valeurs.

| Champ | Valeurs acceptées | Source |
|---|---|---|
| `apiLevel` | `stable`, `advanced` | `types.rs:48-51` |
| `ui.mode` | `standard`, `advanced` | `types.rs:63-66` |

Il n'existe **pas** de niveau d'API nommé `standard` : `standard` est un mode d'interface. Les « trois niveaux » du guide sont en réalité trois combinaisons :

1. **`apiLevel: "stable"`, sans bloc `ui`** — des outils, des événements, des skills, des ressources. Le défaut, et le bon choix par défaut.
2. **`apiLevel: "stable"` + `ui.mode: "standard"`** — s'ajoutent des onglets, réglages, actions et thèmes décrits en données. Vous gardez le design, les traductions, les limites et la validation de Beaver.
3. **`apiLevel: "advanced"` + `ui.mode: "advanced"`** — un module JavaScript chargé dans **la même WebView que Beaver**, sans bac à sable, et l'accès à `beaver.unstable.*`.

Quand le niveau `advanced` s'impose vraiment :

- vous devez manipuler le DOM ou monter une interface que les primitives standard ne permettent pas (`EXTENSIONS.md` § Mode avancé) ;
- vous voulez **remplacer un outil natif** — `beaver.unstable.registerReplacement(...)` lève `advanced_api_required` si `apiLevel` n'est pas `advanced` (`extension-api.mjs:174-179`). Ce sujet a sa propre page, voir *Renvois*.

Ce que `advanced` **ne** donne **pas** : `beaver.unstable.call(...)` ne peut appeler que des méthodes déclarées de niveau `advanced` dans le contrat (`extension-api.mjs:167-173, 219-225`). **Le contrat actuel n'en publie aucune** : les douze méthodes du pont sont toutes de niveau `stable` (`contract.json` → `methods.hostToCore`). Aujourd'hui, tout appel à `unstable.call` échoue donc avec `core_method_unavailable`. Ce n'est pas une porte dérobée vers le backend. *Vérifié dans le code — plus catégorique que la formulation de `EXTENSIONS.md`.*

### 11. Installer, approuver, recharger

Ne pas détailler ici : la page *Le centre des extensions* couvre le parcours. Ce qu'un auteur doit savoir pour boucler sa première extension :

- **Installer et faire confiance sont deux décisions séparées.** Une extension ajoutée reste désactivée et non approuvée : dans le code, `enabled: false` et `trusted: false` à la création de la fiche (`manifest.rs:76-78`). *Vérifié dans le code.*
- Le parcours dans l'application : **Réglages → Extensions**, bouton **Ajouter**, puis choix de la source — **Choisir un fichier**, **Choisir un dossier**, **Installer depuis Git**, **Installer depuis npm** (clés `extensions.actions.add`, `extensions.add.file`, `extensions.add.folder`, `extensions.add.git`, `extensions.add.npm` dans `src/i18n/fr.json`). *Vérifié dans les traductions ; l'écran n'a pas été observé.*
- Après une modification locale de votre code : **Recharger** (`extensions.actions.reload`). Pour une installation Git ou npm : **Mettre à jour** (`extensions.actions.update`). *Vérifié dans les traductions.*
- Toute modification des fichiers couverts par l'empreinte — sources JS/TS, manifeste, artefact d'interface — **révoque l'approbation** et désactive l'extension (`EXTENSIONS.md` § Ajouter l'extension à Beaver). Pendant le développement, attendez-vous donc à ré-approuver souvent. *Issu de EXTENSIONS.md.*
- Une mise à jour qui change l'`id` est refusée : erreur `extensions_update_identity_changed`, message « La mise à jour a changé l'identité de l'extension et a été refusée. » (`contract.json` → `errors.backendCodes` ; `src/i18n/fr.json`). Gardez votre `id` stable et incrémentez `version`. *Vérifié dans le code et les traductions.*

### 12. Distribuer depuis un dépôt Git

**Oui, c'est prévu par le code** — vérifié, pas déduit.

- Beaver accepte deux formes d'adresse :
  - une **URL** de schéma `https` ou `ssh` ; le préfixe `git+` est toléré et retiré (`source_validation.rs:68-83`) ;
  - une **forme SCP** `utilisateur@hôte:chemin` (`source_validation.rs:22-26, 85-102`).
- Sont **refusés** : tout autre schéma — donc `http://` en clair —, une URL sans hôte, un **mot de passe** dans l'URL, une **chaîne de requête** `?`, un **nom d'utilisateur en HTTPS**, un chemin vide (`source_validation.rs:73-81`). *Vérifié dans le code.*
- La révision se met **après un `#`** : branche, tag ou commit. Le découpage se fait sur le **dernier** `#`, un second `#` refuse l'adresse, et la référence est bornée à **200 caractères** (`source_validation.rs:3, 59-66`).
- Formes de commit acceptées : **au moins 7 caractères** hexadécimaux pour une forme abrégée, ou **40** (SHA-1) / **64** (SHA-256) pour la forme complète (`git_source.rs:11-14, 115-127`). *Vérifié dans le code.*
- Ce que Beaver fait de votre dépôt : clone superficiel — sauf si la référence est un commit complet (`git_source.rs:107-113`) —, puis **suppression du dossier `.git`** de la copie gérée, mesure du volume occupé, et installation des seules dépendances de production si le paquet en déclare (`git_source.rs:51-69`). *Vérifié dans le code.*
- **Délai global de l'opération Git : 300 secondes** (`git_source.rs:15`). Valeur absente de `EXTENSIONS.md`. *Vérifié dans le code.*
- Conséquence pratique pour l'auteur : **précompilez** tout ce qui exige un script d'installation npm, car les scripts de cycle de vie npm sont désactivés (`EXTENSIONS.md` § Dépendances npm). *Issu de EXTENSIONS.md.*

### 13. Déboguer : ce que vous voyez quand ça plante

**Le fait le plus important de cette page, et il n'est écrit nulle part ailleurs.**

Dans le processus Hôte, `console.log`, `console.info`, `console.debug`, `console.warn` et `console.error` sont **remplacés par des fonctions vides**, et `process.stdout.write` est lui aussi neutralisé (`host.mjs:12-17`). La raison est structurelle : la sortie standard **est** le canal du protocole JSON-RPC entre Beaver et l'Hôte, et le vrai écrivain a été capturé avant la neutralisation (`protocol.mjs:16`). En parallèle, la sortie d'erreur du processus est jetée à la création — `stderr(Stdio::null())` (`host_process_spawn.rs:99`).

Autrement dit : **une extension ne peut rien afficher. Aucune trace, nulle part.** *Vérifié dans le code.* C'est le premier réflexe qu'un développeur perd en arrivant sur Beaver, et il faut le lui dire à l'avance.

**Ce que vous obtenez à la place : un diagnostic structuré**, composé de quatre champs seulement (`diagnostics.mjs:11-54`) :

| Champ | Contenu |
|---|---|
| `stage` | `import`, `activate` ou `register` |
| `code` | un des sept codes de l'Hôte |
| `file` | le **nom seul** du fichier d'entrée, 128 caractères maximum — jamais le chemin |
| `line` / `column` | position extraite de la pile, si elle y figure |

**Votre message d'erreur n'est jamais transmis.** Il sert seulement à choisir le code : une erreur `SyntaxError` ou un message commençant par `ParseError:` donne `syntax_error` ; les codes Node `ERR_MODULE_NOT_FOUND`, `ERR_UNKNOWN_FILE_EXTENSION` et `MODULE_NOT_FOUND` donnent `module_not_found` ; sinon c'est l'étape qui décide (`diagnostics.mjs:23-34`). *Vérifié dans le code.*

Les sept codes et leur libellé français exact (`contract.json` → `diagnostics.hostCodes` ; `src/i18n/fr.json`, clés `extensions.diagnostics.codes.*`) sont en section « Tableaux ».

**Où lire tout cela dans l'application** (clés de traduction vérifiées, écran non observé) :

- **Réglages → Extensions**, section **Hôte** (`extensions.sections.host`) : **État**, **Node.js**, **Jiti**, **API Beaver**, **Extensions actives**, et **Dernières erreurs de chargement** (`extensions.host.*`).
- Les quatre états possibles de l'Hôte : **Arrêté**, **Démarrage**, **En cours**, **Erreur** (`extensions.host.states.*`, correspondant à `hostStates` du contrat).
- Bouton **Redémarrer l'hôte** (`extensions.actions.restartHost`) et **Ouvrir la source** pour auditer le code réellement chargé (`extensions.actions.openSource`).

**Méthode de mise au point recommandée**, puisque l'affichage est impossible : faites remonter l'information **par le résultat de l'outil** — `content` et `displaySummary` sont, eux, transmis et affichés dans la conversation.

> Ne jamais y mettre une clé ni un jeton. `EXTENSIONS.md` (§ Utiliser les services exposés par Beaver) est explicite : un secret remis au JavaScript ne peut plus être effacé de la mémoire de façon garantie.

**Autres garde-fous à connaître :**

- Un Hôte qui plante est redémarré automatiquement **3 fois au maximum sur une fenêtre de 5 minutes** (`maxHostRestartsPerWindow: 3`, `hostRestartWindowSeconds: 300`).
- Si une interface avancée empêche Beaver de s'ouvrir : quitter complètement, puis maintenir **Maj** au redémarrage, ou lancer l'exécutable avec l'argument exact **`--safe-mode`** (`ui_startup.rs:8` définit la chaîne `--safe-mode` ; procédure décrite dans `EXTENSIONS.md` § En cas de problème). *La chaîne est vérifiée dans le code ; le parcours complet est issu de EXTENSIONS.md.*

### 14. Ce qu'une extension ne peut pas faire

À présenter comme une liste franche, pas comme des excuses.

- **Écrire dans un autre langage que JavaScript ou TypeScript** pour le point d'entrée (`EXTENSIONS.md` § Limites actuelles). Elle peut lancer un programme tiers, mais elle doit le fournir et gérer sa compatibilité.
- **Afficher quoi que ce soit** : console et sortie standard neutralisées (`host.mjs:12-17`), sortie d'erreur jetée (`host_process_spawn.rs:99`).
- **Lire l'environnement du compte** : le processus démarre avec un environnement vidé (`process_environment.rs:41-56`).
- **Écouter autre chose que `session.turn.started`** : un nom d'événement inconnu est refusé (`extension-api.mjs:78-80`, `contract.json` → `events`).
- **Appeler une méthode du cœur hors contrat** : `beaver.call` et `beaver.unstable.call` vérifient niveau et nature de la méthode avant tout envoi, sinon `core_method_unavailable` (`extension-api.mjs:219-225`).
- **Toucher aux composants React de Beaver ou à `invoke` de Tauri** en mode standard (`EXTENSIONS.md` § Emplacements et composition modulaire).
- **Retirer ou remplacer `beaver.settings` et `beaver.extensions`** (`extension-ui/contract.json` → `protectedOccupants`).
- **Recevoir plus d'un dossier de travail** : le contexte d'exécution ne contient rien d'autre (`loader.mjs:36-47`).
- **Fonctionner en mode Chat classique** : les outils d'extension ne sont disponibles qu'en mode Agent (`EXTENSIONS.md` § À retenir avant de commencer).
- **Se faire connaître de tous les modèles** : les modèles Groq via OpenRouter ne reçoivent pas les outils d'extension, et `groq/compound` ne reçoit aucun outil (`EXTENSIONS.md` § Utilisation par le modèle). *Issu de EXTENSIONS.md, non revérifié dans le code.*
- **Se piloter depuis une application déjà lancée** : le protocole d'applications externes est un chantier distinct (`EXTENSIONS.md` § Limites actuelles).

Et une limite qui n'en est pas une, à ne pas laisser croire : **rien n'empêche une extension de lire vos fichiers, d'ouvrir le réseau ou de lancer des processus.** L'accès complet à Node.js est assumé (`EXTENSIONS.md` § Ce que Beaver isole — et ce qu'il n'isole pas).

---

## Tableaux

### Champs du manifeste

| Champ | Obligatoire | Valeurs / contraintes | Source |
|---|---|---|---|
| `id` | Oui | ≤ 96 caractères, ASCII alphanumériques + `.` `_` `-`, commence par un alphanumérique | `validation.rs:120-131` |
| `name` | Oui | ≤ 100 caractères | `validation.rs:35`, `maxExtensionNameChars` |
| `version` | Oui | ≤ 64 caractères | `validation.rs:36` |
| `beaverApi` | Oui | exactement `"1"` | `manifest.rs:49-54`, `contract.json` `apiVersion` |
| `runtime` | Oui | `node` (`builtin` réservé à Beaver) | `validation.rs:40-42` |
| `main` | Oui (runtime `node`) | chemin relatif, sous la racine, extension de source reconnue | `manifest.rs:168-179`, `manifest_source.rs:7-9` |
| `access` | Non | `full` par défaut ; **doit** valoir `full` si `runtime: node` | `types.rs:57-59`, `validation.rs:43-48` |
| `apiLevel` | Non | `stable` (défaut) ou `advanced` | `types.rs:48-55` |
| `essential` | Non | booléen, `false` par défaut | `types.rs:118-119` |
| `description` | Non | ≤ 2 000 caractères | `validation.rs:86-95` |
| `author` | Non | ≤ 2 000 caractères | `validation.rs:86-95` |
| `homepage` | Non | ≤ 2 000 caractères | `validation.rs:86-95` |
| `ui` | Non | objet strict `{ apiVersion, mode, entry? }` | `types.rs:68-75`, `validation.rs:57-77` |

### Classes d'effet et conséquences réelles

| `effect` | Confirmation en mode « Demande d'approbation » | Appel en parallèle | Mode Plan | Accord retenu pour la session |
|---|---|---|---|---|
| `read-only` | Non | Oui | **Autorisé** | Non |
| `external-read` | Oui | Oui | Refusé | Oui |
| `local-write` | Oui | Non | Refusé | Oui |
| `external-write` | Oui | Non | Refusé | Oui |
| `process` | Oui | Non | Refusé | Non |
| `secret` | Oui | Non | Refusé | Non |
| `unknown` | Oui | Non | Refusé | Non |

Source : `permission_policy.rs:13-41` ; mode Plan appliqué par `tool_plan_guard.rs:34-48`. *Vérifié dans le code.* Le mode « Accès complet » contourne la confirmation pour toutes les lignes (`permission_policy.rs:43-45`).

### Emplacements d'interface publics

| Emplacement | Type de contribution accepté | Portée |
|---|---|---|
| `app.navigation.primary` | `tab` | Globale |
| `settings.navigation.preferences` | `settingsTab` | Globale |
| `settings.navigation.agent` | `settingsTab` | Globale |
| `settings.navigation.models` | `settingsTab` | Globale |
| `settings.navigation.integrations` | `settingsTab` | Globale |
| `settings.navigation.application` | `settingsTab` | Globale |
| `app.toolbar.primary` | `action` | Globale |
| `agent.composer.leading` | `action` | Conversation Agent uniquement, exclu du Chat tiers |

Source : `extension-ui/contract.json` → `placements`. *Vérifié dans le code.*

### Codes de diagnostic de chargement

| Code | Libellé français affiché | Quand |
|---|---|---|
| `module_not_found` | Module ou dépendance introuvable | codes Node `ERR_MODULE_NOT_FOUND`, `ERR_UNKNOWN_FILE_EXTENSION`, `MODULE_NOT_FOUND` |
| `syntax_error` | Erreur de syntaxe | `SyntaxError`, ou message commençant par `ParseError:` |
| `activation_failed` | L'activation a échoué | exception levée pendant l'étape `activate` |
| `registration_failed` | L'enregistrement des contributions a échoué | exception levée pendant l'étape `register` |
| `import_failed` | L'import du module a échoué | tous les autres cas |
| `resource_unavailable` | Ressource de l'extension indisponible | lecture d'un skill ou d'une ressource |
| `result_invalid` | Résultat de l'extension invalide | résultat d'outil refusé |

Sources : `diagnostics.mjs:23-34`, `contract.json` → `diagnostics.hostCodes`, `src/i18n/fr.json` (`extensions.diagnostics.codes.*`). *Vérifié dans le code et les traductions.*

### Limites qui concernent l'auteur

| Ressource | Limite | Source |
|---|---|---|
| Outils par extension | **64** | `maxToolsPerExtension` |
| Outils dans tout l'Hôte | **256** | `maxTools`, `loader.mjs:161` |
| Gestionnaires d'événements par extension | **64** | `maxEventsPerExtension` |
| Gestionnaires en vol simultanés | **64** | `maxInFlightHandlers`, `extension-api.mjs:199` |
| Skills / Ressources par extension | **32** / **64** | `maxSkillsPerExtension`, `maxResourcesPerExtension` |
| Longueur d'un identifiant (dont nom d'outil public) | **96 caractères** | `maxIdentifierChars` |
| Description d'outil | **2 000 scalaires Unicode** | `maxExtensionTextChars` |
| Profondeur / nœuds d'un schéma d'outil | **16** / **256** | `schema_definition.rs:5-6` |
| `workingDirectory` | **1 024 caractères** | `maxWorkingDirectoryChars` |
| `content` en chaîne | **1 048 576 unités**, tronqué | `loader.mjs:51`, `maxMessageBytes` |
| `displaySummary` | **1 024 caractères**, tronqué | `loader.mjs:54` |
| Durée d'un appel d'outil | **55 s** | `toolCallTimeoutMs` |
| Durée d'un gestionnaire d'événement | **5 s** (non interruptif) | `eventHandlerTimeoutMs` |
| Durée d'une action d'interface standard | **15 s** | `uiActionTimeoutMs` |
| Contributions UI / actions / thèmes par extension | **32** / **64** / **8** | `extension-ui/contract.json` → `limits` |
| Nœuds de vue / profondeur / champs par vue | **256** / **12** / **32** | `extension-ui/contract.json` → `limits` |
| JSON d'interface par extension | **262 144 octets** | `maxUiBytesPerExtension` |
| `order` d'une contribution UI | entre **−1000** et **1000** | `extension-ui/contract.json` → `validation` |
| Redémarrages automatiques d'un Hôte | **3 sur 5 minutes** | `maxHostRestartsPerWindow` |
| Extensions utilisateur enregistrées | **128** | `maxUserExtensions` |
| Processus Hôte simultanés | **32**, dont un réservé aux plugins officiels | `maxHostProcesses` |
| Délai global d'une opération Git | **300 s** | `git_source.rs:15` |

Sauf mention contraire, source : `src-tauri/resources/extension-host/contract.json` → `limits` / `timeouts`. *Vérifié dans le code.*

---

## Encadrés

**Encadré 1 — Votre extension n'est pas dans un bac à sable.** *(à placer avant le premier bloc de code)*
Le code d'une extension s'exécute avec les droits de votre compte : il peut lire vos fichiers, ouvrir le réseau et lancer des processus. Beaver isole chaque extension dans son propre processus et vide son environnement — cela protège la **stabilité** de l'application, pas votre machine contre du code que vous avez approuvé. Installez et activez uniquement du code que vous avez lu, ou dont vous connaissez l'auteur. *(`EXTENSIONS.md` § Ce que Beaver isole ; `process_environment.rs:41-56`, `host_identity.rs:12-18`.)*

**Encadré 2 — La classe d'effet est une promesse, et Beaver la croit.** *(section 7)*
Beaver ne vérifie pas ce que votre outil fait réellement : il applique la politique de permission correspondant à la classe que vous avez déclarée. Déclarer `read-only` un outil qui écrit sur le disque supprime la demande d'approbation et **autorise l'outil en mode Plan**, un mode où l'utilisateur s'attend précisément à ce que rien ne soit modifié. Déclarez la classe réelle. *(`permission_policy.rs:13-41`, `tool_plan_guard.rs:34-48`.)*

**Encadré 3 — Une faute de frappe sur `effect` ne se voit pas.** *(section 7)*
`effect: "readonly"` au lieu de `"read-only"` n'est pas une erreur : la valeur devient `unknown`, la classe la plus restrictive. L'outil fonctionne, mais il demande confirmation à chaque appel et devient inutilisable en mode Plan — sans qu'aucun message ne l'explique. *(`extension-api.mjs:54`.)*

**Encadré 4 — `secret` : ce que vous emportez ne revient pas.** *(section 7 ou 2)*
Un outil de classe `secret` reçoit une valeur du coffre de l'utilisateur. Une fois cette valeur remise à votre code JavaScript, **Beaver ne peut plus garantir son effacement de la mémoire**. Désactiver ou supprimer l'extension ne retire pas une copie déjà obtenue : la seule réponse à une fuite est de révoquer le secret chez son fournisseur. Ne l'écrivez jamais dans un log, un message d'erreur, un résultat d'outil ou un fichier de diagnostic. *(`EXTENSIONS.md` §§ Utiliser les services exposés par Beaver, Isolation et confiance.)*

**Encadré 5 — L'interface avancée partage la page de Beaver.** *(section 10)*
Un module d'interface `advanced` s'exécute dans **la même WebView** que Beaver, avec les mêmes droits qu'elle. Il n'y a pas de bac à sable, et une boucle synchrone dans ce module bloque toute l'application : la seule sortie est alors de redémarrer, éventuellement en mode sûr. Beaver demande pour cela une confirmation supplémentaire à l'activation. N'y allez que si le mode standard ne suffit pas. *(`EXTENSIONS.md` § Mode avancé ; `ui_startup.rs:8` pour `--safe-mode`.)*

**Encadré 6 — Vous ne pouvez rien afficher.** *(section 13)*
`console.log` et toutes ses variantes sont remplacées par des fonctions vides, la sortie standard est le canal du protocole, et la sortie d'erreur du processus est jetée. Une extension n'écrit nulle part. Pour observer votre code, faites remonter l'information par le `content` ou le `displaySummary` du résultat d'outil — jamais un secret. *(`host.mjs:12-17`, `protocol.mjs:16`, `host_process_spawn.rs:99`.)*

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| L'extension ne se charge pas, diagnostic `activate_missing` | Ni fonction par défaut, ni méthode `activate` sur le module | Utiliser `export default defineExtension({ activate(beaver) {...} })` (`loader.mjs:119-126`) |
| Rien ne se charge, aucun message compréhensible | `beaverApi` absent ou différent de `"1"` | Écrire `"beaverApi": "1"` ; le contrôle a lieu avant tout le reste (`manifest.rs:49-54`) |
| Manifeste refusé, message sur l'accès | `"access": "core"` avec `runtime: node` | Retirer le champ ou écrire `"full"` (`validation.rs:43-48`) |
| Manifeste refusé alors que le bloc `ui` semble correct | Un champ en trop dans `ui` — le bloc est validé strictement | Ne garder que `apiVersion`, `mode` et, en mode avancé, `entry` (`types.rs:68-75`) |
| `ui.mode: "advanced"` refusé | `apiLevel` resté à `stable` | Les deux doivent valoir `advanced` (`validation.rs:61-74`) |
| Schéma d'outil refusé, l'extension entière échoue | La racine n'a pas `"type": "object"`, ou le schéma utilise `$ref` / `$defs` — hors des 38 mots-clés autorisés | Écrire un schéma plat sans référence (`validation.rs:191-193`, `schema_definition.rs:8-41`) |
| L'outil demande confirmation à chaque appel sans raison apparente | `effect` mal orthographié → classe `unknown` | Reprendre une des sept valeurs exactes (`extension-api.mjs:54`) |
| L'outil est invisible en mode Plan | Toute classe autre que `read-only` y est refusée | Vérifier la classe déclarée (`permission_policy.rs:13-41`) |
| Le modèle ne trouve pas l'outil sous le nom écrit dans le code | Beaver préfixe le nom par l'`id` de l'extension | Chercher `<id>.<nom>`, par exemple `com.example.hello.hello` (`extension-api.mjs:43-48`) |
| Nom d'outil trop long, enregistrement refusé | Le préfixe compte dans les 96 caractères | Raccourcir l'`id` ou le nom d'outil (`extension-api.mjs:57`) |
| Les outils marchent, l'interface n'apparaît pas | Bloc `ui` absent du manifeste, ou contribution refusée — `ui.register` ne lève pas d'exception | Lire les diagnostics d'interface dans la fiche (`ui-api.mjs:19-22, 87-91`) |
| `console.log` n'affiche rien | Console et sortie standard neutralisées dans l'Hôte | Passer par le résultat d'outil (`host.mjs:12-17`) |
| Un `process.env.MA_VARIABLE` vaut `undefined` | L'environnement du processus est vidé au lancement | Passer la valeur par un fichier de configuration de l'extension (`process_environment.rs:41-56`) |
| Il faut ré-approuver l'extension après chaque modification | L'empreinte couvre les sources, le manifeste et l'artefact UI ; tout changement révoque l'approbation | Comportement voulu ; regrouper les modifications avant de recharger (`EXTENSIONS.md` § Ajouter l'extension à Beaver) |
| Un dépôt en `http://` est refusé | Seuls `https` et `ssh` sont acceptés | Utiliser HTTPS ou SSH (`source_validation.rs:73-81`) |
| Une URL Git avec identifiants est refusée | Mot de passe interdit, nom d'utilisateur interdit en HTTPS | Configurer l'accès hors de l'URL (`source_validation.rs:73-81`) |
| L'appel d'outil s'arrête à 55 secondes | Délai du contrat ; le code continue de tourner sans être lu | Découper le travail ou rendre un résultat partiel (`loader.mjs:23-32`) |
| Un fichier isolé se retrouve en `apiLevel: advanced` | Manifeste implicite d'un fichier isolé | Écrire un manifeste explicite (`manifest_source.rs:33-56`) |

---

## Renvois

- **`07-integrations/extensions-centre.md`** — installer une extension, la fiche, l'approbation, le suivi des installations, la désactivation et la suppression. C'est la page à lire avant celle-ci pour un utilisateur, et celle vers laquelle renvoyer toute question d'installation détaillée.
- **`07-integrations/extensions-remplacer-un-outil.md`** — `beaver.unstable.registerReplacement`, le niveau `advanced` appliqué au remplacement d'un outil natif, et ses conséquences. La présente page se limite à dire que le remplacement exige `apiLevel: "advanced"`.
- **`07-integrations/extensions-prompt-systeme.md`** — comment les outils d'extension arrivent jusqu'au modèle, `list_extensions` / `inspect_extensions`, découverte progressive et budget de contexte. La présente page ne traite pas de la sélection des outils envoyés au modèle.
- **`05-outils/mcp.md`** — l'outil MCP côté agent, utile parce qu'une extension peut appeler un connecteur MCP autorisé (`extension-api.mjs:138-146`).
- **`04-agent/`, page sur les modes de permission** — pour l'explication complète de « Accès complet », « Demande d'approbation » et du mode Plan, dont dépend le tableau des classes d'effet.

---

## Points à confirmer

1. **Aucun écran n'a été observé.** Tous les libellés de cette page — **Ajouter**, **Recharger**, **Mettre à jour**, **Ouvrir la source**, **Redémarrer l'hôte**, section **Hôte**, **Dernières erreurs de chargement** — viennent des clés de `src/i18n/fr.json`, pas d'une capture. Disposition, emplacement exact des diagnostics et forme de la demande d'approbation restent à vérifier lors de la passe d'interface.
2. **Écart relevé — champs inconnus dans le manifeste.** `EXTENSIONS.md` (l. 264-266) affirme qu'« un champ accepté par npm ou Node.js n'est pas automatiquement accepté par le manifeste Beaver ». Le code dit autre chose : la structure `ExtensionManifest` ne porte **pas** `deny_unknown_fields` (`types.rs:100-123`), donc un champ inconnu à la racine du manifeste est **silencieusement ignoré**. Seul le sous-objet `ui` est validé strictement (`types.rs:68-75`). À trancher : soit corriger `EXTENSIONS.md`, soit ne pas reprendre cette phrase sur le site. Ne pas écrire qu'un champ en trop refuse le manifeste — c'est faux à la racine, vrai dans `ui`.
3. **Écart de vocabulaire — « trois niveaux d'API ».** Le manifeste n'a que deux valeurs d'`apiLevel` (`stable`, `advanced`) et deux valeurs de `ui.mode` (`standard`, `advanced`). Il n'existe **pas** de niveau d'API nommé `standard`. Décider du vocabulaire du site : soit parler de deux niveaux d'API et deux modes d'interface, soit parler de trois « paliers » en assumant que ce n'est pas le vocabulaire du manifeste.
4. **Précision à trancher — le rôle de Jiti.** `EXTENSIONS.md` (l. 202) écrit que « Beaver utilise Jiti pour charger JavaScript ou TypeScript ». Le code réserve Jiti aux extensions de fichier non natives et aux replis (`module-loader.mjs:13-32`). La conclusion pratique est identique — pas de compilation obligatoire — mais la phrase mérite d'être reformulée.
5. **Le budget de 5 secondes des gestionnaires d'événements n'interrompt rien.** `EXTENSIONS.md` (l. 355) dit seulement que « son temps est borné ». Le code résout la course au lieu de la rejeter (`extension-api.mjs:206-216`) : le gestionnaire continue de s'exécuter sans surveillance. Formulation à valider avec le propriétaire : est-ce un comportement voulu qu'on documente, ou un point à corriger avant publication ?
6. **Occupants `beaver.*` non recensés.** La liste des identifiants utiles (`beaver.agent-local`, `beaver.heartbeat`, `beaver.tools`, `beaver.providers`…) vient de `EXTENSIONS.md`. Le fichier d'autorité `src/features/extension-ui/core-occupants.tsx` existe mais n'a pas été relu pour cette page. À faire avant publication : recopier la liste exacte depuis ce fichier, ou renvoyer le lecteur vers lui sans énumérer.
7. **Limite provider non revérifiée.** L'indisponibilité des outils d'extension pour les modèles Groq via OpenRouter et pour `groq/compound` est reprise de `EXTENSIONS.md` (§ Utilisation par le modèle) sans avoir été retrouvée dans le code. À vérifier, ou à traiter dans la page `extensions-prompt-systeme.md` qui couvre ce sujet.
8. **Mode sûr sous Linux Wayland.** `EXTENSIONS.md` indique que Beaver demande la touche Maj *après* l'apparition de la WebView sous Wayland. Seule la chaîne `--safe-mode` a été vérifiée (`ui_startup.rs:8`) ; la détection de la touche selon le système ne l'a pas été.
9. **Recette de test avant publication.** `EXTENSIONS.md` (§ Recette minimale avant publication) donne une liste de douze vérifications. Elle n'a pas été reprise ici pour ne pas doubler la page ; décider si elle mérite une section « Avant de publier » ou une page à part.
10. **Skills, ressources et résultats de fichier volontairement écartés.** `beaver.registerSkill`, `beaver.registerResource` et les résultats d'outil à blocs (texte + fichier) existent et ont été vérifiés dans le code (`extension-api.mjs:96-118`, `sdk/index.d.ts:31-53`), mais ils sortent du parcours « premier outil qui marche ». À décider : les ajouter en fin de page comme « pour aller plus loin », ou leur donner une page dédiée.
