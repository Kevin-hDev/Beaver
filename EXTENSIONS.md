# Extensions Beaver

> Guide utilisateur et auteur — état de l’implémentation au 6 septembre 2026.
>
> Ce guide décrit les extensions hébergées par Beaver. Les contrats JSON restent les
> autorités exécutables lorsque leur détail diffère d’un exemple humain.

Beaver peut charger des extensions personnalisées pour ajouter des outils à l’agent,
réagir aux événements de l’application et étendre son interface. Une extension peut
être un simple fichier local, un dossier, un dépôt Git ou un package npm.

Ce document explique le parcours utilisateur et les bases nécessaires pour créer une
extension. La [référence du SDK](./src-tauri/resources/extension-host/sdk/README.md)
et les contrats JSON générés restent les autorités techniques complètes :

- [contrat du runtime](./src-tauri/resources/extension-host/contract.json) ;
- [contrat de l’interface](./src-tauri/resources/extension-ui/contract.json).

## Navigation

- [Choisir le bon type de contribution](#choisir-rapidement)
- [Comprendre les risques et la compatibilité](#à-retenir-avant-de-commencer)
- [Créer le manifeste](#créer-le-manifeste)
- [Coder un premier outil](#coder-un-premier-outil)
- [Ajouter des skills, ressources et fichiers](#skills-ressources-et-résultats-de-fichier)
- [Étendre ou remplacer l’interface](#étendre-linterface)
- [Installer depuis un fichier, Git ou npm](#ajouter-lextension-à-beaver)
- [Comprendre la confiance et les permissions](#isolation-et-confiance)
- [Diagnostiquer et récupérer Beaver](#en-cas-de-problème)
- [Préparer une extension à distribuer](#préparer-une-extension-à-distribuer)
- [Consulter les limites actuelles](#limites-actuelles)

## Choisir rapidement

| Besoin                                                                     | Solution recommandée                                              |
| -------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Ajouter un ou plusieurs outils à l’Agent                                   | Extension `stable` avec `registerTool`                            |
| Fournir des instructions longues à charger seulement si nécessaire         | Skill déclaré par l’extension                                     |
| Fournir un modèle, une documentation, une image ou un fichier de référence | Ressource déclarée par l’extension                                |
| Produire un document, une image ou un autre fichier dans une conversation  | Résultat d’outil riche                                            |
| Ajouter un onglet, un réglage, un bouton ou un thème cohérent avec Beaver  | Interface `standard`                                              |
| Déplacer, remplacer ou retirer un emplacement public                       | Interface `standard` avec une opération de placement              |
| Modifier librement le DOM, le CSS ou une partie non exposée par le contrat | Interface `advanced`, après audit complet                         |
| Remplacer un outil natif explicitement remplaçable                         | API `advanced` instable                                           |
| Piloter Beaver depuis une application déjà lancée                          | Non disponible : les applications externes sont un autre chantier |

Une extension classique regroupe plusieurs de ces contributions dans le même dossier.
Il n’est pas nécessaire de créer un serveur, un port, un MCP ou une application séparée :
Beaver lance le point d’entrée dans son Hôte Node.js et lui fournit directement le SDK.

### Vocabulaire Beaver

- Un **Tool interne** appartient au cœur de Beaver. Ce n’est ni un plugin ni une
  extension installable.
- Un **plugin officiel** est une extension livrée et auditée avec Beaver, comme les
  plugins Office. Les plugins officiels actifs partagent l’Hôte officiel.
- Une **extension personnalisée** vient de l’utilisateur, d’un dépôt Git ou de npm.
  Chaque extension tierce active possède son propre Hôte.
- Un **MCP** est un protocole d’outils séparé. Une extension peut appeler un connecteur
  MCP autorisé, mais installer une extension ne crée pas un MCP.
- Une **application externe** est un programme autonome déjà lancé qui voudrait se
  connecter à Beaver. Ce protocole n’appartient pas au système décrit ici.

## À retenir avant de commencer

- Une extension personnalisée est du code local de confiance, exécuté avec les droits
  du compte utilisateur. Ce n’est pas une sandbox.
- Beaver protège son protocole, borne les échanges et isole chaque extension tierce
  dans son propre processus, mais il ne peut pas rendre un code malveillant inoffensif.
- L’utilisateur est responsable du code qu’il installe, de ses dépendances, de ses
  mises à jour et des secrets qu’il choisit de lui transmettre.
- Les outils d’extension sont disponibles en mode Agent. Le mode Chat classique
  conserve seulement ses fonctions de recherche et de lecture du Web. Les actions
  tierces du composeur y restent également masquées.
- Une application externe n’est pas une extension. Beaver charge une extension dans
  son runtime ; il ne fournit pas encore ici un protocole général pour connecter une
  application déjà lancée.

## Compatibilité

| Élément           | Prise en charge actuelle                                               |
| ----------------- | ---------------------------------------------------------------------- |
| Systèmes          | macOS Apple Silicon, Linux x64 et Windows x64                          |
| Langages d’entrée | JavaScript et TypeScript, modules JS/TS/JSX/TSX compris                |
| Runtime           | Node.js 20 minimum ; l’environnement Beaver actuel vise Node.js 24 LTS |
| API Beaver        | `beaverApi: "1"`                                                       |
| Interface         | API UI `1`, modes `standard` et `advanced`                             |
| Sources           | fichier local, dossier local, Git HTTPS/SSH et registre npm officiel   |

Ces systèmes sont les cibles de Beaver. Les parcours automatisés ont été validés sur les
trois systèmes, mais les recettes manuelles finales sur paquets installés Windows et
Linux restent suivies dans les checklists liées à la fin de ce guide. Une extension qui
utilise un module natif conserve sa propre matrice de compatibilité.

Le point d’entrée d’une extension hébergée doit être écrit en JavaScript ou TypeScript.
Comme ce code possède l’accès complet à Node.js, il peut appeler un programme écrit
dans un autre langage ou un service externe. L’auteur doit alors fournir ce programme,
vérifier sa présence et gérer sa compatibilité sur chaque système.

Une extension qui dépend d’un module natif, d’un exécutable ou d’un comportement propre
à un système n’est pas automatiquement multiplateforme. Testez-la sur chaque système
que vous annoncez compatible.

### Ce que Beaver isole — et ce qu’il n’isole pas

| Mécanisme                              | Garantie réelle                                                                         |
| -------------------------------------- | --------------------------------------------------------------------------------------- |
| Un processus Hôte par extension tierce | Un crash ordinaire n’arrête pas les autres extensions                                   |
| Identité attribuée par Rust            | Une extension ne peut pas choisir l’identité d’une autre sur le pont Beaver             |
| Environnement de processus minimal     | Les clés et variables du compte ne sont pas transmises automatiquement                  |
| Validation Node.js puis Rust           | Une contribution mal formée n’est pas publiée au modèle ou à l’interface                |
| Empreinte et approbation               | Un changement détecté demande une nouvelle approbation avant chargement                 |
| Accès complet à Node.js                | L’extension peut toujours lire des fichiers, lancer des processus et utiliser le réseau |
| Interface avancée dans la WebView      | Aucune sandbox : le module partage la page et les droits de l’interface Beaver          |

L’isolation améliore la stabilité et l’attribution des erreurs. Elle ne protège pas le
compte utilisateur contre un code qu’il a volontairement approuvé.

## Ce qu’une extension peut faire

Une extension peut notamment :

- déclarer jusqu’à 64 outils typés utilisables par le modèle ;
- lire le dossier de travail transmis à chaque appel d’outil ;
- écouter les événements publics de Beaver, actuellement `session.turn.started` ;
- consulter les sessions et les projets exposés par l’API ;
- lister les connecteurs MCP et appeler un de leurs outils ;
- consulter la configuration des canaux ;
- demander une clé de provider, un jeton OAuth MCP, une valeur MCP ou un jeton de
  canal appartenant à l’utilisateur ;
- ajouter des onglets, des réglages, des actions et des thèmes ;
- en mode avancé, remplacer certains points d’extension instables, par exemple un outil
  natif prévu pour être remplaçable ;
- utiliser les API Node.js pour lire ou écrire des fichiers, lancer des processus ou
  accéder au réseau.

Toutes les méthodes stables, les emplacements d’interface, les composants déclaratifs,
les jetons de thème et les limites sont listés dans la
[référence du SDK](./src-tauri/resources/extension-host/sdk/README.md).

## Skills, ressources et résultats de fichier

Une extension peut déclarer des skills et des ressources texte, image ou fichier. Leur
description n’est jamais ajoutée à toutes les conversations : l’agent doit d’abord
inspecter l’extension, puis charger explicitement le skill ou la ressource demandée.
Beaver vérifie alors l’extension approuvée, sa provenance et le chemin déclaré avant de
lire un contenu borné. Une modification ou une désactivation rend la lecture indisponible.

Un skill pointe vers un fichier nommé exactement `SKILL.md` ou `skill.md`.
Les ressources ordinaires peuvent porter d’autres noms. Les limites individuelles ne
s’additionnent pas sans limite : la réponse complète de chargement doit rester sous
le budget de message du contrat, enveloppe comprise, sinon l’enregistrement est refusé.

Un outil peut aussi retourner du texte et des fichiers relatifs au dossier de travail.
Beaver contrôle ces fichiers avant et pendant la lecture, applique le budget du lot et
conserve des métadonnées attribuées sans enregistrer les octets binaires. Un aperçu image
ne rejoint un fournisseur que si sa route le permet ; les autres routes reçoivent une
référence textuelle. Le mode Chat classique et les sous-agents ne reçoivent pas ces
capacités d’extension.

À l’ouverture d’une conversation, la vérification automatique des fichiers dispose
d’un budget total de 64 Mio. Chaque lecture réserve son coût maximal, y compris le
contrôle de dépassement. Les fichiers non examinés sont indiqués « Non vérifié » :
ce statut ne signifie ni « intact » ni « absent ». La réutilisation d’un aperçu par
le modèle conserve ses propres contrôles complets.

## Structure recommandée

Pour une extension distribuable, utilisez un dossier avec un manifeste explicite :

```text
hello-beaver/
├── beaver-extension.json
├── index.ts
├── package.json
└── package-lock.json
```

`package.json` et son verrou ne sont nécessaires que si l’extension utilise des
dépendances npm. Un fichier source isolé peut aussi être ajouté directement : Beaver
lui attribue alors une identité locale, la version `0.0.0` et le niveau d’API
`advanced`. Écrivez un manifeste explicite pour utiliser le niveau `stable` recommandé,
garder une identité stable, publier l’extension ou déclarer une interface.

Beaver reconnaît `beaver-extension.json`, `beaver.json` ou un bloc `beaver` dans
`package.json`.

### Préparer l’environnement de développement

Le SDK est injecté par Beaver au moment du chargement. Il ne faut donc pas embarquer
une seconde copie de `@beaver/sdk` dans le code distribué. Le dossier
[`src-tauri/resources/extension-host/sdk/`](./src-tauri/resources/extension-host/sdk/)
contient les types TypeScript, les exemples et le petit module utilisable comme
dépendance de développement locale.

Deux approches sont possibles :

- écrire directement une extension JavaScript sans compilation ;
- utiliser TypeScript et référencer le SDK local uniquement pour l’autocomplétion et
  le contrôle des types.

Beaver utilise Jiti pour charger JavaScript ou TypeScript, ESM ou CommonJS. Un outil de
compilation externe n’est donc pas obligatoire pour une extension simple. Si votre
extension possède sa propre compilation, le fichier indiqué par `main` doit exister et
rester sous la racine de l’extension au moment où Beaver la charge.

Pour l’autocomplétion hors du dépôt Beaver, utilisez les types du SDK provenant de la
même version de Beaver que celle ciblée, uniquement comme dépendance de développement.
L’import distribué reste `@beaver/sdk` : Beaver le résout lui-même dans l’Hôte au
chargement. Ne publiez ni une copie d’exécution du SDK ni un chemin absolu propre à
votre machine dans le package.

## Créer le manifeste

Exemple minimal de `beaver-extension.json` :

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

Règles principales :

- `id` doit être stable et unique. Un nom de domaine inversé réduit les collisions ;
- `version` décrit la version de l’extension ;
- `main` est un chemin relatif qui reste dans le dossier de l’extension ;
- `runtime` vaut actuellement `node` pour une extension personnalisée ;
- `access` vaut `full`, car du code Node.js local ne constitue pas une frontière de
  sécurité ;
- `apiLevel: "stable"` est le choix recommandé ;
- `apiLevel: "advanced"` donne accès aux surfaces instables et doit être utilisé
  seulement quand elles sont réellement nécessaires ;
- `essential: true` demande que les schémas du plugin soient chargés tôt lors de la
  découverte progressive. Cette priorité est bornée et ne remplace pas le choix de
  priorité effectué par l’utilisateur.

| Champ         |      Obligatoire | Rôle                                                        |
| ------------- | ---------------: | ----------------------------------------------------------- |
| `id`          |              Oui | Identité canonique stable de l’extension                    |
| `name`        |              Oui | Nom affiché à l’utilisateur                                 |
| `version`     |              Oui | Version déclarée de l’extension                             |
| `beaverApi`   |              Oui | Version du contrat, actuellement `1`                        |
| `runtime`     |              Oui | Runtime hébergé, actuellement `node`                        |
| `main`        | Oui avec Node.js | Point d’entrée JavaScript ou TypeScript relatif             |
| `access`      |              Non | `full` par défaut et valeur actuelle des extensions Node.js |
| `apiLevel`    |              Non | `stable` par défaut dans un manifeste, ou `advanced`        |
| `essential`   |              Non | Priorité bornée lors de la réduction du catalogue           |
| `description` |              Non | Résumé visible et utilisé pour comprendre l’extension       |
| `author`      |              Non | Auteur déclaré                                              |
| `homepage`    |              Non | Page d’information déclarée                                 |
| `ui`          |              Non | Contrat d’interface `standard` ou point d’entrée `advanced` |

Tous les identifiants et textes sont bornés. Utilisez des chaînes courtes et du JSON
strict. Un champ accepté par npm ou Node.js n’est pas automatiquement accepté par le
manifeste Beaver.

Le même manifeste peut vivre dans `package.json` :

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

## Coder un premier outil

Le module `@beaver/sdk` est fourni par Beaver au chargement. Il n’a pas besoin d’être
embarqué dans le package exécuté.

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

Beaver transforme le nom public en `com.example.hello.hello`. Donnez à chaque outil un
nom court, une description précise, un schéma JSON strict et la bonne classe d’effet.
Ces informations aident le modèle à choisir correctement l’outil et permettent à
Beaver d’appliquer la politique de permission adaptée.

### Cycle de vie du module

Beaver importe le module puis appelle `activate(beaver)`. Enregistrez les outils,
événements, skills, ressources et contributions standard pendant cette activation.

La fonction facultative `deactivate()` doit arrêter les timers, fermer les fichiers,
retirer les écouteurs et libérer les ressources appartenant à l’extension. Les fonctions
retournées par `beaver.on`, `beaver.ui.register` et `beaver.ui.onAction` servent aussi
au nettoyage et doivent pouvoir être appelées sans danger une seconde fois.

Une extension locale est reconstruite avec **Recharger**. Une extension Git ou npm est
reconstruite lors d’une mise à jour gérée. Les contributions enregistrées après la fin
du chargement ne modifient pas silencieusement le catalogue courant : un rechargement de
l’Hôte rend le nouvel ensemble visible.

### Événement disponible

Le seul événement public actuel est `session.turn.started`. Son objet contient
`sessionId` et `mode`. Un gestionnaire ne doit pas bloquer le tour : son temps est borné
et une panne du gestionnaire est attribuée à son extension.

```ts
const unsubscribe = beaver.on("session.turn.started", async (event) => {
  const { sessionId, mode } = event as {
    sessionId: string;
    mode: string;
  };

  void sessionId;
  void mode;
});
```

Conservez la fonction `unsubscribe` et appelez-la pendant `deactivate()`.

### Détecter les capacités facultatives

L’API reste `1`, mais les ajouts compatibles comme les skills, ressources et résultats
riches sont annoncés dans `beaver.capabilities`. Une extension récente doit vérifier à
la fois la capacité et la présence de la méthode afin de pouvoir se dégrader proprement
sur un ancien Beaver :

```ts
if (
  beaver.capabilities?.includes("skills") &&
  beaver.capabilities?.includes("resources") &&
  beaver.registerSkill &&
  beaver.registerResource
) {
  beaver.registerSkill({
    id: "guide",
    name: "Guide",
    description: "Méthode complète d’utilisation de l’extension.",
    path: "skills/guide/SKILL.md",
  });

  beaver.registerResource({
    id: "reference",
    name: "Référence",
    description: "Données de référence de l’extension.",
    type: "text",
    path: "resources/reference.txt",
  });
}
```

Le chemin d’un skill se termine obligatoirement par `SKILL.md` ou `skill.md`. Les
ressources acceptent les types `text`, `image` et `file`. Ces chemins sont relatifs à la
racine attribuée à l’extension ; les chemins absolus, sorties par `..`, liens dangereux
et fichiers hors limites sont refusés.

### Retourner des fichiers et des aperçus

Quand `richToolResults` est disponible, un outil peut retourner des blocs texte et
fichier. Le chemin d’un résultat est relatif au dossier de travail transmis à l’outil :

```ts
return {
  content: [
    { type: "text", text: "Le rapport est prêt." },
    {
      type: "file",
      path: "rapport.pdf",
      purpose: "artifact",
      displayName: "rapport.pdf",
    },
    {
      type: "file",
      path: "graphique.png",
      purpose: "preview",
      displayName: "graphique.png",
    },
  ],
};
```

`artifact` affiche un fichier produit. `preview` autorise un aperçu image uniquement si
la route du modèle le permet. Une erreur d’outil ne peut pas publier de fichier. Beaver
revalide taille, type, chemin et empreinte avant de lire ou réutiliser le résultat.

### Utiliser les services exposés par Beaver

Le SDK fournit des méthodes typées pour :

- `beaver.info()` ;
- `beaver.sessions.list()` et `beaver.sessions.get(id)` ;
- `beaver.projects.list()` ;
- `beaver.mcp.listConnectors()` et `beaver.mcp.callTool(...)` ;
- `beaver.channels.getConfig()` ;
- `beaver.secrets.getProviderKey(...)`, les jetons MCP et les jetons de canaux.

`beaver.call(method, params)` expose le même pont stable de plus bas niveau. N’inventez
pas un nom de méthode : seules les méthodes du contrat du runtime sont acceptées.

Les erreurs du pont sont des `BeaverExtensionError` bornées. Elles fournissent `reason`,
`code` et `retryable`. Retentez seulement une erreur explicitement retentable, avec un
nombre d’essais et une attente bornés :

```ts
import { isBeaverExtensionError } from "@beaver/sdk";

try {
  const sessions = await beaver.sessions.list();
  void sessions;
} catch (error) {
  if (isBeaverExtensionError(error) && error.retryable) {
    // Reporter ou retenter avec une politique bornée propre à l’extension.
  } else {
    throw error;
  }
}
```

Ne placez jamais une clé ou un jeton dans un log, une erreur, un résultat d’outil ou un
fichier de diagnostic. Une fois remis au JavaScript, un secret ne peut plus être
zéroïsé immédiatement de manière garantie par Beaver.

## Déclarer l’effet d’un outil

| Effet            | Usage typique                        | Comportement en mode manuel                 |
| ---------------- | ------------------------------------ | ------------------------------------------- |
| `read-only`      | Lecture locale sans effet externe    | Pas de confirmation ; autorisé en mode Plan |
| `external-read`  | Lecture réseau ou service distant    | Confirmation ; refusé en mode Plan          |
| `local-write`    | Écriture sur la machine              | Confirmation ; refusé en mode Plan          |
| `external-write` | Modification d’un service distant    | Confirmation ; refusé en mode Plan          |
| `process`        | Lancement ou contrôle d’un processus | Confirmation ; refusé en mode Plan          |
| `secret`         | Accès à un secret                    | Confirmation ; refusé en mode Plan          |
| `unknown`        | Effet absent ou impossible à classer | Bloqué par défaut comme action sensible     |

Le mode automatique peut autoriser une action sans dialogue. Un effet absent ou
invalide n’est pas refusé : Beaver enregistre l’outil avec la classe `unknown`, la plus
restrictive. Déclarez toujours la classe réelle afin de documenter le comportement et
de protéger correctement les modes plus restrictifs.

## Étendre l’interface

### Mode standard

Ajoutez ceci au manifeste :

```json
{
  "ui": {
    "apiVersion": "1",
    "mode": "standard"
  }
}
```

Le point d’entrée principal peut ensuite déclarer des onglets, réglages, actions ou
thèmes avec `beaver.ui.register(...)`. Il décrit l’interface sous forme de données et
Beaver la rend avec ses propres composants. C’est le mode recommandé : il conserve le
design, les traductions, les limites et la validation de Beaver.

Le mode standard n’accepte pas de champ `ui.entry` : toute sa contribution passe par le
SDK de l’Hôte et reste une donnée validée, jamais du JavaScript injecté dans la WebView.

Les vues standard disposent de textes localisés, de rangées, piles, titres, badges,
séparateurs, champs texte ou numériques, sélecteurs, interrupteurs et boutons. Un
bouton appelle une action enregistrée avec `beaver.ui.onAction(...)`.

### Emplacements et composition modulaire

Une contribution standard ne modifie pas directement un composant React. Beaver et les
extensions publient des occupants dans le même registre d’emplacements, puis Beaver
calcule la vue finale. Désactiver l’extension retire sa contribution et fait revenir
l’occupant Beaver remplacé ou déplacé.

Les emplacements publics actuels sont :

| Emplacement                        | Contribution acceptée           | Portée                        |
| ---------------------------------- | ------------------------------- | ----------------------------- |
| `app.navigation.primary`           | Onglet principal                | Globale                       |
| `settings.navigation.preferences`  | Onglet de réglages Préférences  | Globale                       |
| `settings.navigation.agent`        | Onglet de réglages Agent        | Globale                       |
| `settings.navigation.models`       | Onglet de réglages Modèles      | Globale                       |
| `settings.navigation.integrations` | Onglet de réglages Intégrations | Globale                       |
| `settings.navigation.application`  | Onglet de réglages Application  | Globale                       |
| `app.toolbar.primary`              | Action de la barre supérieure   | Globale                       |
| `agent.composer.leading`           | Action près du menu `+`         | Conversation Agent uniquement |

Chaque contribution possède `id`, `placement`, `order` et, si elle vise un occupant,
`operation` avec `targetId` :

| Opération | Effet                                                         |
| --------- | ------------------------------------------------------------- |
| Absente   | Ajoute la contribution selon son ordre                        |
| `before`  | Ajoute la contribution avant l’occupant compatible ciblé      |
| `after`   | Ajoute la contribution après l’occupant compatible ciblé      |
| `replace` | Remplace entièrement l’occupant ciblé par la contribution     |
| `move`    | Déplace l’occupant non protégé vers un emplacement compatible |
| `remove`  | Retire l’occupant non protégé                                 |

Exemple : ajouter un onglet après Réveils sans modifier son composant :

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

Les identifiants `beaver.*` des occupants actuels sont définis par Beaver. Les plus
utiles pour le placement sont notamment `beaver.agent-local`, `beaver.heartbeat`,
`beaver.personality`, `beaver.settings`, les onglets de réglages comme `beaver.tools`,
`beaver.providers` et `beaver.extensions`, ainsi que les actions de barre préfixées
`beaver.toolbar.`. Consultez l’autorité courante dans
[`core-occupants.tsx`](./src/features/extension-ui/core-occupants.tsx) avant de publier
une extension qui dépend d’un occupant Beaver précis.

Deux protections restent non négociables en mode standard : `beaver.settings` ne peut
pas être retiré ou remplacé dans la navigation principale, et `beaver.extensions` ne
peut pas être retiré ou remplacé des réglages. Elles garantissent que l’utilisateur peut
revenir en arrière.

L’ordre final est déterministe : `order`, puis identifiant d’extension, puis identifiant
de contribution. Si plusieurs extensions déplacent, retirent ou remplacent le même
occupant, Beaver refuse les mutations conflictuelles et conserve l’original. Une cible
absente ou incompatible produit un diagnostic sans supprimer les contributions saines.
Pour modifier seulement une partie interne d’un composant Beaver, il faut soit demander
un nouvel emplacement public dans le contrat, soit utiliser le mode avancé. Le contrat
standard ne donne jamais accès aux composants React internes ni à Tauri `invoke`.

Les textes utilisent toujours `{ default: "..." }`, puis facultativement `fr`, `en`,
`es`, `de`, `it`, `zh` et `ja`. Beaver choisit la langue active et utilise `default`
comme repli. Une action reçoit les valeurs bornées des champs de sa vue et la langue,
puis retourne une notification ou une nouvelle vue :

```ts
const stopAction = beaver.ui.onAction("save", async ({ fields }, context) => ({
  type: "notification",
  level: "success",
  message: {
    default: `Saved ${String(fields.name ?? "")} (${context.locale})`,
    fr: `Enregistré : ${String(fields.name ?? "")}`,
  },
}));
```

Un thème standard ne fournit ni CSS ni sélecteur. Il redéfinit uniquement les jetons
publics du contrat avec une couleur hexadécimale complète :

```ts
const unregisterTheme = beaver.ui.register({
  type: "theme",
  id: "night-blue",
  order: 0,
  label: { default: "Night blue", fr: "Bleu nocturne" },
  base: "dark",
  tokens: {
    "--surface": "#101827",
    "--ink": "#F8FAFC",
  },
});
```

Un jeton inconnu ou une valeur invalide refuse l’ensemble des contributions d’interface
de l’extension, thème compris, avec un diagnostic. Conservez les fonctions
`unregisterTab`, `stopAction` et `unregisterTheme` pour le nettoyage.

### Mode avancé

Le mode avancé charge un module JavaScript arbitraire dans la même WebView que Beaver.
Il permet de manipuler le DOM et de monter une interface personnalisée dans les
emplacements publics.

```json
{
  "apiLevel": "advanced",
  "ui": {
    "apiVersion": "1",
    "mode": "advanced",
    "entry": "./ui.ts"
  }
}
```

`ui.entry` est obligatoire en mode avancé, relatif à la racine et limité aux entrées
JavaScript/TypeScript reconnues par le contrat UI. `apiLevel` doit également être
`advanced` ; mélanger un niveau stable et une interface avancée refuse le manifeste.

Ce mode n’est pas une sandbox. Une confirmation supplémentaire est demandée à
l’activation. Beaver compile et empreinte l’artefact, refuse un artefact modifié et
retire les montages lors d’un rechargement ou d’une désactivation, mais l’auteur reste
responsable de tout ce que son module exécute dans la WebView.

Le module exporte `activate(context)`. `context.mount(emplacement, callback)` fournit un
conteneur appartenant à Beaver. Le callback peut retourner une fonction de nettoyage ;
`activate` peut également en retourner une et le module peut exporter `deactivate`.
Si le module ne monte volontairement rien, il appelle `completeWithoutMounts()`.

```ts
import type { BeaverAdvancedUiModule } from "@beaver/sdk";

export const activate: BeaverAdvancedUiModule["activate"] = (context) => {
  context.mount("app.toolbar.primary", (container) => {
    const button = document.createElement("button");
    button.textContent = "Dashboard";
    container.append(button);

    return () => button.remove();
  });
};
```

Puisqu’il partage la WebView, ce code peut aussi chercher, masquer, déplacer ou remplacer
des éléments du DOM et injecter du CSS. Ces manipulations ne forment pas une API stable :
React peut recréer un élément et une mise à jour de Beaver peut changer sa structure.
Préférez toujours `context.mount` et les emplacements publics lorsqu’ils suffisent.

Utilisez le mode avancé seulement si le contrat standard ne permet pas l’interface
voulue.

### Remplacer un outil natif

Une extension dont le manifeste déclare `apiLevel: "advanced"` peut utiliser
`beaver.unstable.registerReplacement(...)` avec le nom d’un outil du catalogue natif
pour le recouvrir. La définition reprend le même format qu’un outil ordinaire :

```ts
beaver.unstable.registerReplacement({
  name: "web_search",
  description: "Recherche Web personnalisée.",
  parameters: {
    type: "object",
    properties: {
      query: { type: "string" },
    },
    required: ["query"],
    additionalProperties: false,
  },
  effect: "external-read",
  async execute({ query }) {
    return `Recherche demandée : ${String(query)}`;
  },
});
```

Beaver ne maintient pas de liste blanche de points de remplacement. Si le nom correspond
à un outil du catalogue natif courant, la définition de l’extension le recouvre. Sans
correspondance, elle devient actuellement un nouvel outil avancé non préfixé. Ce
comportement est instable et peut changer entre versions.

Un remplacement respecte le réglage d’activation de l’outil natif correspondant. En
revanche, sa propre classe d’effet détermine les confirmations, comme pour tout outil
d’extension. Le niveau `advanced` ne donne pas automatiquement accès à une méthode du
cœur absente du contrat. `beaver.unstable.call(...)` existe pour les futures méthodes
déclarées avancées, mais le contrat actuel n’en publie aucune : ce n’est pas une porte
d’accès arbitraire au backend.

## Ajouter l’extension à Beaver

| Source                   | Stockage et mise à jour                                                                                 | Suppression                              |
| ------------------------ | ------------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| Fichier ou dossier local | Beaver charge la source choisie ; l’auteur applique ses changements avec **Recharger**                  | La source de l’utilisateur reste intacte |
| Git                      | Beaver copie une révision validée dans son stockage géré ; **Mettre à jour** prépare une nouvelle copie | Beaver retire uniquement sa copie gérée  |
| npm                      | Beaver installe le package et ses dépendances de production dans son stockage géré                      | Beaver retire uniquement sa copie gérée  |

L’installation connecte l’extension à Beaver. Aucun port, jeton d’appairage ou serveur
local supplémentaire n’est nécessaire. L’extension reste désactivée et non approuvée
après son ajout : installation et confiance sont deux décisions séparées.

Dans Beaver :

1. Ouvrez **Réglages → Extensions → Extensions**.
2. Cliquez sur **Ajouter**.
3. Choisissez une source :
   - un fichier JavaScript ou TypeScript ;
   - un dossier contenant le manifeste ;
   - une URL Git HTTPS ou SSH, éventuellement suivie de `#branche`, `#tag` ou
     `#commit` ;
   - un package npm, par exemple `@example/hello-beaver` ou
     `hello-beaver@latest`.
4. Ouvrez la fiche créée, vérifiez la source et les contributions annoncées.
5. Activez l’extension et confirmez que vous faites confiance à son code. Le mode UI
   avancé demande une confirmation explicite supplémentaire.

Une extension locale reste liée au fichier ou au dossier choisi. Pour appliquer vos
modifications, utilisez **Recharger**. Les installations Git et npm sont copiées dans
le stockage géré par Beaver et disposent d’une action **Mettre à jour**.

L’ajout et les mises à jour s’exécutent en arrière-plan. Après le lancement, vous
pouvez fermer la fenêtre d’ajout et continuer à naviguer. Le bouton de suivi dans la
barre supérieure reste accessible lorsque la barre latérale est repliée ; la page
Extensions affiche les mêmes installations. Fermer ce suivi ne les annule pas.
**Annuler** demande l’arrêt du travail et le nettoyage de ses fichiers temporaires ;
Beaver confirme l’annulation après cet arrêt. Vos sources locales sont conservées.
Une installation réussie ne change pas automatiquement votre écran et n’approuve
pas le code : son activation reste une décision séparée.

Au-delà de 1 Gio occupé par une installation gérée, Beaver arrête les écritures et
demande **Continuer l’installation** ou **Annuler** dans le suivi. Les caches et
fichiers temporaires comptent aussi. Cette confirmation n’est pas un plafond à
configurer : elle autorise la poursuite dans l’espace disponible, en conservant une
réserve de 1 Gio. Si l’espace devient insuffisant, le travail s’arrête. Il s’agit
d’une surveillance pratique, pas d’un quota imposé par le système d’exploitation :
des écritures peuvent dépasser brièvement le seuil entre deux mesures.

Une seule installation travaille à la fois. Une confirmation laissée sans réponse
reste en attente ; les installations suivantes en indiquent la raison et permettent
d’afficher la demande. Aucun délai ne vaut accord. Un pourcentage n’est montré que
si le total est connu ; sinon Beaver affiche l’étape en cours.

Après une interruption, aucune installation ne reprend automatiquement.
**Reprendre l’installation** apparaît seulement lorsqu’un point de reprise peut
être revérifié ; sinon **Réessayer** lance une nouvelle opération. Un journal de
récupération illisible bloque les nouvelles installations et conserve les fichiers
concernés : il nécessite une intervention, sans empêcher le reste de Beaver de fonctionner.

Si l’enregistrement du journal ou le redémarrage du travail échoue, Beaver restaure
l’état précédent du job : une reprise refusée ne devient jamais exécutable en silence.
Si le travailleur s’arrête pendant une annulation, l’opération finit en échec avec son
point de reprise conservé, plutôt que d’afficher une fausse annulation réussie. Une
annulation n’est confirmée qu’après l’arrêt du processus possédé et le nettoyage des
fichiers dont Beaver est responsable.

Après une mise à jour ou un changement des fichiers couverts par l’empreinte, Beaver
révoque l’approbation précédente, désactive l’extension et demande une nouvelle
confirmation. Ce contrôle porte sur les fichiers sélectionnés par l’empreinte au
moment de la vérification : sources JavaScript/TypeScript, manifeste et artefact UI.
Il exclut notamment `node_modules` et ne fige pas les fichiers ouverts ensuite par
Node. Il ne garantit donc pas qu’une modification locale d’une dépendance ou un
remplacement entre vérification et import demandera une nouvelle approbation.

## Dépôts Git

Beaver accepte une URL Git HTTPS ou SSH validée. HTTP non chiffré et les identifiants
placés directement dans une URL sont refusés. Une branche, un tag, une empreinte complète
ou une référence abrégée hexadécimale suffisamment longue peut être sélectionnée.

Beaver fixe la révision choisie avant la publication, ignore les sous-modules, retire les
métadonnées `.git` de la copie gérée et valide le manifeste avant de modifier le registre.
Une mise à jour dont l’identité d’extension change est refusée.

Les budgets Git bornent les transferts et les fichiers, mais ne constituent pas encore
un délai mural capable d’interrompre immédiatement toutes les phases bloquantes d’un
client Git. Une annulation demande l’arrêt de l’arbre du processus ; Beaver ne publie pas
la copie tant que cet arrêt et son état final ne sont pas établis.

## Dépendances npm

Pour un dossier local, installez vous-même les dépendances de production dans ce
dossier avant de l’ajouter à Beaver.

Pour une installation Git ou npm gérée, Beaver installe uniquement les dépendances de
production avec son npm et le registre officiel HTTPS. Il désactive notamment :

- les scripts de cycle de vie npm ;
- la création de liens exécutables ;
- les workspaces ;
- les configurations npm du dépôt ou de la machine.

Beaver conserve le cache pendant une attente de confirmation. Il verrouille les
dépendances avant leur installation et revérifie ce verrouillage avant de relancer
une étape interrompue ; une modification le fait échouer. Git conserve de même la
révision choisie pendant l’attente. Cette reprise peut refaire une étape et ne
promet pas de continuer exactement au dernier octet téléchargé.

Précompilez donc les dépendances qui exigent un script d’installation, ou distribuez
une extension déjà prête à exécuter. Vous pouvez aussi ajouter localement un dossier
que vous avez préparé et audité vous-même.

## Utilisation par le modèle

Activer une extension rend ses outils disponibles au mode Agent. **Afficher dans le
chat** ne change pas les capacités du modèle : ce réglage ajoute seulement un raccourci
d’activation dans le menu `+` du composeur Agent. La liste montre huit lignes avant de
devenir défilable ; huit n’est pas une limite d’extensions.

Tant que les définitions des plugins occupent au plus 10 % de la fenêtre de contexte,
Beaver conserve les plugins activés dans le catalogue envoyé au modèle. Au-delà,
Beaver passe à la découverte progressive : les plugins choisis comme prioritaires,
ceux marqués essentiels et ceux inspectés sont chargés en premier. Le nom et
l’identifiant de chaque extension active et approuvée restent visibles dans une
section dédiée de la description de `list_extensions`. Le modèle peut lister les
extensions puis appeler `inspect_extensions` avec plusieurs identifiants à la fois.
L’utilisateur n’a pas à sélectionner manuellement des outils à chaque requête.

Les limites propres à un provider restent applicables. Actuellement, les modèles Groq
utilisés via OpenRouter ne reçoivent pas les outils d’extension ; les modèles
`groq/compound` ne reçoivent aucun outil. Beaver affiche cette indisponibilité dans la
conversation.

## Isolation et confiance

Les plugins officiels partagent un Hôte Beaver audité. Chaque extension tierce activée
dispose de son propre processus Hôte, de sa propre identité et de son propre domaine de
panne. Le crash d’une extension ne doit donc pas arrêter les autres extensions.

Cette isolation améliore la stabilité, pas la sécurité contre l’utilisateur lui-même :
une extension tierce peut toujours accéder aux fichiers, au réseau, aux processus et
aux secrets que son code demande. Ne l’activez que si vous acceptez ce comportement.

L’interface normale de Beaver ne reçoit pas les clés du coffre. Une extension
explicitement approuvée peut demander directement les secrets via l’API du SDK :
cette approbation porte sur tout son code et ses dépendances, sans permission séparée
par clé. Cet accès permet aux extensions de travailler avec les services choisis par
l’utilisateur ; il ne constitue pas une garantie de confinement du code installé.

Beaver enregistre l’autorisation d’accès sans écrire la clé ni les paramètres de la
demande dans le journal. Si cet enregistrement échoue, le secret n’est pas remis.
Le journal atteste l’autorisation, pas la réception effective de la réponse.
Désactiver ou supprimer une extension ne retire pas une copie qu’elle aurait déjà
obtenue : pour invalider cette copie, révoquez l’accès auprès du fournisseur concerné.

Pour une interface avancée, Beaver retire ses conteneurs et ses styles avant
d’attendre les callbacks de nettoyage de l’extension. Cette attente partage un
budget global borné ; son expiration libère les chargements suivants, mais n’annule
pas le JavaScript tiers. Une boucle synchrone bloquant la page principale nécessite
toujours un redémarrage, éventuellement en mode sûr.

Beaver borne notamment le nombre de fichiers, la taille de l’empreinte, les messages,
les requêtes en vol, le temps d’exécution et les redémarrages automatiques. Les limites
actuelles incluent 128 extensions utilisateur enregistrées, 64 outils par extension,
256 outils au total et 31 processus tiers actifs, le dernier emplacement étant réservé
à l’Hôte officiel. Consultez les contrats générés pour les valeurs exhaustives.

## En cas de problème

Procédez dans cet ordre :

1. Ouvrez **Réglages → Extensions → Hôte** et lisez le diagnostic localisé.
2. Ouvrez la fiche de l’extension pour vérifier son état, sa source et sa dernière
   erreur.
3. Utilisez **Ouvrir la source** pour auditer le code réellement chargé.
4. Utilisez **Recharger** après une correction locale, ou **Mettre à jour** pour une
   installation Git/npm.
5. Désactivez l’extension. Son processus et ses permissions de session sont alors
   révoqués.
6. Redémarrez l’Hôte depuis son panneau si le runtime lui-même est en erreur.
7. Retirez l’extension si vous ne lui faites plus confiance.

| Symptôme                                                     | Explication probable                                                                                       | Action                                                                                           |
| ------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| Extension ajoutée mais absente du modèle                     | Elle est désactivée, non approuvée, masquée par la limite du fournisseur ou non inspectée au-delà des 10 % | Activer, vérifier la fiche, puis utiliser `list_extensions` et `inspect_extensions` dans l’Agent |
| Skill ou ressource indisponible                              | Extension non inspectée, désactivée, modifiée ou fichier refusé                                            | Réinspecter l’extension et vérifier son chemin relatif                                           |
| Installation en attente                                      | Une confirmation de volume ou le job précédent bloque la file                                              | Ouvrir le suivi global et répondre à la demande ; aucun délai ne vaut accord                     |
| Installation interrompue                                     | Un point de reprise existe peut-être, mais Beaver ne redémarre jamais le travail seul                      | Utiliser **Reprendre l’installation** si proposé, sinon **Réessayer**                            |
| Interface cassée au lancement                                | Module avancé ou chargement UI interrompu                                                                  | Démarrer sans UI tierce avec Maj ou `--safe-mode`, puis désactiver l’extension                   |
| Extensions indisponibles mais conversation encore utilisable | Registre ou état de conversation refusé ; Beaver a conservé uniquement les outils natifs sûrs              | Lire l’avertissement, préserver les fichiers et redémarrer ou mettre Beaver à jour               |
| Un Hôte redémarre en boucle                                  | L’extension plante pendant son activation ou ses appels                                                    | Lire le diagnostic, corriger puis recharger ; le budget automatique est limité                   |
| Secret potentiellement divulgué                              | Le code approuvé a pu conserver une copie                                                                  | Désactiver l’extension puis révoquer immédiatement le secret chez son fournisseur                |

Si Beaver s’est arrêté pendant le chargement d’une extension, il affiche au prochain
démarrage un parcours de récupération. Vous pouvez garder l’extension désactivée,
retenter son chargement ou restaurer l’état précédent.

Si `extensions.json` vient d’une version future, contient un type d’extension inconnu,
est tronqué ou ne peut pas être lu, Beaver refuse le registre entier avant migration ou
nettoyage. Il ne le remplace pas par un registre vide et ne supprime pas les fichiers
qu’il ne comprend pas. Les mutations d’extensions sont bloquées jusqu’au retour à un
registre compatible.

Dans une conversation Agent, cette indisponibilité retire les extensions, leurs
remplacements, leur découverte, leurs skills et leurs ressources. Beaver conserve
uniquement les outils natifs encore admissibles pour permettre à la conversation de
continuer avec un avertissement traduit. Les outils dynamiques inconnus, notamment les
outils MCP, ne sont pas ajoutés par ce repli. Si le fichier de la conversation elle-même
ne peut plus être enregistré, le tour s’arrête : Beaver n’exécute pas un outil sans
pouvoir conserver sa trace.

Une préférence de découverte enregistrée dont le catalogue ne peut plus être reconstruit
rend également l’ancien catalogue indisponible. Beaver préfère fermer les extensions et
réessayer au prochain démarrage plutôt que servir une configuration qui ne correspond
plus au disque.

Si une interface avancée empêche l’application de s’ouvrir correctement, quittez
complètement Beaver puis maintenez **Maj** pendant son redémarrage. Vous pouvez aussi
lancer l’exécutable Beaver avec l’argument exact `--safe-mode`. Ce lancement n’affiche
aucune interface tierce et laisse l’écran Extensions disponible pour désactiver le
module fautif. Sous Linux Wayland, Beaver demande la touche Maj après l’apparition de
la WebView, car le système ne permet pas la même détection native qu’ailleurs.

Les sorties brutes d’une extension ne sont pas conservées : elles peuvent contenir des
secrets. Les diagnostics indiquent l’étape, la catégorie et la position sûre quand elle
est disponible, sans exposer la sortie complète du code tiers.

### Informations utiles pour signaler un défaut

Joignez uniquement :

- version de Beaver, système et architecture ;
- type de source : locale, Git ou npm ;
- identifiant public de l’extension et étape affichée ;
- action qui reproduit le problème ;
- diagnostic localisé et extrait de journal expurgé ;
- confirmation que le problème disparaît ou non en désactivant l’extension ou en mode
  sûr.

Ne publiez jamais de clé, jeton, paramètres d’appel secrets, chemin personnel complet,
contenu privé de session ou sortie brute non relue d’une extension.

## Préparer une extension à distribuer

### Recette minimale avant publication

Testez l’extension depuis un profil Beaver jetable, jamais depuis le seul fichier source :

1. ajoutez-la localement et vérifiez qu’elle reste désactivée et non approuvée ;
2. lisez la fiche, activez-la et vérifiez la demande de confiance ;
3. exécutez chaque outil avec arguments valides, invalides, annulation et délai ;
4. vérifiez ses classes d’effet en modes automatique, manuel et Plan ;
5. confirmez son absence complète en mode Chat classique ;
6. inspectez puis chargez chaque skill et ressource ;
7. vérifiez chaque fichier normal, absent, modifié et trop grand ;
8. désactivez, réactivez, rechargez puis redémarrez Beaver ;
9. faites échouer volontairement l’activation et confirmez que les autres extensions
   continuent de fonctionner ;
10. testez l’interface au clavier, à largeur étroite et dans les six thèmes Beaver ;
11. si l’interface est avancée, cassez-la et vérifiez le démarrage avec Maj ou
    `--safe-mode` ;
12. installez enfin la même archive réelle depuis Git ou npm, car un dossier local ne
    prouve pas le parcours distribué.

Pour une contribution directe au dépôt Beaver, les fixtures et tests de l’Hôte vivent
dans `scripts/extensions/` et les fixtures d’acceptation dans `src-tauri/tests/fixtures/`.
Les commandes du dépôt sont notamment `npm run test:extensions-host`,
`npm run test:extensions-runtime-smoke` et `npm run contracts:check`. Un auteur externe
reste responsable de la suite de tests de son propre package.

Avant de publier :

- gardez un `id` stable et incrémentez `version` ;
- déclarez uniquement l’API et les effets réellement utilisés ;
- fournissez un schéma JSON strict et une description utile pour chaque outil ;
- testez les résultats normaux, les erreurs, les annulations et les délais ;
- vérifiez la désactivation et le nettoyage de tous les écouteurs ou montages ;
- testez chaque système annoncé compatible ;
- verrouillez et auditez les dépendances ;
- n’enregistrez jamais de secret dans le dépôt, les messages d’erreur ou les logs ;
- documentez clairement les fichiers, commandes, réseaux et secrets utilisés ;
- essayez l’installation depuis la même source que vos futurs utilisateurs ;
- vérifiez l’interface dans chaque thème Beaver si vous fournissez une contribution UI.

Les fixtures hors ligne dans
[`scripts/extensions/fixtures/ui/`](./scripts/extensions/fixtures/ui/) montrent des
contributions standard, avancées, localisées, limitées et volontairement invalides.
Elles servent de référence testable avec le SDK et les contrats générés.
La fixture d’acceptation complète des skills, ressources et résultats se trouve dans
[`src-tauri/tests/fixtures/extensions/api-expansion/`](./src-tauri/tests/fixtures/extensions/api-expansion/).

## Limites actuelles

- Le runtime hébergé accepte JavaScript et TypeScript, pas directement Python, Rust,
  Go, Java ou C#.
- Les événements publics sont encore limités à `session.turn.started`.
- L’API avancée est instable et peut changer entre deux versions de Beaver.
- Il n’existe pas encore de marketplace intégrée pour découvrir des extensions.
- Le protocole pour connecter des applications externes est un chantier distinct et
  n’est pas inclus dans l’API d’extension actuelle.
- Beaver ne vérifie pas la qualité fonctionnelle d’une extension tierce. La validation
  du manifeste et du protocole ne remplace pas un audit de son comportement.

### Limites principales du contrat actuel

| Ressource                            |                                                   Limite actuelle |
| ------------------------------------ | ----------------------------------------------------------------: |
| Extensions utilisateur enregistrées  |                                                               128 |
| Hôtes simultanés                     |                         32, dont un réservé aux plugins officiels |
| Outils                               |                                    64 par extension, 256 au total |
| Skills                               |                                                  32 par extension |
| Ressources                           |                                                  64 par extension |
| Contributions UI standard            |                                                  32 par extension |
| Actions UI                           |                                                  64 par extension |
| Thèmes                               |                                                   8 par extension |
| Résultat riche                       |                        16 blocs, dont 8 fichiers, 20 Mio au total |
| Lot parallèle de fichiers éphémères  |                                                            64 Mio |
| Aperçus multimodaux par continuation |                                                                 8 |
| Message entre Beaver et l’Hôte       |                                                             1 Mio |
| Empreinte                            | 2 000 fichiers, 4 Mio par fichier, 32 Mio au total, profondeur 16 |
| Exécution d’un outil                 |                                                       55 secondes |
| Gestionnaire d’événement             |                                                        5 secondes |
| Action UI standard                   |                                                       15 secondes |
| Redémarrages automatiques d’un Hôte  |                                    3 sur une fenêtre de 5 minutes |

Ces nombres sont un résumé daté. Les valeurs exhaustives et faisant foi restent dans
les deux contrats JSON liés en tête du document et dans les tables générées du SDK.

## État des validations multiplateformes

Le code, les contrats et les parcours automatisés fusionnés sont validés. Les essais
manuels avec paquets installés Windows et Linux restent suivis séparément afin de ne pas
transformer une CI verte en preuve d’un comportement qu’aucun testeur n’a observé :

- [acceptation fonctionnelle Windows/Linux](./docs/fonctionnalites/extension/CHECKLIST_ACCEPTATION_EXTENSIONS_WINDOWS_LINUX.md) ;
- [résilience du démarrage Windows/Linux](./docs/fonctionnalites/extension/CHECKLIST_STARTUP_RESILIENCE_WINDOWS_LINUX.md).

Chaque colonne Windows et Linux se ferme indépendamment. Un succès macOS, Vite ou CI
n’est pas recopié manuellement dans une case qui exige un paquet installé.

L’écosystème est conçu pour évoluer sans transformer les Tools internes de Beaver en
plugins. Les futures capacités devront étendre les contrats versionnés plutôt que créer
une seconde autorité parallèle.
