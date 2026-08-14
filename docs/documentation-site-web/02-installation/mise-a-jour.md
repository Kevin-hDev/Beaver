# Mettre à jour Beaver

**Emplacement site** — Démarrage › Mise à jour (ou Référence › Mises à jour)
**Répond à** — « Comment Beaver se met à jour, qu'est-ce qui est vérifié, et que faire si ça casse ? »
**Sources** — `CROSS-PLATFORM.md` (lignes 3-58), `src-tauri/src/commands/app_update.rs`, `app_update_install.rs`, `app_update_manifest.rs`, `app_update_source.rs`, `app_update_notes.rs`, `services/app_update_helper.rs`, `services/update_health.rs`, `src/hooks/use-update-checker.ts`, `src/components/layout/update-notifications.tsx`
**Vérification** — Issu de `CROSS-PLATFORM.md`, recoupé avec les noms de fichiers présents dans le dépôt ; le détail du processus n'a pas été relu ligne à ligne

---

## Plan de page proposé

1. Comment les mises à jour sont détectées
2. Ce qui est vérifié avant installation
3. L'installation selon le système
4. Si l'installation échoue
5. Les mises à jour des modèles Ollama
6. Ce qui n'est jamais touché

---

## Contenu

### 1. Comment les mises à jour sont détectées

- Beaver interroge la **dernière release publiée du dépôt GitHub officiel**.
- Vérification **au lancement, puis toutes les heures**.
- La version locale est comparée au tag de la release.
- Quand une mise à jour existe, une **notification** apparaît, avec une barre de progression pendant le téléchargement.

Point à expliquer : **le dépôt doit être public** pour que cette vérification fonctionne sans authentification. C'est une contrainte assumée du projet.

### 2. Ce qui est vérifié avant installation

C'est la partie qui mérite d'être détaillée sur le site : elle explique pourquoi une mise à jour peut être refusée.

L'asset attendu dépend du système : `.dmg` sur macOS, `.deb` sur Linux, `-setup.exe` sur Windows.

Beaver exige que le fichier `update-manifest.json` de la release confirme **quatre éléments** :

- la même **version** que celle annoncée ;
- le même **nom de fichier** ;
- la même **taille** ;
- l'**empreinte SHA-256** attendue.

Si un seul de ces éléments ne correspond pas, la mise à jour n'est pas proposée. Le téléchargement est plafonné en taille, et l'empreinte est revérifiée après réception.

**Principe à énoncer** : en cas de doute, Beaver refuse de se mettre à jour plutôt que d'installer un fichier non vérifié.

### 3. L'installation selon le système

Élément commun aux trois systèmes, à expliquer d'abord : Beaver copie un **programme d'assistance autonome** dans un fichier temporaire, se ferme, et laisse ce programme faire le travail. L'assistance attend la fermeture complète de l'application avant de toucher quoi que ce soit, et n'utilise **aucun shell intermédiaire**.

Point important pour les installations personnalisées : **la mise à jour remplace l'installation réellement lancée**, pas un chemin par défaut. Quelqu'un qui a installé Beaver ailleurs que dans le dossier proposé ne se retrouve pas avec deux copies.

Le détail par système est en section Tableaux.

### 4. Si l'installation échoue

Différence importante entre systèmes, à ne pas masquer :

- **macOS** — l'ancienne version est conservée jusqu'à confirmation du redémarrage, et **restaurée automatiquement** si cette confirmation n'arrive pas.
- **Linux et Windows** — l'échec est détecté, mais le gestionnaire de paquets ou l'installeur a déjà remplacé les fichiers. **Il n'y a pas de retour arrière automatique.**

Dans ce dernier cas, la marche à suivre : réinstaller manuellement la dernière version connue comme fonctionnelle depuis sa release GitHub. **Les données utilisateur ne sont pas touchées** et sont reprises par la version réinstallée.

### 5. Les mises à jour des modèles Ollama

Mécanisme distinct de celui de l'application, à ne pas confondre :

- Beaver liste les modèles installés et interroge le registre pour chaque famille.
- La comparaison porte sur les **empreintes**, pas sur un numéro de version.
- Les modèles ayant une mise à jour disponible sont signalés.
- La vérification est relancée après chaque téléchargement de modèle.

### 6. Ce qui n'est jamais touché

À affirmer clairement, c'est la question que se pose tout le monde avant de cliquer :

- les conversations ;
- la mémoire ;
- les clés API et le coffre ;
- les réglages ;
- les modèles Ollama téléchargés.

Le dossier de données est indépendant de l'application installée.

---

## Tableaux

### Tableau — Installation par système

| Système | Fichier | Déroulement | Retour arrière automatique |
|---|---|---|---|
| macOS | `.dmg` | Montage en lecture seule, vérification du bundle, préparation de la copie, conservation de l'ancienne version, bascule, vérification du redémarrage | **Oui** |
| Linux | `.deb` | Installation par le gestionnaire de paquets avec élévation de privilèges, arguments passés séparément, vérification du redémarrage | Non |
| Windows | `-setup.exe` | Installeur silencieux dans le dossier d'installation courant, vérification du redémarrage | Non |

### Tableau — Les deux systèmes de mise à jour

| | Application | Modèles Ollama |
|---|---|---|
| Source | Releases GitHub | Registre Ollama |
| Comparaison | Numéro de version | Empreinte |
| Fréquence | Au lancement, puis chaque heure | Au lancement et après chaque téléchargement |
| Redémarrage | Oui | Non |

---

## Encadrés

**Encadré « Vos données sont conservées »**
> Une mise à jour ne touche ni vos conversations, ni votre mémoire, ni vos clés, ni vos modèles. Elles vivent dans un dossier séparé de l'application.

**Encadré « En cas d'échec sous Linux ou Windows »** — avertissement.
> Contrairement à macOS, il n'y a pas de retour arrière automatique sur ces deux systèmes. Si l'application ne redémarre pas après une mise à jour, réinstallez la dernière version qui fonctionnait depuis les releases GitHub : vos données seront reprises.

**Encadré « Ne pas utiliser le script d'installation pour mettre à jour »**
> Le script de première installation refuse de remplacer une installation existante. Passez par la mise à jour intégrée.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Aucune mise à jour proposée alors qu'une release existe | Un contrôle du manifeste a échoué : version, nom, taille ou empreinte | Attendre une correction de la release ; c'est un refus volontaire |
| La vérification échoue systématiquement | API GitHub injoignable, proxy ou pare-feu | Vérifier l'accès réseau à `api.github.com` |
| L'application ne redémarre pas après mise à jour | Installation défaillante | macOS restaure seul ; ailleurs, réinstaller manuellement |
| Deux versions installées | Ne devrait pas arriver : la mise à jour remplace l'installation lancée | Signaler le problème |
| Notification de mise à jour de modèle confondue avec celle de l'application | Deux mécanismes distincts | Voir le tableau des deux systèmes |

---

## Renvois

- *Installation* — les trois pages par système
- *Sécurité › Mises à jour vérifiées* — le détail des contrôles
- *Modèles › Ollama* — la mise à jour des modèles
- *Projet › Versions et changelog* — l'historique des versions

---

## Points à confirmer

- **Le processus complet n'a pas été relu ligne à ligne.** Cette page s'appuie principalement sur `CROSS-PLATFORM.md`, document daté d'avril 2026 alors que la version courante est 1.1.2. Les fichiers cités existent bien dans le dépôt, mais leur comportement doit être revérifié avant publication — c'est le sujet où une documentation fausse coûte le plus cher.
- **Contradiction sur le statut de la release.** `CLAUDE.md` affirme que « la release CI est créée directement en non-draft ». Le workflow `.github/workflows/release.yml` utilise pourtant `--draft`. Sans incidence pour l'utilisateur final, mais l'une des deux affirmations est fausse et doit être corrigée dans le dépôt.
- **La migration depuis l'ancien nom CL-GO.** Un mécanisme de version-pont est décrit pour les installations `v1.0.1`. Vérifier s'il concerne encore quelqu'un, et s'il mérite une mention publique ou seulement une note d'archive.
- **Le rôle exact de `update_health.rs`.** Le README mentionne des contrôles de santé et une installation « qui échoue fermé ». À détailler pour la page *Sécurité › Mises à jour vérifiées*.
- **Les notes de version affichées dans l'application.** Plusieurs fichiers y sont consacrés (`app_update_notes.rs`, `app_update_notes_wire.rs`) et `app-release-notes.json` existe à la racine en sept langues. Décrire ce que voit l'utilisateur au moment de la mise à jour.
- **La possibilité de désactiver la vérification automatique.** Non vérifiée. Un utilisateur hors ligne ou en réseau contraint voudra savoir si la vérification horaire peut être coupée.
