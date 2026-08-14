# Pièces jointes

**Emplacement site** — Agent › Pièces jointes
**Répond à** — « Comment je donne un fichier à l'agent, et lesquels sont acceptés ? »
**Sources** — `src-tauri/src/services/attachment_access.rs` (lignes 9-14, 23-114), `src-tauri/src/commands/attachments.rs` (lignes 8-29), `attachment_access_tests.rs`
**Vérification** — Vérifié dans le code : limites, mécanisme d'autorisation, code d'erreur

---

## Plan de page proposé

1. Joindre un fichier
2. Les limites
3. Comment l'accès est autorisé
4. Ce qui se passe au redémarrage
5. Pièce jointe ou lecture de fichier

---

## Contenu

### 1. Joindre un fichier

Des fichiers peuvent être attachés à un message. L'agent y accède pour la conversation.

### 2. Les limites

| Limite | Valeur |
|---|---|
| Pièces jointes | **15** |
| Taille d'une pièce jointe | **20 Mo** |
| Longueur d'un chemin | 4 096 octets |

Un fichier au-delà de 20 Mo est refusé. Pour un fichier volumineux situé dans un dossier autorisé, mieux vaut le laisser où il est et laisser l'agent le lire avec ses outils — voir section 5.

### 3. Comment l'accès est autorisé

Mécanisme discret mais qui mérite d'être expliqué, parce qu'il explique un message d'erreur.

Joindre un fichier ne rend pas ce fichier librement accessible. Chaque chemin reçoit une **autorisation signée** :

- l'autorisation est calculée avec une clé conservée dans le **coffre chiffré** ;
- elle est **vérifiée à chaque accès** ;
- le chemin est **résolu** — liens et raccourcis compris — avant d'être autorisé ;
- une autorisation qui ne correspond pas donne l'erreur `attachment_access_denied`.

L'intérêt : une pièce jointe donne accès à **ce fichier précis**, et rien d'autre. Un chemin fabriqué ou modifié ne passe pas.

### 4. Ce qui se passe au redémarrage

Les autorisations peuvent être **restaurées** au lancement suivant, pour que les pièces jointes d'une conversation reprise restent accessibles.

Voir *Points à confirmer* : la portée exacte de cette restauration n'a pas été établie.

### 5. Pièce jointe ou lecture de fichier

Deux façons de donner un fichier à l'agent, à distinguer :

| | Pièce jointe | Lecture par l'agent |
|---|---|---|
| Le fichier peut être n'importe où | Oui | Non, il doit être dans un dossier autorisé |
| Taille | 20 Mo au maximum | Selon l'outil employé |
| Nombre | 15 par message | Sans limite fixe |
| Autorisation | Signée, par fichier | Portée d'accès disque |

**Le cas d'usage de la pièce jointe** est le fichier isolé, hors de vos dossiers de travail : un document reçu, une capture, un export téléchargé.

**Pour un fichier du projet en cours**, il est inutile de le joindre : l'agent le lit directement avec ses outils, sans limite de 20 Mo ni de nombre.

---

## Encadrés

**Encadré « Une pièce jointe donne accès à un seul fichier »**
> Joindre un fichier autorise l'accès à ce fichier précis, pas au dossier qui le contient. L'autorisation est signée et vérifiée à chaque lecture.

**Encadré « Inutile de joindre un fichier du projet »**
> Si le fichier se trouve dans un dossier auquel l'agent a accès, il peut le lire directement. La pièce jointe sert aux fichiers isolés, hors de vos dossiers de travail.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Un fichier est refusé | Plus de 20 Mo | Le laisser dans un dossier autorisé et demander à l'agent de le lire |
| Impossible d'ajouter une pièce de plus | Quinze au maximum | Envoyer en plusieurs messages |
| « Accès à la pièce jointe refusé » | Autorisation invalide, fichier déplacé ou supprimé | Joindre le fichier à nouveau |
| Une pièce jointe d'une ancienne conversation n'est plus lisible | Fichier déplacé, ou autorisation non restaurée | La joindre à nouveau |

---

## Renvois

- *Agent › Répertoire de travail* — la portée d'accès disque
- *Outils › Fichiers* — comment l'agent lit un fichier
- *Interface › Arbre de fichiers et prévisualisations*
- *Sécurité › Accès aux fichiers*

---

## Points à confirmer

Cette page est courte parce que le code lu couvre le contrôle d'accès, pas l'expérience utilisateur. Plusieurs points restent à établir :

- **Les formats acceptés.** Le mécanisme d'autorisation ne filtre pas par extension. Vérifier ce qui est réellement exploitable : images pour un modèle qui les comprend, texte, documents bureautiques ? La page est incomplète sans cette liste.
- **Comment on joint un fichier** : bouton, glisser-déposer, collage depuis le presse-papier ? Non relevé.
- **Le traitement des images.** Un modèle sans capacité visuelle ne peut rien en faire. Vérifier ce que Beaver fait dans ce cas — refus, avertissement, ou envoi silencieux qui échoue.
- **La portée de la restauration des autorisations** au redémarrage : toutes les conversations, ou seulement celles rouvertes ?
- **Les pièces jointes comptent-elles dans le contexte ?** Un fichier de 20 Mo ne peut pas y tenir. Savoir comment il est traité — extrait, résumé, tronqué — est nécessaire.
- **Le sort d'une pièce jointe quand la conversation est archivée ou clonée.**
