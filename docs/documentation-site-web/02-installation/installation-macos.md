# Installation sur macOS

**Emplacement site** — Démarrage › Installation › macOS
**Répond à** — « Comment j'installe Beaver sur mon Mac, et pourquoi macOS me met en garde ? »
**Sources** — `install.sh` (lignes 107-140, 166-175), `CROSS-PLATFORM.md`, `.github/workflows/release.yml`
**Vérification** — Vérifié dans le code : toutes les étapes ci-dessous sont lues dans `install.sh`

---

## Plan de page proposé

1. Méthode recommandée — le script d'installation
2. Ce que fait le script
3. Méthode alternative — le fichier `.dmg`
4. Gatekeeper et l'absence de signature
5. Choisir un autre dossier d'installation
6. Mettre à jour
7. Désinstaller

---

## Contenu

### 1. Méthode recommandée — le script d'installation

Commande à publier telle quelle :

```bash
(
  installer="$(mktemp /tmp/beaver-bootstrap.XXXXXXXX)" &&
  trap 'rm -f "$installer"' EXIT &&
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error \
    --output "$installer" \
    https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.sh &&
  bash "$installer"
)
```

Deux points à expliquer, sinon la commande a l'air gratuitement compliquée :

- Elle télécharge le script dans un fichier temporaire au lieu de le passer directement à `bash`. Le fichier est supprimé à la sortie, quoi qu'il arrive.
- Elle impose HTTPS et TLS 1.2 au minimum.

**Le script est réservé à une première installation.** Il refuse de remplacer une installation existante — voir section 6.

### 2. Ce que fait le script

Décrire les étapes : c'est un script qui demande le mot de passe administrateur, l'utilisateur a le droit de savoir ce qu'il exécute.

1. Détecte le système et l'architecture. **N'accepte que `Darwin:arm64` ou `Darwin:aarch64`** — Apple Silicon uniquement. Tout le reste échoue avec « Système non pris en charge ».
2. Interroge l'API GitHub Releases, réponse plafonnée à **512 Kio**.
3. Exige la présence de l'asset Beaver et du fichier `update-manifest.json`.
4. Suit **trois redirections au maximum**, et uniquement vers `release-assets.githubusercontent.com`.
5. Télécharge le `.dmg` avec un plafond de **2 Gio**, puis vérifie sa taille et son **SHA-256** contre le manifeste.
6. Demande le dossier d'installation, `/Applications` par défaut.
7. Monte le `.dmg` en lecture seule, sans l'afficher dans le Finder.
8. **Vérifie le contenu du bundle** avant de l'installer : présence de `Contents/Info.plist` et de `Contents/MacOS/cl-go-dash`, absence de liens symboliques, identifiant `com.clgo.dash`, exécutable `cl-go-dash`.
9. Copie vers un dossier temporaire à nom aléatoire, revérifie le bundle copié, puis le déplace en place. En cas d'échec à n'importe quelle étape, tout est nettoyé.
10. Démonte le `.dmg` et lance l'application.

Mentionner que le script **demande une élévation de privilèges** pour écrire dans `/Applications`.

### 3. Méthode alternative — le fichier `.dmg`

- Le `.dmg` est attaché à chaque release GitHub, nommé `Beaver_X.Y.Z_aarch64.dmg`.
- Téléchargement, ouverture, glisser-déposer vers Applications : la manipulation macOS habituelle.
- **Inconvénient réel** : un fichier téléchargé par un navigateur reçoit l'attribut de quarantaine de macOS, et Gatekeeper le bloque. Le script `install.sh` passe par `curl`, ce qui évite ce marquage.

Présenter cette méthode comme le repli, pas comme le chemin principal.

### 4. Gatekeeper et l'absence de signature

À dire franchement, sans tourner autour :

- **L'application n'est pas signée par un certificat de développeur Apple.**
- macOS affiche donc un avertissement à l'ouverture d'un `.dmg` téléchargé par navigateur : application d'un développeur non identifié.
- Contournement standard : clic droit sur l'application puis « Ouvrir », et confirmer. Ou bien Réglages Système › Confidentialité et sécurité › « Ouvrir quand même ».
- **Le script d'installation évite le problème** parce que `curl` ne pose pas l'attribut de quarantaine.

Ne pas minimiser : un utilisateur qui voit cet avertissement sans explication préalable referme la fenêtre. La page doit l'annoncer avant qu'il le rencontre.

### 5. Choisir un autre dossier d'installation

- Le script propose `/Applications` et attend une confirmation par Entrée.
- Un autre chemin absolu peut être saisi. `~` et `~/…` sont acceptés et développés.
- Le chemin est refusé s'il dépasse 1024 caractères, s'il n'est pas absolu, s'il contient des caractères de contrôle ou un segment `..`.

### 6. Mettre à jour

- **Le script d'installation ne met pas à jour.** Il échoue avec « Une application est déjà installée. Utilise sa mise à jour intégrée. »
- La détection couvre `Beaver.app` **et** `CL-GO.app` — l'ancien nom de l'application — y compris sous forme de lien symbolique.
- Les mises à jour passent par la fonction intégrée à l'application. Renvoyer vers *Mise à jour*.

### 7. Désinstaller

Voir *Points à confirmer* : la procédure complète n'est pas documentée dans le dépôt.

Ce qui est certain :
- L'application se supprime en jetant `Beaver.app` du dossier d'installation.
- Les données restent dans `~/.local/share/cl-go-dash/` et ne sont pas supprimées avec l'application.
- La clé maîtresse du coffre reste dans le trousseau macOS.

---

## Tableaux

### Tableau — Les deux méthodes

| | Script `install.sh` | Fichier `.dmg` |
|---|---|---|
| Vérification SHA-256 | Automatique | À faire soi-même |
| Avertissement Gatekeeper | Évité | À contourner manuellement |
| Choix du dossier | Proposé | Libre |
| Remplace une installation existante | Non, refuse | Oui |
| Lance l'application à la fin | Oui | Non |

---

## Encadrés

**Encadré « Apple Silicon uniquement »** — avertissement.
> Beaver est distribué pour les Mac à puce Apple Silicon (M1 et suivantes). Sur un Mac Intel, le script s'arrête avec « Système non pris en charge ».

**Encadré « Application non signée »** — information, à placer en section 4.
> Beaver n'est pas signé par un certificat de développeur Apple. En passant par le script d'installation, vous n'aurez pas d'avertissement. En téléchargeant le `.dmg` depuis un navigateur, macOS en affichera un : faites un clic droit sur l'application, puis « Ouvrir ».

**Encadré « Mise à jour »**
> Le script d'installation ne sert qu'à la première installation. Ensuite, utilisez la mise à jour intégrée à l'application.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Système non pris en charge » | Mac Intel, ou architecture non reconnue | Aucune : pas de build Intel |
| « Une application est déjà installée » | `Beaver.app` ou `CL-GO.app` présent | Utiliser la mise à jour intégrée, ou supprimer l'ancienne installation |
| Avertissement de développeur non identifié | `.dmg` téléchargé par navigateur, attribut de quarantaine | Clic droit › Ouvrir, ou passer par le script |
| « Installation impossible » | Échec de vérification du bundle, du montage ou de la copie | Relancer ; si ça persiste, vérifier l'espace disque et les droits sur le dossier choisi |
| « Impossible de récupérer la version » | API GitHub injoignable | Vérifier la connexion et un éventuel proxy |

---

## Renvois

- *Prérequis* — compatibilité matérielle
- *Premier lancement* — ce qui se passe ensuite
- *Mise à jour* — la mise à jour intégrée
- *Dépannage › Installation* — les cas non couverts ici

---

## Points à confirmer

- **La procédure de désinstallation complète.** Rien dans le dépôt ne la décrit. Établir et vérifier : suppression de l'application, du dossier de données `~/.local/share/cl-go-dash/`, et de l'entrée du trousseau. Un utilisateur qui désinstalle sans nettoyer le trousseau y laisse la clé maîtresse de son coffre.
- **Le comportement au lancement d'une application non signée après installation par script.** L'installation évite la quarantaine, mais confirmer qu'aucun avertissement n'apparaît aux lancements suivants.
- **Le nom exact du binaire visible par l'utilisateur.** L'exécutable interne s'appelle `cl-go-dash`, l'application `Beaver.app`. Vérifier ce qui apparaît dans le moniteur d'activité, et si ça mérite une note — voir un processus au nom inconnu inquiète légitimement.
- **La version minimale de macOS.** Non définie dans `tauri.conf.json`.
