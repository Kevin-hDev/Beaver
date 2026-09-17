# E05 — Lancement shell et nettoyage des ressources

Date du relevé : 17 septembre 2026
Commit examiné : `bc6e1067`

## Décision

Conserver les deux constructions de commande et le nettoyage explicite des échecs. Le partage proposé ne produit plus de simplification nette dans le code actuel.

Le lancement interactif doit rendre une `tokio::process::Command`, gérer un processus long, ses pipes et son annulation. La capture du profil utilise une `std::process::Command`, attend immédiatement sa sortie et possède sa garde de dossier temporaire. Construire toutes les commandes en `std` puis convertir le cas interactif déplacerait les mêmes réglages et ajouterait une conversion sans réunir les parcours.

Une garde automatique pour `tool_bash_process::prepare` serait également plus complexe que les sorties actuelles : son nettoyage est asynchrone, et les ressources apparaissent progressivement. Une garde devrait lancer des tâches depuis `Drop` ou porter plusieurs états optionnels. Le code explicite montre au contraire quelle ressource existe à chaque échec.

## Parcours et propriétaires

| Étape | Ressource acquise | Échec | Libération actuelle |
|---|---|---|---|
| Préparation du bac | Dossier temporaire et politique | exécutable ou politique indisponible | suppression immédiate du dossier |
| Annulation avant lancement | Dossier temporaire | session ou arrêt annulé | `cleanup_temp` |
| Stockage de sortie | Dossier temporaire puis fichier de sortie | préparation du stockage impossible | `cleanup_temp` |
| Lancement | Stockage et dossier temporaire | création ou admission refusée | nettoyage du dossier et `store.finalize(false)` |
| Lecture du PID | Processus possédé | PID absent | arrêt de l'enfant, attente, puis nettoyages |
| Acquisition des pipes | Processus, stockage et dossier | pipe manquant | arrêt de l'arbre, attente, puis nettoyages |
| Inscription de session | Toutes les ressources | registre plein ou erreur | arrêt de l'arbre, attente, puis nettoyages |
| Exécution | Session inscrite | sortie, arrêt, annulation ou délai | terminaison si nécessaire, drainage, finalisation du stockage et nettoyage du bac |

Le processus reste sous une seule autorité. `OwnedProcess::spawn_tokio` effectue l'admission ; les branches suivantes utilisent `tool_bash_platform::terminate_process_tree` uniquement après une admission réussie. Aucun second groupe de processus n'est créé par le nettoyage.

## Doublons évalués

### Environnement du helper

`protect_helper` et `protect_helper_std` répètent la suppression de trois variables d'injection pour deux types de commande qui n'ont pas de trait commun dans la bibliothèque standard. Ajouter un trait local, une macro ou une fonction à fermeture économiserait moins de lignes qu'il n'en introduirait. La liste `PROCESS_INJECTION_ENVS` est déjà l'autorité unique.

### Transport de la politique

`write` et `write_std` ne contiennent chacun que l'affectation d'une variable d'environnement. La sérialisation, la limite, l'écriture privée et le condensat sont déjà dans la fonction commune `store`. Il n'y a pas de logique métier dupliquée à extraire.

### Construction de la commande

Les arguments communs du helper sont courts, mais la capture de profil ajoute son marqueur, son dossier courant et une propriété de nettoyage différente. Le parcours disque complet contourne le helper et, sur macOS, passe par la garde du parent. Une fonction à options cacherait ces différences sans réduire le nombre de branches.

### Nettoyage de `tool_bash_process`

Les appels se répètent, mais ils ne portent pas tous les mêmes ressources. La finalisation du stockage commence seulement après sa création ; l'arrêt par arbre commence seulement après l'obtention du PID ; l'attente de l'enfant est obligatoire après sa terminaison. Une fonction universelle aurait plusieurs paramètres optionnels et reproduirait cette machine d'états.

## Garanties à conserver

- aucune commande sans le bac prévu, sauf autorisation explicite de disque complet ;
- suppression des variables d'injection avant le lancement du helper ;
- admission unique par `OwnedProcess`, avec groupe de processus ou Job natif ;
- aucun pipe perdu après un lancement partiel ;
- arrêt et attente de l'enfant avant de rendre une erreur lorsque le PID est connu ;
- sortie bornée et finalisée, y compris après annulation ou délai ;
- nettoyage borné des anciens dossiers au démarrage ;
- capture de profil synchrone indépendante du processus interactif.

## Preuves par plateforme

Sur macOS, les tests de bac, de garde parent, de groupe de processus, de pipes, de cycle de vie et d'annulation couvrent le parcours natif. Sous Windows, les tests dédiés vérifient la création sans console, le Job via `OwnedProcess`, le profil ACL et le nettoyage des enregistrements. La CI Windows reste la preuve d'exécution nécessaire pour ces appels natifs ; une exécution macOS ne peut pas la remplacer.

Toute future modification de ce code devra vérifier au minimum : échec de création, refus d'admission, PID absent, pipe absent, registre saturé, arrêt demandé, annulation, délai, sortie normale et nettoyage du dossier. La preuve Windows doit inclure le mode de tests sans CEF, qui n'a pas de rapport avec l'isolation shell mais partage la matrice CI.

## Conséquences pour le plan

- E05 ne crée aucune correction autonome.
- Le bac, `OwnedProcess` et le stockage de sortie gardent chacun leur propriétaire.
- Une extraction ne sera reconsidérée que si un troisième type de commande reprend le même parcours complet, ressources et erreurs comprises.
- Un changement local peut toujours remplacer deux sorties strictement identiques par une petite fonction, mais aucune abstraction de lancement générale n'est prévue.

## Validation de l'étude

Les appels de préparation ont été suivis depuis `shell_sandbox::prepare_command` jusqu'à `OwnedProcess::spawn_tokio`, puis dans toutes les sorties de `tool_bash_process::prepare` et dans la finalisation de `tool_bash_process_run`. Les variantes environnement et transport ont été comparées ligne à ligne. Le constat 04-C8 est exact sur la répétition textuelle, mais son estimation ne tient pas compte des propriétaires distincts et ne justifie plus une modification.
