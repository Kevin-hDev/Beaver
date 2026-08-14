# Premier lancement

**Emplacement site** — Démarrage › Premier lancement
**Répond à** — « J'ai installé, je lance, que va-t-il se passer et qu'est-ce qui est créé sur mon disque ? »
**Sources** — `src-tauri/src/storage_migration.rs` (lignes 3-95), `src-tauri/src/services/paths.rs`, `src/components/onboarding/onboarding-screen.tsx`, `index.html` (écran d'attente), `CROSS-PLATFORM.md`
**Vérification** — Vérifié dans le code pour la structure créée, la migration et les valeurs par défaut

---

## Plan de page proposé

1. La séquence de démarrage
2. Le dossier de données
3. Ce qui est créé au premier lancement
4. La migration depuis une installation antérieure
5. Le téléchargement d'Ollama
6. Une seule instance à la fois
7. Le comportement du bouton de fermeture

---

## Contenu

### 1. La séquence de démarrage

Dans l'ordre :

1. **Écran d'attente** — l'icône de l'application s'affiche pendant le chargement de l'interface. Le fond s'adapte au thème clair ou sombre, retenu du lancement précédent. Il disparaît sans transition dès que l'application est prête.
2. **Initialisation du stockage** — création du dossier de données et de sa structure si elle n'existe pas, migration éventuelle depuis un ancien emplacement.
3. **Parcours d'accueil** — quatre à cinq étapes selon qu'Ollama est déjà présent. Voir *Onboarding*.
4. **Application** — la fenêtre principale.

### 2. Le dossier de données

**Le même chemin sur les trois systèmes** :

| Système | Chemin |
|---|---|
| macOS | `~/.local/share/cl-go-dash/` |
| Linux | `~/.local/share/cl-go-dash/` |
| Windows | `C:\Users\<utilisateur>\.local\share\cl-go-dash\` |

Deux points à expliquer, sinon ça surprend :

- **Le nom `cl-go-dash` est un identifiant historique**, conservé volontairement. L'application s'est appelée CL-GO avant de devenir Beaver ; changer le nom du dossier aurait obligé à déplacer les données de tous les utilisateurs existants. Beaver ne déplace, ne copie et ne recrée rien.
- **L'emplacement sous Windows est inhabituel.** Un utilisateur Windows cherchera dans `%APPDATA%` ; il faut lui dire où regarder.

### 3. Ce qui est créé au premier lancement

**Dossiers créés** — vérifié dans `init_base_structure` :

`memory/core`, `inbox`, `skills`, `agent-sessions`, `tool-results`, `translations`, `logs`

**Fichiers créés avec leur valeur par défaut** :

| Fichier | Valeur initiale |
|---|---|
| `config.json` | `{}` |
| `agent-settings.json` | `{"permissionMode":"auto"}` |
| `configured-providers.json` | `[]` |
| `favorite-models.json` | `[]` |
| `projects.json` | `[]` |
| `terminal-tabs.json` | `[]` |
| `inbox/pending.json` | `[]` |
| `personality-injection.json` | Quatre fichiers de personnalité, tous désactivés |

**Information importante à ne pas manquer** : le mode de permission par défaut est **Accès complet** (`auto` dans le fichier), c'est-à-dire que l'agent exécute ses outils sans demander confirmation. Ce n'est pas le mode le plus prudent. La page *Permissions* doit le dire, et cette page aussi : quelqu'un qui installe et lance immédiatement une tâche doit savoir dans quel mode il se trouve.

Un composant additionnel est également installé au premier lancement : le moteur de prévision local.

### 4. La migration depuis une installation antérieure

Trois migrations automatiques existent, chacune marquée par un fichier témoin pour ne pas se répéter :

| Origine | Témoin déposé |
|---|---|
| `~/.local/share/cl-go` (ancien nom du projet) | `.migrated-from-cl-go` |
| `~/Library/Application Support/cl-go-dash` (macOS) | `.migrated-from-appsupport` |
| `%APPDATA%\cl-go-dash` (Windows) | `.migrated-from-appdata` |

À dire à l'utilisateur : **rien n'est perdu et rien n'est à faire manuellement**. Si vous utilisiez CL-GO, ou une version antérieure de Beaver rangeant ses données ailleurs, elles sont reprises au premier démarrage.

### 5. Le téléchargement d'Ollama

Ollama n'est pas inclus dans l'application. Il est téléchargé au premier lancement.

**Détection** : Beaver considère Ollama comme disponible si le port **11434** est ouvert, ou si le binaire existe déjà dans `~/.local/share/cl-go-dash/ollama-bundle/`.

- **Si un démon Ollama tourne déjà** — parce qu'Ollama est installé séparément — Beaver l'utilise tel quel et ne télécharge rien.
- **Sinon**, l'écran de configuration propose le téléchargement.

**Archive téléchargée selon le système** :

| Système | Archive | Choix selon le GPU |
|---|---|---|
| macOS | `ollama-darwin.tgz` | Non — Metal est intégré |
| Windows | `ollama-windows-amd64.zip` | Non — archive unique |
| Linux | `ollama-linux-amd64.tar.zst` ou la variante ROCm | Oui — détection du constructeur du GPU |

**Contrôles appliqués au téléchargement** :

- taille minimale de **10 Mo** — en dessous, c'est une page d'erreur ou un téléchargement incomplet ;
- rejet si le type de contenu est `text/html` ;
- vérification que le binaire existe après extraction ;
- nettoyage automatique de l'archive temporaire et du dossier de destination en cas d'échec.

**L'étape peut être passée.** L'application reste utilisable avec des modèles distants uniquement.

### 6. Une seule instance à la fois

Lancer Beaver une seconde fois ne crée pas de deuxième fenêtre : le focus revient sur celle qui est déjà ouverte. Comportement identique sur les trois systèmes.

### 7. Le comportement du bouton de fermeture

Différence importante entre systèmes, à documenter :

| Système | Clic sur la fermeture | Effet |
|---|---|---|
| macOS | Pastille rouge | **Masque la fenêtre**, l'application reste dans le Dock. Un clic sur l'icône du Dock la ramène. `Cmd+Q` quitte réellement. |
| Windows | Croix | Quitte l'application et arrête Ollama |
| Linux | Croix | Quitte l'application et arrête Ollama |

Le comportement macOS est conforme aux usages du système, mais surprend qui vient de Windows : l'application semble fermée alors qu'elle tourne encore.

---

## Encadrés

**Encadré « Le mode de permission par défaut »** — avertissement, section 3.
> À la première installation, Beaver est en **Accès complet** : l'agent exécute ses outils sans demander confirmation. Si vous préférez valider chaque action, passez en **Demande d'approbation** avant de lancer votre première tâche.

**Encadré « Vos données existantes sont conservées »** — section 4.
> Si vous utilisiez déjà CL-GO ou une version antérieure, vos conversations et vos réglages sont repris automatiquement au premier démarrage.

**Encadré « Pourquoi ce nom de dossier »** — section 2.
> Le dossier s'appelle `cl-go-dash`, l'ancien nom du projet. Il est conservé tel quel pour que les installations existantes continuent de fonctionner sans déplacement de données.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| L'écran de configuration d'Ollama échoue | Pas de connexion, ou GitHub injoignable | Réessayer plus tard ; l'étape est reprenable |
| Le téléchargement s'arrête tout de suite | Fichier reçu inférieur à 10 Mo ou page HTML | Réseau intercepté par un portail captif ou un proxy |
| Sous Windows, le téléchargement de modèle échoue | Accès contrôlé aux dossiers bloque `ollama.exe` | Cliquer « Autoriser » dans la notification |
| L'application semble fermée mais tourne encore (macOS) | La fermeture masque la fenêtre | Cliquer l'icône du Dock, ou `Cmd+Q` pour quitter |
| Un second lancement ne fait rien | Instance unique | Normal : le focus revient sur la fenêtre existante |
| Données introuvables sous Windows | Emplacement inhabituel | `C:\Users\<utilisateur>\.local\share\cl-go-dash\` |

---

## Renvois

- *Onboarding* — le détail des étapes du parcours d'accueil
- *Ollama — runtime géré* — la gestion du démon après installation
- *Agent › Permissions* — changer le mode par défaut
- *Référence › Stockage local* — le contenu complet du dossier de données
- *Dépannage › Ollama*

---

## Points à confirmer

- **Le mode de permission par défaut reste un arbitrage produit ouvert.** `agent-settings.json` est créé avec `{"permissionMode":"auto"}`, soit **Accès complet**, alors que le mockup présente **Demande d'approbation** comme le mode recommandé au quotidien. Les libellés sont désormais fixés, mais la contradiction de fond demeure : soit le défaut change, soit la documentation assume qu'on démarre en Accès complet et le dit clairement. Ne pas laisser les deux discours coexister sur le site.
- **Le nom exact du composant de prévision installé au premier lancement.** Vérifier ce qui est réellement déposé et où, pour la page *Forecast*.
- **La migration Windows depuis `%APPDATA%`.** Le mécanisme existe dans le code ; confirmer qu'il fonctionne encore en 1.1.2 et sur quelles versions d'origine.
- **Le dossier `translations/`** créé au premier lancement — son rôle n'est documenté nulle part. À élucider avant d'écrire la page *Stockage local*.
- **Le dossier `inbox/`** et son fichier `pending.json` — non documentés non plus. Probablement lié aux messages entre agent parent et sous-agents, à confirmer.
- **La taille réelle du téléchargement d'Ollama** par système, pour pouvoir l'annoncer à l'utilisateur avant qu'il lance l'opération.
