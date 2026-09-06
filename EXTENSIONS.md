# Extensions Beaver

Beaver peut charger des extensions personnalisées pour ajouter des outils à l’agent,
réagir aux événements de l’application et étendre son interface. Une extension peut
être un simple fichier local, un dossier, un dépôt Git ou un package npm.

Ce document explique le parcours utilisateur et les bases nécessaires pour créer une
extension. La [référence du SDK](./src-tauri/resources/extension-host/sdk/README.md)
et les contrats JSON générés restent les autorités techniques complètes :

- [contrat du runtime](./src-tauri/resources/extension-host/contract.json) ;
- [contrat de l’interface](./src-tauri/resources/extension-ui/contract.json).

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

| Élément | Prise en charge actuelle |
|---|---|
| Systèmes | macOS Apple Silicon, Linux x64 et Windows x64 |
| Langages d’entrée | JavaScript et TypeScript, modules JS/TS/JSX/TSX compris |
| Runtime | Node.js 20 minimum ; l’environnement Beaver actuel vise Node.js 24 LTS |
| API Beaver | `beaverApi: "1"` |
| Interface | API UI `1`, modes `standard` et `advanced` |
| Sources | fichier local, dossier local, Git HTTPS/SSH et registre npm officiel |

Le point d’entrée d’une extension hébergée doit être écrit en JavaScript ou TypeScript.
Comme ce code possède l’accès complet à Node.js, il peut appeler un programme écrit
dans un autre langage ou un service externe. L’auteur doit alors fournir ce programme,
vérifier sa présence et gérer sa compatibilité sur chaque système.

Une extension qui dépend d’un module natif, d’un exécutable ou d’un comportement propre
à un système n’est pas automatiquement multiplateforme. Testez-la sur chaque système
que vous annoncez compatible.

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
lui attribue alors une identité locale. Le manifeste explicite reste préférable pour
garder une identité stable, publier l’extension et déclarer une interface.

Beaver reconnaît `beaver-extension.json`, `beaver.json` ou un bloc `beaver` dans
`package.json`.

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
          name: { type: "string", description: "Nom de la personne." }
        },
        required: ["name"],
        additionalProperties: false
      },
      effect: "read-only",
      async execute({ name }, context) {
        return {
          content: `Bonjour ${String(name)} depuis ${context.workingDirectory}`,
          displaySummary: "Salutation créée"
        };
      }
    });

    unsubscribe = beaver.on("session.turn.started", async (event) => {
      // Réagir au début d’un tour sans bloquer les autres extensions.
      void event;
    });
  },

  deactivate() {
    unsubscribe();
  }
});
```

Beaver transforme le nom public en `com.example.hello.hello`. Donnez à chaque outil un
nom court, une description précise, un schéma JSON strict et la bonne classe d’effet.
Ces informations aident le modèle à choisir correctement l’outil et permettent à
Beaver d’appliquer la politique de permission adaptée.

## Déclarer l’effet d’un outil

| Effet | Usage typique | Comportement en mode manuel |
|---|---|---|
| `read-only` | Lecture locale sans effet externe | Pas de confirmation ; autorisé en mode Plan |
| `external-read` | Lecture réseau ou service distant | Confirmation ; refusé en mode Plan |
| `local-write` | Écriture sur la machine | Confirmation ; refusé en mode Plan |
| `external-write` | Modification d’un service distant | Confirmation ; refusé en mode Plan |
| `process` | Lancement ou contrôle d’un processus | Confirmation ; refusé en mode Plan |
| `secret` | Accès à un secret | Confirmation ; refusé en mode Plan |
| `unknown` | Effet absent ou impossible à classer | Bloqué par défaut comme action sensible |

Le mode automatique peut autoriser une action sans dialogue. La classe d’effet reste
obligatoire : elle documente le comportement réel et protège les modes plus restrictifs.

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

Les vues standard disposent de textes localisés, de rangées, piles, titres, badges,
séparateurs, champs texte ou numériques, sélecteurs, interrupteurs et boutons. Un
bouton appelle une action enregistrée avec `beaver.ui.onAction(...)`.

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

Ce mode n’est pas une sandbox. Une confirmation supplémentaire est demandée à
l’activation. Beaver compile et empreinte l’artefact, refuse un artefact modifié et
retire les montages lors d’un rechargement ou d’une désactivation, mais l’auteur reste
responsable de tout ce que son module exécute dans la WebView.

Utilisez le mode avancé seulement si le contrat standard ne permet pas l’interface
voulue.

## Ajouter l’extension à Beaver

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

Après une mise à jour ou un changement des fichiers couverts par l’empreinte, Beaver
révoque l’approbation précédente, désactive l’extension et demande une nouvelle
confirmation. Ce contrôle porte sur les fichiers sélectionnés par l’empreinte au
moment de la vérification : sources JavaScript/TypeScript, manifeste et artefact UI.
Il exclut notamment `node_modules` et ne fige pas les fichiers ouverts ensuite par
Node. Il ne garantit donc pas qu’une modification locale d’une dépendance ou un
remplacement entre vérification et import demandera une nouvelle approbation.

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

Si Beaver s’est arrêté pendant le chargement d’une extension, il affiche au prochain
démarrage un parcours de récupération. Vous pouvez garder l’extension désactivée,
retenter son chargement ou restaurer l’état précédent.

Si une interface avancée empêche l’application de s’ouvrir correctement, quittez
complètement Beaver puis maintenez **Maj** pendant son redémarrage. Vous pouvez aussi
lancer l’exécutable Beaver avec l’argument exact `--safe-mode`. Ce lancement n’affiche
aucune interface tierce et laisse l’écran Extensions disponible pour désactiver le
module fautif. Sous Linux Wayland, Beaver demande la touche Maj après l’apparition de
la WebView, car le système ne permet pas la même détection native qu’ailleurs.

Les sorties brutes d’une extension ne sont pas conservées : elles peuvent contenir des
secrets. Les diagnostics indiquent l’étape, la catégorie et la position sûre quand elle
est disponible, sans exposer la sortie complète du code tiers.

## Préparer une extension à distribuer

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

L’écosystème est conçu pour évoluer sans transformer les Tools internes de Beaver en
plugins. Les futures capacités devront étendre les contrats versionnés plutôt que créer
une seconde autorité parallèle.
