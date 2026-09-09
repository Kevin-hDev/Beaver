# Le centre d'extensions

**Emplacement site** — Intégrations › Extensions › Le centre d'extensions
**Répond à** — « Comment je trouve, installe, active, mets à jour, désactive et supprime une extension — et qu'est-ce que je risque ? »
**Sources** — `EXTENSIONS.md` (racine, daté du 6 septembre 2026) ; `src-tauri/src/services/extensions/` (`registry.rs`, `registry_state.rs`, `manifest.rs`, `builtin.rs`, `fingerprint.rs`, `fingerprint_paths.rs`, `storage.rs`, `managed_store.rs`, `ui_artifact_store.rs`, `access_log.rs`, `operation_log.rs`, `loading_marker.rs`, `discovery_preferences.rs`, `discovery_limits.rs`, `host_paths.rs`, `host_identity.rs`, `installer_uninstall.rs`, `runtime_sync.rs`, `runtime_status.rs`, `ui_startup.rs`, `install_jobs/`) ; `src-tauri/src/commands/extensions.rs`, `commands/extension_installs.rs`, `commands/agent_settings.rs` ; `src-tauri/src/services/agent_local/tool_catalog.rs`, `tool_group_catalog.rs` ; `src-tauri/resources/extension-host/contract.json`, `builtin-plugins/catalog.json` ; `src-tauri/resources/extension-discovery/contract.json` ; `src/components/extensions/`, `src/hooks/use-extensions.tsx`, `src/components/settings/tools-settings.tsx`, `src/components/agent-local/chat-plus-plugin-menu.tsx`, `src/i18n/fr.json`, `src/styles/tokens.css`
**Vérification** — Vérifié dans le code, sauf les points d'affichage listés en fin de fichier

---

## Plan de page proposé

1. Ce qu'est une extension, et ce qu'elle peut faire à votre machine
2. Extension personnalisée ou plugin officiel : deux choses différentes
3. Où trouver une extension
4. Où vivent les extensions sur votre disque
5. Ajouter une extension, étape par étape
6. Les deux décisions séparées : installer, puis faire confiance
7. Activer et désactiver
8. Afficher dans le champ de saisie
9. Mettre à jour
10. Recharger après une modification locale
11. Supprimer
12. Quand une extension plante
13. Ce que vous voyez dans les réglages — et ce que vous n'y voyez pas
14. Les limites

---

## Contenu

### 1. Ce qu'est une extension, et ce qu'elle peut faire à votre machine

Une extension ajoute des capacités à Beaver : de nouveaux outils pour l'agent, des skills, des ressources, des éléments d'interface.

**C'est du code écrit par quelqu'un d'autre, que Beaver exécute sur votre ordinateur avec vos droits d'utilisateur.** Le dire d'emblée, sans le diluer.

Faits à énoncer, tous vérifiés :

- Le code d'une extension approuvée s'exécute **avec les droits de votre compte** : il peut lire et écrire vos fichiers, lancer des programmes et utiliser le réseau (`EXTENSIONS.md`, « À retenir avant de commencer » et tableau « Ce que Beaver isole — et ce qu'il n'isole pas »).
- **Ce n'est pas un bac à sable.** Beaver l'écrit lui-même : « Une extension personnalisée est du code local de confiance, exécuté avec les droits du compte utilisateur. Ce n'est pas une sandbox. » L'isolation par processus sert la **stabilité**, pas la protection contre le code que vous avez approuvé (`EXTENSIONS.md`, section « Isolation et confiance »).
- Une extension approuvée peut **demander vos clés d'API et vos jetons** directement, par l'API du kit de développement. L'approbation porte sur tout son code et toutes ses dépendances : **il n'existe pas d'autorisation clé par clé** (`EXTENSIONS.md`, « Isolation et confiance »).
- Beaver **ne vérifie pas ce que fait une extension**. Il valide le manifeste et le protocole ; cela ne remplace pas un audit du comportement (`EXTENSIONS.md`, « Limites actuelles »).
- Le manifeste d'une extension Node.js déclare `access: "full"`, et le code commente pourquoi : « du code Node.js local ne constitue pas une frontière de sécurité » (`EXTENSIONS.md`, « Créer le manifeste »).

Ce que Beaver garantit malgré tout, et qu'il faut présenter comme un vrai bénéfice, pas comme une excuse :

| Mécanisme | Ce qu'il garantit réellement |
|---|---|
| Un processus séparé par extension tierce | Le plantage de l'une n'arrête pas les autres |
| Identité attribuée par Beaver, pas déclarée par l'extension | Une extension ne peut pas se faire passer pour une autre (`host_identity.rs:6-17`) |
| Environnement de processus minimal | Vos variables d'environnement et vos clés ne sont pas transmises automatiquement |
| Empreinte des fichiers | Un fichier modifié révoque l'approbation et désactive l'extension |
| Journal d'accès aux secrets | Chaque remise de secret est tracée, sans écrire le secret |

### 2. Extension personnalisée ou plugin officiel : deux choses différentes

C'est la distinction structurante de l'écran, et la page doit l'établir avant tout le reste.

| | Plugin officiel | Extension personnalisée |
|---|---|---|
| Qui l'écrit | Beaver | Vous, un dépôt Git, un package npm |
| Où il vit | Dans l'application, livré avec elle | Dans vos données, ajouté par vous |
| Audité | Oui, livré et audité avec Beaver | **Non** |
| Approbation | **Approuvé d'office** — `trusted: true` à la création (`builtin.rs:38`) | Demandée explicitement, extension par extension |
| Actif au premier lancement | Oui pour les quatre plugins livrés (`builtin-plugins/catalog.json`) | **Non** : désactivé et non approuvé (`manifest.rs:75-76`) |
| Processus | Tous partagent **un** hôte officiel | **Un hôte par extension** (`host_identity.rs:12-17`) |
| Suppression | Impossible — il fait partie de l'application | Possible |
| Onglet où il apparaît | **Plugins** | **Extensions** |

Les quatre plugins officiels livrés aujourd'hui, avec leur identifiant technique et leur nom affiché en français :

| Identifiant | Nom affiché | Ce qu'il fait |
|---|---|---|
| `beaver.office.documents` | **Documents** | Créer et modifier des documents DOCX éditables |
| `beaver.office.pdf` | **PDF** | Lire, créer, fusionner et vérifier des PDF |
| `beaver.office.spreadsheets` | **Feuilles de calcul** | Créer, inspecter et modifier des classeurs XLSX |
| `beaver.office.presentations` | **Présentations** | Créer des présentations PPTX et remplir des modèles |

Tous les quatre sont **activés** et **affichés dans le chat** par défaut (`builtin-plugins/catalog.json`, champs `enabled` et `showInChat` ; libellés dans `src/i18n/fr.json`, `extensions.official.*`).

Dans l'interface, le type est écrit en toutes lettres sous le nom : **Plugin officiel** ou **Extension locale** (`src/i18n/fr.json`, `extensions.kinds`).

### 3. Où trouver une extension

À dire franchement : **il n'existe pas encore de catalogue intégré ni de place de marché** dans Beaver (`EXTENSIONS.md`, « Limites actuelles »).

Vous apportez donc l'extension vous-même, par l'une des quatre sources acceptées :

| Source | Ce que vous fournissez |
|---|---|
| Fichier local | Un fichier JavaScript, TypeScript ou un manifeste JSON |
| Dossier local | Un dossier contenant un manifeste Beaver ou un `package.json` |
| Dépôt Git | Une adresse **HTTPS ou SSH**, éventuellement suivie de `#branche`, `#tag` ou `#commit` |
| Package npm | Un nom publié, par exemple `@exemple/extension` ou `extension@latest` |

Extensions de fichier acceptées par le sélecteur : `js`, `mjs`, `cjs`, `jsx`, `ts`, `mts`, `cts`, `tsx`, `mtsx`, `ctsx`, `json` (`extension-add-dialog.tsx:39-42`).

Refusés côté Git : **HTTP non chiffré**, et toute adresse contenant des identifiants (`EXTENSIONS.md`, « Dépôts Git »).

Comme il n'y a pas de catalogue, la page doit donner un critère de choix à l'utilisateur, pas seulement une procédure. Trois questions à poser avant d'installer, à formuler sur le site :

1. Savez-vous qui a écrit ce code, et pouvez-vous le vérifier ?
2. Pouvez-vous lire ce que fait l'extension avant de l'approuver ?
3. Cette extension a-t-elle une raison légitime de demander vos clés d'API ?

### 4. Où vivent les extensions sur votre disque

Tout est dans le dossier de données de Beaver, identique sur les trois systèmes : **`~/.local/share/cl-go-dash/`**.

| Chemin | Contenu |
|---|---|
| `extensions.json` | Le registre : la liste des extensions, leur état, leur approbation (`storage.rs:8` et `:35`) |
| `extension-installs/` | Les **copies gérées** des installations Git et npm (`managed_store.rs:5` et `:49-51`) |
| `extensions-ui/` | Les modules d'interface compilés et empreintés (`ui_artifact_store.rs:11` et `:69`) |
| `extension-install-jobs/jobs.json` | Le journal des installations, qui permet la reprise (`install_jobs/checkpoint.rs:9-10` et `:32`) |
| `extension-install.jsonl` | Le journal des opérations d'installation (`operation_log.rs:5` et `:41`) |
| `extension-access.jsonl` | Le journal des **accès aux secrets** demandés par les extensions (`access_log.rs:7` et `:125`) |
| `extension-loading.json` | Le marqueur du chargement en cours, lu au démarrage suivant en cas d'arrêt brutal (`loading_marker.rs:6` et `:200`) |
| `extension-discovery-preferences.json` | Les plugins que vous avez marqués prioritaires (`discovery_preferences.rs:91`) |
| `extension-tool-usage.json` | L'usage des outils, utilisé pour la découverte progressive (`discovery_usage.rs:98`) |
| `extension-host-channels/` | Les canaux de communication temporaires avec les hôtes (`runtime.rs:38`) |

Les deux journaux `.jsonl` sont **bornés à 64 Kio** : les lignes les plus anciennes sont retirées au fur et à mesure (`bounded_jsonl.rs:5`, `:12-16`).

**Point important pour l'utilisateur, à énoncer clairement :** une extension ajoutée depuis un **fichier ou un dossier local** n'est pas copiée. Beaver charge la source à l'endroit où elle se trouve chez vous. Une extension installée depuis **Git ou npm** est copiée dans `extension-installs/`, et c'est cette copie que Beaver exécute (`EXTENSIONS.md`, tableau « Ajouter l'extension à Beaver » ; `managed_store.rs:58-68`).

Le code de l'hôte, lui, vit dans l'application et non dans vos données : `resources/extension-host/`, avec `host.mjs` et `ui-build.mjs`. Beaver utilise le Node.js livré avec l'application s'il est présent, sinon celui du système ; **Node 20 est le minimum** (`host_paths.rs:20-60` ; `types.rs:6`).

### 5. Ajouter une extension, étape par étape

Le parcours complet, chaque étape vérifiée. C'est le cœur de la page : à rédiger comme un guidage numéroté, avec un encadré de risque au moment de la décision.

**Étape 1 — Ouvrir l'écran.** **Réglages › Extensions**. Trois onglets : **Plugins**, **Extensions**, **Hôte** (`extension-sections.ts:16-20` ; libellés dans `src/i18n/fr.json`, `extensions.sections`).

**Étape 2 — Se placer sur l'onglet Extensions.** Le bouton **Ajouter** n'apparaît que sur cet onglet (`extensions-page.tsx:95-102`). Les plugins officiels ne s'ajoutent pas : ils sont livrés.

**Étape 3 — Cliquer sur Ajouter.** La fenêtre **Ajouter une extension** s'ouvre. Elle affiche d'abord un avertissement, avant toute option :

> « Le code sélectionné s'exécutera avec les droits de votre compte utilisateur. Installez uniquement une extension à laquelle vous choisissez de faire confiance. »
> (`extension-add-dialog.tsx:105-108` ; texte exact dans `src/i18n/fr.json`, `extensions.add.fullAccessWarning`)

**Étape 4 — Choisir la source** parmi les quatre proposées : **Choisir un fichier**, **Choisir un dossier**, **Installer depuis Git**, **Installer depuis npm**. Git et npm ouvrent un champ de saisie ; fichier et dossier ouvrent le sélecteur du système (`extension-add-dialog.tsx:112-165`).

**Étape 5 — Laisser l'installation travailler.** Elle s'exécute **en arrière-plan**. Vous pouvez fermer la fenêtre et continuer à utiliser Beaver ; fermer le suivi n'annule rien. Le suivi apparaît à deux endroits : sur la page Extensions, et dans le panneau de notifications de la barre supérieure (`extensions-tab.tsx:28`, `app-layout.tsx:173`).

Les étapes affichées pendant le travail, dans l'ordre : **Recherche de la source**, **Téléchargement**, **Installation des dépendances**, **Vérification**, **Préparation de l'interface**, **Finalisation**, **Nettoyage** (`src/i18n/fr.json`, `extensionInstalls.phase`).

Les états possibles : **En file d'attente**, **Installation en cours**, **Une confirmation est nécessaire**, **Annulation en cours…**, **Installation réussie**, **Installation annulée**, **Échec de l'installation**, **Installation interrompue** (`src/i18n/fr.json`, `extensionInstalls.status`).

**Une seule installation travaille à la fois** : les autres attendent leur tour, et la ligne en attente indique laquelle la bloque (`install_jobs/worker.rs:41-58` — un seul travailleur, qui prend les travaux l'un après l'autre ; `extensionInstalls.queueBlocked`).

**Étape 6 — Répondre à la demande de volume, si elle apparaît.** Au-delà de **1 Gio** occupé par l'installation, Beaver arrête les écritures et demande **Continuer l'installation** ou **Annuler** (`install_jobs/disk_policy.rs:3`). Il conserve par ailleurs une réserve de **1 Gio** d'espace libre (`disk_policy.rs:4`). **Aucun délai ne vaut accord** : une demande sans réponse reste en attente indéfiniment (`EXTENSIONS.md`, « Ajouter l'extension à Beaver »).

**Étape 7 — Ouvrir la fiche créée.** L'extension apparaît dans la liste. Elle est là — **et elle ne fait rien**. Son état est **Désactivée**, elle n'est pas approuvée (`manifest.rs:75-76`). C'est voulu.

**Étape 8 — Lire la fiche avant d'activer.** La fiche affiche : Activation, État, Niveau d'API (**Stable** ou **Avancé**), Runtime, API Beaver, Auteur, Mode d'installation, Source, Révision Git le cas échéant (`extension-detail.tsx:62-98` ; libellés dans `extensions.detail`). Un bandeau d'avertissement est présent en permanence sur toute extension non officielle :

> « Cette extension n'est pas auditée par Beaver et peut accéder aux données, secrets et fonctions de votre compte. »
> (`extension-detail.tsx:55-60` ; `extensions.fullAccessWarning`)

Le bouton **Ouvrir la source** ouvre le code réellement chargé, pour l'auditer avant de l'approuver (`extension-actions.tsx:30-37` ; commande `open_extension_source`, `commands/extensions.rs:197-201`).

**Étape 9 — Activer, et confirmer la confiance.** Voir la section suivante : c'est une décision distincte, et le code l'impose.

### 6. Les deux décisions séparées : installer, puis faire confiance

**C'est le point le plus important de la page. À traiter comme tel, pas comme un détail de procédure.**

Installer ne donne aucun droit. Le code refuse formellement d'activer une extension non approuvée si la confirmation n'accompagne pas la demande :

```
if enabled && record.kind != ExtensionKind::Builtin && !record.trusted && !trust_confirmed {
    return Err(ACTIVATION_CONFIRMATION_REQUIRED);
}
```
(`registry.rs:117-122`)

Côté interface, basculer l'interrupteur d'une extension non approuvée **n'active rien** : cela ouvre une fenêtre de confirmation (`use-extensions.tsx:120-128`).

La fenêtre affiche :

- le titre **« Activer {nom} ? »** ;
- le texte : **« Cette extension aura un accès total aux données, secrets et fonctions de votre compte. Beaver ne l'a pas auditée. »** ;
- deux boutons : **Annuler** et **Faire confiance et activer**.

(`extension-activation-dialog.tsx:54-97` ; textes dans `extensions.activation`)

**Confirmation supplémentaire pour une interface avancée.** Si le manifeste déclare un module d'interface en mode `advanced`, la fenêtre ajoute un paragraphe et **une case à cocher obligatoire** :

- « Son module d'interface avancé s'exécute directement dans Beaver et peut modifier, consulter ou casser toute l'interface. »
- Case : « Je comprends et j'approuve explicitement ce module d'interface avancé. »

Le bouton de confirmation **reste inactif tant que la case n'est pas cochée** (`extension-activation-dialog.tsx:29-30`, `:61-74`, `:93`).

**Ce que l'approbation enregistre.** Au moment où vous confirmez, Beaver calcule l'**empreinte** des fichiers de l'extension, la stocke, et note la date d'approbation (`registry.rs:124-126` ; `registry_state.rs:6-14`). La fiche affiche ensuite **Approuvée depuis** avec cette date (`extension-detail.tsx:84`).

**Ce que couvre l'empreinte**, et ce qu'elle ne couvre pas — à écrire, parce que c'est une limite réelle :

- couverts : les sources JavaScript/TypeScript, le manifeste, l'artefact d'interface ;
- **exclus : `node_modules` et `.git`** (`fingerprint_paths.rs:54-58`) ;
- les liens symboliques sont refusés (`fingerprint_paths.rs:3-9`) ;
- l'empreinte est comparée en **temps constant** (`fingerprint.rs:57`) ;
- limites : **2 000 fichiers**, **4 Mio par fichier**, **32 Mio au total**, **profondeur 16** (`contract.json`, champs `fingerprintMax*`).

Conséquence à énoncer : une modification apportée à une **dépendance** installée dans `node_modules` ne redéclenche **pas** de demande d'approbation.

**Un raccourci existe aussi dans le champ de saisie de l'agent** — le menu `+` — mais il mène exactement à la même fenêtre de confirmation : le raccourci n'est pas une porte dérobée (`chat-plus-menu.tsx:209-211` appelle `setEnabled`, qui ouvre la même fenêtre).

### 7. Activer et désactiver

**Activer** rend les outils de l'extension disponibles à l'agent et lance son processus.

**Désactiver** fait trois choses, toutes vérifiées :

1. l'extension est marquée désactivée dans le registre, et ce changement est **écrit sur le disque avant** l'arrêt du processus (`installer_uninstall.rs:15-19` pour la suppression ; `registry.rs:129` pour la désactivation) ;
2. les **permissions accordées pendant les conversations sont effacées** (`registry.rs:142-145`, `permission_gate::clear_extension`) ;
3. le processus de l'extension est arrêté (`commands/extensions.rs:89-95`).

**L'approbation, elle, survit à la désactivation.** Réactiver ensuite ne redemande pas de confirmation, tant que les fichiers n'ont pas changé (`registry.rs:117-122` : la confirmation n'est exigée que si `!record.trusted`).

**Un rappel apparaît si l'extension avait obtenu un secret.** Si vous désactivez ou supprimez une extension à laquelle un secret a été remis, Beaver affiche :

> « Par précaution, changez auprès de leur fournisseur les accès utilisés par cette extension. »
> (`extensions.sensitiveAccessReminder` ; déclenché par `registry.rs:137` et remonté dans `use-extensions.tsx:89-92`)

**Ce rappel n'est pas décoratif, et la page doit expliquer pourquoi** : désactiver ne récupère pas une copie déjà obtenue. Seul le fournisseur du secret peut l'invalider (`EXTENSIONS.md`, « Isolation et confiance »).

### 8. Afficher dans le champ de saisie

Chaque extension porte un second interrupteur, **Afficher dans le chat**, indépendant de l'activation (`extension-detail.tsx:86-97` ; `extension-row.tsx:51-59`).

Ce réglage **ne change rien aux capacités du modèle**. Il ajoute seulement un raccourci d'activation dans le menu `+` du champ de saisie de l'agent (`chat-plus-plugin-menu.tsx:14-16`).

La liste de ce menu affiche **huit lignes** avant de devenir défilable — c'est une hauteur d'affichage, **pas** une limite d'extensions (`src/styles/tokens.css:187-188` : hauteur de ligne `2.25rem`, hauteur maximale `2.25rem × 8`).

Ce menu n'existe qu'en mode Agent (`chat-plus-menu.tsx:199`, condition `agentic`).

Valeurs par défaut : **activé** pour les quatre plugins officiels, **désactivé** pour toute extension ajoutée (`builtin-plugins/catalog.json` ; `manifest.rs:80`).

### 9. Mettre à jour

**Seules les extensions installées depuis Git ou npm ont un bouton Mettre à jour.** Une extension ajoutée depuis un fichier ou un dossier local n'en a pas : c'est vous qui possédez la source (`extension-detail.tsx:36` ; `extension-actions.tsx:38-46`).

Le bouton demande une confirmation en deux temps : **Mettre à jour**, puis **Confirmer la mise à jour** (`extension-actions.tsx:39-45`).

Un avertissement est affiché en permanence au-dessus des actions d'une extension gérée :

> « La mise à jour désactivera l'extension. Examinez sa nouvelle version avant de lui refaire confiance. »
> (`extension-actions.tsx:23-28` ; `extensions.updateTrustWarning`)

Ce qui se passe réellement :

- Beaver prépare une **nouvelle copie à côté** de l'ancienne et ne remplace qu'après validation ;
- l'empreinte change, donc **l'approbation est révoquée**, l'extension est **désactivée**, son état passe à **Erreur** et son message devient « Les fichiers de l'extension ont changé et doivent être vérifiés à nouveau » (`registry_state.rs:16-34` ; `error_codes` `extensions_fingerprint_changed`) ;
- une mise à jour qui **change l'identité** de l'extension est refusée : « La mise à jour a changé l'identité de l'extension et a été refusée » (`EXTENSIONS.md`, « Dépôts Git » ; `extensions_update_identity_changed`).

**Aucune mise à jour automatique n'existe.** Rien ne se met à jour tout seul, et rien ne se réactive tout seul.

### 10. Recharger après une modification locale

Pour une extension liée à un fichier ou un dossier de votre machine, **Recharger** est la façon d'appliquer vos modifications.

**Deux conséquences à écrire explicitement, parce qu'elles surprennent :**

1. **Recharger redémarre l'hôte entier**, pas seulement l'extension concernée. Le bouton est sur la fiche de l'extension, mais la commande est `reload_extension_host` : toutes les extensions s'arrêtent et se rechargent (`extensions-tab.tsx:54` → `use-extensions.tsx:192` → `commands/extensions.rs:115-124`).
2. **Vos modifications font changer l'empreinte.** Au rechargement, Beaver revérifie les fichiers de chaque extension locale ; celles dont les fichiers ont changé sont **révoquées et désactivées**, et demandent une nouvelle approbation (`runtime_sync.rs:44-47` ; `fingerprint.rs:18-46`).

Autrement dit : pendant que vous développez une extension, **chaque rechargement vous fait repasser par la fenêtre de confiance**. Ce n'est pas un défaut, c'est la garantie qui protège les utilisateurs qui, eux, ne modifient jamais leur code.

### 11. Supprimer

Le bouton **Supprimer** demande une confirmation en deux temps : **Supprimer**, puis **Confirmer** (`extension-actions.tsx:55-61`).

Ce que Beaver fait, dans cet ordre (`installer_uninstall.rs:11-45`) :

1. il efface les permissions accordées pendant les conversations ;
2. il **écrit la désactivation sur le disque avant** d'arrêter quoi que ce soit — un commentaire du code explique pourquoi : ainsi un processus dont l'arrêt n'est pas confirmé ne peut jamais redevenir actif au redémarrage suivant ;
3. il arrête le processus de l'extension et **attend la confirmation de cet arrêt** avant de continuer ;
4. il retire l'extension du registre ;
5. il supprime son artefact d'interface ;
6. il supprime la **copie gérée** — uniquement si l'installation venait de Git ou npm.

**Ce que la suppression ne fait pas :**

- elle **ne touche pas** à votre fichier ou à votre dossier si l'extension était locale : seule la copie appartenant à Beaver est retirée (`installer_uninstall.rs:29-41` ; `EXTENSIONS.md`, tableau « Ajouter l'extension à Beaver ») ;
- elle **ne récupère aucun secret** déjà remis à l'extension. Si un secret a été remis, le rappel s'affiche et il faut le révoquer chez son fournisseur ;
- elle **ne supprime pas** les fichiers que l'extension aurait écrits ailleurs sur votre disque pendant qu'elle tournait.

**Aucune annulation n'est proposée après une suppression.** À signaler dans la page, et à porter comme demande d'évolution.

Les copies gérées devenues orphelines sont nettoyées **au démarrage suivant** (`registry_startup.rs:22-27`).

### 12. Quand une extension plante

L'ordre de dépannage à donner, repris et vérifié depuis `EXTENSIONS.md` (« En cas de problème ») :

1. **Réglages › Extensions › Hôte** — lire le diagnostic.
2. Ouvrir la fiche de l'extension : état, source, dernière erreur.
3. **Ouvrir la source** pour auditer le code réellement chargé.
4. **Recharger** après une correction locale, ou **Mettre à jour** pour une installation Git/npm.
5. **Désactiver** l'extension : son processus et ses permissions de session sont révoqués.
6. **Redémarrer l'hôte** depuis son panneau si le runtime lui-même est en erreur.
7. **Supprimer** l'extension si vous ne lui faites plus confiance.

**Ce que l'onglet Hôte affiche** : État (Arrêté / Démarrage / En cours / Erreur), version de Node.js, version de Jiti, version de l'API Beaver, nombre d'extensions actives, et la liste des **Dernières erreurs de chargement** avec, pour chacune, l'identifiant de l'extension fautive et le fichier concerné (`extensions-host-panel.tsx:39-73`).

Deux boutons y vivent : **Redémarrer l'hôte**, et un **Mode de récupération** décrit ainsi : « Désactive les plugins Beaver et les extensions locales afin de retrouver un Beaver fonctionnel » (`extensions-host-panel.tsx:74-89` ; `extensions.host.recovery*`).

**Un plantage n'entraîne pas les autres.** Chaque extension tierce a son propre processus, donc son propre domaine de panne. Beaver redémarre automatiquement un hôte au plus **3 fois sur une fenêtre de 5 minutes** ; au-delà, il s'arrête d'essayer (`contract.json`, `maxHostRestartsPerWindow: 3`, `hostRestartWindowSeconds: 300`).

**Si Beaver s'est arrêté pendant le chargement d'une extension**, un parcours de récupération apparaît au démarrage suivant, avec quatre issues : **Ouvrir la fiche**, **Garder désactivée**, **Réessayer le chargement**, **Restaurer le registre précédent** (`extensions.recovery.*`).

**Si une interface avancée empêche Beaver de s'ouvrir**, il existe un démarrage sans interface tierce :

- quitter complètement Beaver, puis **maintenir Maj** pendant le redémarrage ;
- ou lancer l'exécutable avec l'argument exact **`--safe-mode`** (`ui_startup.rs:8`, `:19-59`).

Ce lancement n'affiche aucune interface tierce et **laisse l'écran Extensions accessible** pour désactiver le module fautif. Sous **Linux avec Wayland**, la touche Maj est demandée après l'apparition de la fenêtre, le système ne permettant pas la même détection qu'ailleurs (`EXTENSIONS.md`, « En cas de problème » ; `ui_startup.rs:63-68`).

**Si le registre lui-même est illisible ou vient d'une version plus récente de Beaver**, l'application refuse le registre entier plutôt que de le remplacer par une liste vide, conserve vos fichiers, et bloque toute modification d'extension jusqu'au retour à un état compatible. Les conversations continuent avec les seuls outils internes et un avertissement (`EXTENSIONS.md`, « En cas de problème » ; messages `extensions_registry_*` dans `src/i18n/fr.json`).

**Les sorties brutes d'une extension ne sont jamais conservées**, parce qu'elles peuvent contenir des secrets. Les diagnostics donnent l'étape, la catégorie et la position, pas le contenu (`EXTENSIONS.md`, « En cas de problème »).

### 13. Ce que vous voyez dans les réglages — et ce que vous n'y voyez pas

**Sur la fiche d'une extension**, la section **Contributions enregistrées** liste chaque outil déclaré avec son nom technique, sa **classe d'effet** traduite (Lecture seule, Écriture locale, Lecture externe, Écriture externe, Processus, Accès aux secrets, Effet inconnu) et, le cas échéant, la mention **Remplace un outil intégré** (`extension-detail.tsx:133-177` ; `extensions.effects`, `extensions.detail.replacesCore`).

Une section **Skills et ressources** liste les skills et ressources déclarés (`extension-capabilities.tsx`).

**Ce que vous ne voyez pas — et c'est le piège de cette page.**

L'écran **Réglages › Outils**, qui liste ce que l'agent a le droit de faire, affiche **uniquement les outils de Beaver, groupés par famille** : Terminal, Fichiers, Recherche de fichiers, Web, Connecteurs externes, puis les familles optionnelles (Skills, Automatisations, Choix utilisateur, Sous-agents, Mode Plan, Todo, Branches Git, Forecast, Images) (`tool_group_catalog.rs:12-80` ; `tools-settings.tsx:66-105`).

**Aucun outil d'extension n'y figure**, et ce pour deux raisons distinctes :

1. Les outils qu'une extension déclare (par exemple `com.exemple.hello.hello`) sont **dynamiques** : ils ne sont pas dans le catalogue statique de cet écran. Ils se pilotent uniquement en activant ou désactivant leur extension.
2. Les **trois outils par lesquels l'agent découvre les extensions** — `list_extensions`, `inspect_extensions` et `load_extension_resource` — sont marqués **toujours actifs** dans le code (`tool_catalog.rs:41-49`, entrées `locked` du groupe `extensions` ; noms dans `resources/extension-discovery/contract.json:2`). Mais **le groupe `extensions` n'existe pas** dans la liste des familles affichées (`tool_group_catalog.rs`, aucune entrée `extensions`). Résultat : **ces trois outils sont actifs en permanence et n'apparaissent nulle part dans les réglages.**

Ce n'est pas un défaut de sécurité — ces trois outils ne font que lister, inspecter et lire ce que vous avez vous-même approuvé — mais c'est une **zone d'ombre à documenter**, sans quoi l'utilisateur croit que l'écran Outils dit tout.

**Un dernier réglage à connaître : Plugins prioritaires.** Quand les définitions des extensions dépassent **10 %** de la fenêtre de contexte du modèle, Beaver ne les envoie plus toutes et bascule en découverte progressive. Vous pouvez désigner jusqu'à **15** plugins à garder disponibles en priorité (`discovery_limits.rs:1` ; `extension-discovery/contract.json`, `contextThresholdPercent: 10` ; libellés `extensions.discovery`).

### 14. Les limites

À présenter en tableau, en distinguant ce qui concerne l'utilisateur de ce qui concerne l'auteur d'extension — la page renvoie à `extensions-ecrire.md` pour la seconde moitié.

---

## Tableaux

### Les limites qui concernent l'utilisateur

| Limite | Valeur | Source |
|---|---|---|
| Extensions enregistrables | **128** | `contract.json`, `maxUserExtensions` |
| Total avec les plugins officiels | **132** | `contract.json`, `maxExtensions` |
| Processus hôtes simultanés | **32**, dont un réservé aux plugins officiels | `contract.json`, `maxHostProcesses` |
| Outils au total | **256** | `contract.json`, `maxTools` |
| Outils par extension | **64** | `contract.json`, `maxToolsPerExtension` |
| Plugins prioritaires sélectionnables | **15** | `discovery_limits.rs:1` |
| Seuil de découverte progressive | **10 %** de la fenêtre de contexte | `extension-discovery/contract.json` |
| Installations simultanées | **1** en cours, **8** en file | `install_jobs/worker.rs:41-58`, `install_jobs/limits.rs:1` |
| Installations conservées dans le suivi | **32** | `install_jobs/limits.rs:2` |
| Seuil de confirmation de volume | **1 Gio** | `install_jobs/disk_policy.rs:3` |
| Réserve d'espace libre conservée | **1 Gio** | `install_jobs/disk_policy.rs:4` |
| Redémarrages automatiques d'un hôte | **3 par tranche de 5 minutes** | `contract.json` |
| Durée maximale d'un appel d'outil | **55 secondes** | `contract.json`, `toolCallTimeoutMs` |
| Taille des journaux d'extension | **64 Kio** chacun | `bounded_jsonl.rs:5` |

### Ce que chaque action fait à l'approbation

| Action | Approbation | Activation | Processus | Fichiers |
|---|---|---|---|---|
| Ajouter | Non accordée | Désactivée | Aucun | Copie gérée si Git/npm |
| Activer + confirmer | **Accordée**, empreinte calculée | Activée | Démarré | Inchangés |
| Désactiver | **Conservée** | Désactivée | Arrêté | Inchangés |
| Réactiver | Conservée, pas de nouvelle demande | Activée | Démarré | Inchangés |
| Recharger après modification locale | **Révoquée** | Désactivée | Arrêté | Inchangés |
| Mettre à jour (Git/npm) | **Révoquée** | Désactivée | Arrêté | Copie remplacée |
| Supprimer | Effacée | Retirée | Arrêté | Copie gérée supprimée, source locale intacte |

### Les quatre sources d'installation

| Source | Copiée par Beaver | Bouton Mettre à jour | Bouton Recharger | Suppression retire |
|---|---|---|---|---|
| Fichier local | Non | Non | Oui | La référence seulement |
| Dossier local | Non | Non | Oui | La référence seulement |
| Dépôt Git | Oui, dans `extension-installs/` | **Oui** | Oui | La copie de Beaver |
| Package npm | Oui, dans `extension-installs/` | **Oui** | Oui | La copie de Beaver |

---

## Encadrés

Ces encadrés sont la partie non négociable de la page. Ils vont dans le corps du texte, à l'endroit indiqué — **pas en note de bas de page, pas en fin de page**.

> **⚠ À placer en tête de page, avant toute procédure — Une extension n'est pas dans un bac à sable.**
> Le code d'une extension que vous approuvez s'exécute avec **vos droits d'utilisateur**. Il peut lire et modifier vos fichiers, lancer des programmes et utiliser le réseau, exactement comme un logiciel que vous installeriez vous-même. Beaver isole chaque extension dans son propre processus : cela empêche l'une de faire tomber les autres, **cela ne l'empêche pas de faire ce qu'elle veut de votre machine**. Beaver le dit lui-même dans sa documentation technique, et cette page ne l'adoucit pas.

> **⚠ À placer juste avant l'étape 9 du parcours — Approuver, c'est tout approuver d'un coup.**
> L'approbation porte sur **tout le code de l'extension et sur toutes ses dépendances**. Il n'existe pas d'autorisation par clé, par fichier ou par dossier. Une extension approuvée peut demander vos clés d'API et vos jetons directement, sans nouvelle question.

> **⚠ À placer dans la section Interface avancée — Une interface avancée s'exécute dans Beaver lui-même.**
> Un module d'interface en mode avancé partage la même page que l'interface de Beaver, avec les mêmes droits. Il peut lire, modifier ou casser n'importe quelle partie de ce que vous voyez. C'est pourquoi Beaver exige une **case à cocher supplémentaire** avant d'activer une telle extension, et pourquoi le bouton reste inactif tant que vous ne l'avez pas cochée. Si vous ne savez pas pourquoi une extension a besoin de ce niveau, ne l'activez pas.

> **⚠ À placer dans la section Désactiver — Désactiver ne récupère pas ce qui est déjà parti.**
> Si une extension a obtenu une de vos clés, la désactiver ou la supprimer **ne l'annule pas** : elle a pu en garder une copie. Beaver vous le rappelle avec le message « Par précaution, changez auprès de leur fournisseur les accès utilisés par cette extension. » La seule action efficace est de **révoquer la clé chez le service concerné**.

> **ℹ À placer dans la section Empreinte — Ce que la vérification des fichiers ne couvre pas.**
> Beaver détecte les modifications des sources, du manifeste et du module d'interface, et vous redemande votre accord. Il **ne surveille pas** le contenu de `node_modules` : une dépendance modifiée localement ne redéclenche pas de demande d'approbation.

> **ℹ À placer dans la section Où trouver une extension — Il n'y a pas de catalogue.**
> Beaver ne propose pas encore de place de marché. Aucune extension ne vous est recommandée, et aucune n'est vérifiée à votre place. Vous apportez le code, vous en répondez.

> **ℹ À placer dans la section Recharger — Développer une extension vous fait repasser par la confiance.**
> Chaque modification de vos fichiers change l'empreinte, donc désactive l'extension et redemande votre accord. C'est normal, et c'est ce qui protège les utilisateurs qui ne modifient jamais leur code.

> **ℹ À placer dans la section Réglages › Outils — L'écran Outils ne montre pas tout.**
> Les outils apportés par une extension n'apparaissent pas dans **Réglages › Outils**. Ils se pilotent uniquement en activant ou en désactivant leur extension, sur sa fiche. Trois outils internes — ceux qui permettent à l'agent de lister et d'inspecter vos extensions — sont **toujours actifs** et ne figurent dans aucun écran.

> **ℹ À placer dans la section Supprimer — La suppression n'est pas annulable.**
> Beaver ne propose pas de restaurer une extension supprimée. Pour une installation Git ou npm, il faudra la réinstaller depuis sa source. Pour une extension locale, votre fichier ou votre dossier est intact : rien n'est perdu.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'extension est installée mais l'agent l'ignore » | Elle est installée, **pas approuvée ni activée** — c'est l'état normal après un ajout | Ouvrir sa fiche, basculer l'interrupteur, confirmer dans la fenêtre |
| « Le bouton de confirmation reste grisé » | L'extension déclare une interface avancée : la case à cocher est obligatoire | Cocher « Je comprends et j'approuve explicitement ce module d'interface avancé » |
| « J'ai modifié mon fichier, l'extension s'est désactivée toute seule » | L'empreinte a changé : Beaver a révoqué l'approbation | Réactiver et reconfirmer — c'est le comportement voulu |
| « Cette extension doit être approuvée à nouveau avant son activation » | Idem : les fichiers ont changé depuis l'approbation | Vérifier ce qui a changé, puis reconfirmer |
| « Les fichiers de l'extension ont changé et doivent être vérifiés à nouveau » | Mise à jour Git/npm appliquée, ou fichiers modifiés | Examiner la nouvelle version avant de réactiver |
| « Il n'y a pas de bouton Mettre à jour » | L'extension vient d'un fichier ou d'un dossier local : c'est vous qui possédez la source | Modifier la source, puis **Recharger** |
| « Recharger a coupé mes autres extensions » | Recharger redémarre **l'hôte entier** | Comportement attendu ; elles se rechargent ensemble |
| « Une confirmation est nécessaire » et rien n'avance | Une demande de volume attend une réponse ; **aucun délai ne vaut accord** | Ouvrir le suivi, cliquer sur **Afficher la demande**, répondre |
| « Installation interrompue » | Beaver ne redémarre jamais un travail seul | **Reprendre l'installation** si le bouton apparaît, sinon **Réessayer** |
| « Cette installation ne peut pas être reprise » | Aucun point de reprise revérifiable | **Réessayer** : une nouvelle installation démarre |
| « Espace disque insuffisant pour poursuivre » | La réserve de 1 Gio est atteinte | Libérer de l'espace, puis réessayer |
| « Une extension avec cet identifiant est déjà installée » | Deux extensions déclarent le même `id` | Supprimer l'ancienne, ou demander à l'auteur un identifiant distinct |
| « Ce package n'est pas une extension Beaver » | Le package npm ne contient ni manifeste `beaver-extension.json` ni bloc `beaver` dans `package.json` | Vérifier que le package est bien prévu pour Beaver |
| « Cette extension n'est pas compatible avec cette version de Beaver » | Le champ `beaverApi` ne correspond pas | Mettre Beaver à jour, ou attendre une version compatible de l'extension |
| « Beaver ne s'ouvre plus depuis que j'ai activé une extension » | Un module d'interface avancé casse la page | Quitter complètement, redémarrer en maintenant **Maj**, ou lancer avec `--safe-mode`, puis désactiver l'extension |
| « L'agent n'a plus aucune extension mais la conversation continue » | Le registre est illisible ou vient d'une version plus récente ; Beaver a gardé les seuls outils internes | Lire l'avertissement, redémarrer ou mettre Beaver à jour ; **aucune donnée n'est supprimée** |
| « Un hôte redémarre en boucle » | L'extension plante à l'activation ; le budget est de 3 redémarrages par 5 minutes | Lire le diagnostic dans l'onglet Hôte, corriger, recharger |
| « L'extension marche en mode Agent mais pas en mode Chatbot » | Les outils d'extension ne sont pas disponibles en mode Chatbot | Passer en mode Agent |
| « Mon extension n'apparaît pas dans Réglages › Outils » | Les outils d'extension ne figurent pas dans cet écran | Les gérer depuis la fiche de l'extension |
| « L'hôte n'a pas confirmé son arrêt » | Un processus tiers ne s'est pas terminé proprement | Quitter complètement Beaver puis le relancer avant de réactiver l'extension |

---

## Renvois

- `07-integrations/extensions-remplacer-un-outil.md` — remplacer un outil natif de Beaver par celui d'une extension, et ce que ça implique
- `07-integrations/extensions-prompt-systeme.md` — comment les extensions se présentent au modèle, la découverte progressive et le prompt système
- `07-integrations/extensions-ecrire.md` — écrire, tester et distribuer sa propre extension ; le manifeste, le kit de développement, les classes d'effet
- `07-integrations/mcp-connecteurs.md` — les connecteurs externes, à ne pas confondre avec les extensions : catalogue fermé d'un côté, code libre de l'autre
- `05-outils/permissions.md` — les modes de permission et les confirmations déclenchées par les classes d'effet
- `11-securite/durcissement.md` — la place des extensions dans la vue d'ensemble de la sécurité
- `10-reglages/integrations.md` — l'écran de réglages où vivent les extensions
- `13-depannage/mcp-extensions-channels.md` — le dépannage général

---

## Points à confirmer

**Décisions produit et rédaction — à trancher avant publication**

1. **Faut-il un avertissement bloquant sur la page elle-même ?** C'est la page la plus sensible du site : elle explique comment exécuter du code tiers sur sa machine. Question posée à l'équipe : ces encadrés suffisent-ils, ou faut-il un bandeau permanent en tête de section Extensions du sommaire ? Recommandation : bandeau permanent, parce qu'un lecteur qui arrive par un moteur de recherche n'aura pas lu l'introduction.
2. **La liste des quatre plugins officiels sera périmée dès qu'un cinquième sera livré.** Elle vit dans `builtin-plugins/catalog.json`. Même arbitrage que pour les connecteurs externes : reproduire la liste et la maintenir, ou renvoyer à l'application. Recommandation : donner les quatre, en précisant la version de Beaver documentée.
3. **Faut-il documenter le mode sûr (`--safe-mode`) sur le site public ?** Il suppose de lancer l'exécutable en ligne de commande, ce que le public de Beaver ne fait pas forcément. La méthode « maintenir Maj au démarrage » suffit peut-être seule ; l'argument exact pourrait vivre uniquement dans la page de dépannage.
4. **La suppression sans annulation est un écart avec la règle d'interface du projet** (« toute action destructive s'accompagne d'une annulation plutôt que d'une confirmation »). Ici il y a une double confirmation et aucune annulation. À signaler comme demande d'évolution, et à décider si la page le mentionne ou reste descriptive.
5. **Le vocabulaire « hôte ».** L'onglet s'appelle **Hôte** dans l'application. C'est un terme technique pour un public non développeur. À décider : garder le mot de l'application et l'expliquer en une phrase à sa première occurrence, ou introduire un synonyme sur le site au risque de désaligner la documentation de l'écran. Recommandation : garder le mot de l'application.

**Écarts relevés entre `EXTENSIONS.md` et le code — à arbitrer**

6. **`EXTENSIONS.md` présente Recharger comme une action portant sur une extension** (« Une extension locale est reconstruite avec **Recharger** »). Dans le code, le bouton de la fiche appelle `reload_extension_host`, qui redémarre **l'hôte entier** et donc toutes les extensions (`extensions-tab.tsx:54`, `commands/extensions.rs:115-124`). La documentation du site doit décrire le comportement réel. À signaler à l'équipe pour que `EXTENSIONS.md` soit corrigé, ou pour que le bouton soit renommé.
7. **Le texte de l'onglet Hôte parle d'un seul processus** : « État du processus séparé qui exécute les plugins officiels et les extensions personnalisées » (`src/i18n/fr.json`, `extensions.host.description`). L'architecture réelle est **un hôte pour les plugins officiels, plus un hôte par extension tierce**, et le panneau n'affiche qu'un état agrégé (`runtime_status.rs`, `runtime.rs:56-61`). Le site ne doit pas reprendre cette formulation. À signaler comme correction de texte dans l'application, dans les sept langues.
8. **`EXTENSIONS.md` annonce « 31 processus tiers actifs »** dans sa prose et « 32, dont un réservé aux plugins officiels » dans son tableau. Le contrat dit `maxHostProcesses: 32`. Les deux se rejoignent, mais les deux formulations coexistent dans le même document. Retenir **32 au total, un réservé aux plugins officiels** sur le site.
9. **`EXTENSIONS.md` dit « Une seule installation travaille à la fois »** sans mentionner que **8 installations** peuvent attendre en file et que **32** sont conservées dans le suivi (`install_jobs/limits.rs:1-2`). Ce n'est pas une contradiction, c'est une information manquante. Vérifier avec l'équipe qu'aucune des deux valeurs n'est en cours de changement avant de les publier.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

10. **La fenêtre d'approbation** : disposition réelle, taille du pictogramme d'avertissement, lisibilité de la case à cocher du mode avancé, contraste du bouton « Faire confiance et activer » dans les six thèmes. À capturer dans les deux thèmes au minimum.
11. **La fenêtre d'ajout** : les quatre options côte à côte ou empilées selon la largeur, apparence du champ Git/npm quand il se déplie, lisibilité de l'avertissement en haut.
12. **Le suivi d'installation** : où exactement se trouve le bouton dans la barre supérieure, à quoi ressemble une demande de volume, où se trouve **Afficher la demande**, et ce que voit l'utilisateur quand la barre latérale est repliée. Le code place ce suivi dans le panneau de notifications (`app-layout.tsx:173`) ; l'ancrage visuel exact n'a pas été observé.
13. **La fiche d'extension** : longueur réelle de la page une fois toutes les sections présentes (informations, contributions, skills et ressources, interface, actions), et comportement du chemin de la source quand il est très long (`extension-detail.tsx:75`, valeur affichée en police à chasse fixe avec une infobulle).
14. **L'onglet Hôte** : apparence de la liste des diagnostics quand elle est longue, et lisibilité du bloc **Mode de récupération**.
15. **Le menu `+` du champ de saisie** : vérifier à l'écran que la liste devient bien défilable au-delà de huit lignes, et que l'ouverture de la fenêtre d'approbation depuis ce menu ne coupe pas le menu.
16. **Les messages d'erreur d'installation ne recevront pas de capture** : les provoquer suppose un disque plein, un dépôt injoignable ou une archive corrompue. Ils sont documentés d'après le code et les clés de traduction ; c'est un cas prévu par la convention de ce dossier.
