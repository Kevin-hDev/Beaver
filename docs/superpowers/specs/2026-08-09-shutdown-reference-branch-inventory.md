# Inventaire de reprise — branche de référence de fermeture

## Autorité et photographie auditée

Ce document complète le [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md). Il empêche qu'une correction ou qu'un test utile disparaisse lorsque la branche `codex/fix-app-shutdown-lifecycle` sera remplacée par cinq PR propres.

La photographie auditée s'arrête au commit documentaire `42823ba` :

- ancêtre commun et `main` de comparaison : `d68ccdc37ad7b9839992ef88a38c93f35c8520ce` ;
- 31 commits propres à la branche ;
- 22 commits qui modifient le code ou la CI ;
- 9 commits exclusivement documentaires.

Chaque ligne de code ci-dessous reste ouverte jusqu'à ce que le jalon indiqué fournisse soit une correction équivalente avec ses tests, soit une décision d'abandon explicite et justifiée. Une simple ressemblance de code, un cherry-pick qui s'applique ou un test ancien encore vert ne suffit pas à fermer une ligne.

Deux contrôles directs sur ce `main` confirment le risque signalé par la review : `agent_chat_streams.rs` renvoie encore la limite de flux en français depuis le backend, et le scheduler ne possède ni `fire_once.rs` ni issue typée pour une revendication ponctuelle. Les lignes `887afe5` et `e835ef7` ne sont donc pas des doublons déjà intégrés.

## Règles de reprise

- **Réimplémenter** : conserver le scénario de panne et la preuve, mais écrire la correction avec les nouvelles autorités du contrat.
- **Réutilisation isolée possible** : comparer d'abord le commit à la tête de `main`, faire échouer le test correspondant, puis seulement reprendre le plus petit diff utile.
- **Mécanisme remplacé** : ne pas restaurer l'ancienne structure signalée par les reviews ; son comportement utile et son test restent obligatoires.
- Une ligne affectée à deux jalons n'est fermée qu'après les deux validations.
- Chaque PR met à jour la colonne **État** avec le commit de remplacement et les tests exécutés.
- La branche de référence n'est supprimable qu'après fermeture des 22 lignes et la review cumulative du jalon 4.

## Matrice exhaustive des 22 commits de code

| # | Commit | Acquis ou défaut à ne pas perdre | Jalon propriétaire | Décision de reprise | État initial |
|---:|---|---|---|---|---|
| 1 | `b1d666d` — arrêt déterministe | Sémantique croix/Quitter par OS, clic droit du tray préservé, annulation du démarrage, arrêt SearXNG/Ollama/extensions, ordre services → CEF → balayage, logs SearXNG bornés et nettoyés | J1 pour la fermeture ; J1B pour l'ordre et le confinement CEF ; J2 pour les services | Réimplémenter. Ne pas reprendre les délais indépendants de 3 s ni les démarrages détachés. Rejouer les tests de décision de fermeture, d'ordre CEF, d'annulation SearXNG et de balayage borné. | J1 repris par `b86499d`, puis durci par `f30ece8` : décisions de fermeture, menu tray, ordre post-boucle et panique bloquante couverts. J1B/J2 restent ouverts. |
| 2 | `d3c7011` — CI macOS native | Compilation et Clippy du backend CEF sur un runner macOS réel ; correction `cfg(unix)` révélée par ce job | J1B, puis revalidation J4 | Réutilisation isolée possible après comparaison avec la CI courante. Le job est un prérequis du jalon CEF et doit être vert dès le jalon 1B ; le jalon 4 le revalide sur l'ensemble final. Préparer CEF et exercer tous les targets, pas seulement recopier le YAML. | Ouvert J1B/J4 |
| 3 | `105ea70` — travaux annulables attendus | Un flux ou téléchargement annulé libère ses ressources avant la sortie ; l'ancien worker ne retire pas le nouveau ; abandon borné d'un worker qui ignore son jeton | J1 pour le registre ; J2 pour flux et téléchargements | Réimplémenter avec l'admission non clonable et la preuve de fin. Ne pas reprendre `TaskControl` comme autorité parallèle. Rejouer les tests de remplacement, file annulée et libération avant sortie. | Registre J1 repris par `b055e69` : fin normale, panique et abort couverts par `registry_tests`. Adoption flux/téléchargements J2 ouverte. |
| 4 | `a1efd6f` — démarrage SearXNG lié à l'app | Un arrêt pendant installation, attente de disponibilité ou spawn détruit et moissonne le processus déjà créé ; le handle reste la propriété du service | J2 | Réimplémenter via l'admission suivie et `stop_and_wait`. Garder les tests aux états `starting`, `running` et `stopping`. | Ouvert J2 |
| 5 | `4ce1b84` — helper de mise à jour validé | Une seule exception de sortie, validée par PID, parent, heure de démarrage et exécutable canonique ; guard local avant publication ; balayage qui n'exclut que cette identité | J2 | Conserver `UpdateHandoff` comme décidé dans le contrat. Réutilisation partielle possible après audit, sans reprendre les anciennes limites de profondeur ni faire confiance au PID seul. | Ouvert J2 |
| 6 | `50c5de9` — nettoyage avant sortie Tauri | États `Running/Cleaning/Ready`, fenêtres masquées, polling Ollama et file watcher annulables, CEF fermé après la boucle Tauri, sortie refusée tant que le nettoyage n'est pas prêt | J1 pour l'état ; J1B pour la frontière CEF ; J2 pour les services | Réimplémenter avec les échéances absolues et le watchdog. Ne pas exécuter les arrêts bloquants dans la future chronométrée. Rejouer le test d'ordre post-boucle. | État et frontière J1 repris par `b86499d`, transition concurrente et panique bloquante durcies par `f30ece8`. Preuve native CEF J1B et adoption services J2 ouvertes. |
| 7 | `0d67047` — fenêtres de course | Refus tardif des flux, téléchargements, gateway, Ollama et réveils ; persistance terminale avant publication sous-agent ; aucune mutation métier après le début de fermeture | J1 pour l'admission ; J2 pour l'adoption | Réimplémenter sous une seule admission atomique. Conserver les tests « refus avant enregistrement », « aucun effet de bord » et l'ordre exact sauvegarde/registre/événement. | Autorité d'admission J1 reprise par `b055e69` et branchée par `b86499d` ; `f30ece8` sérialise les demandes simultanées et ferme toute transition partielle. Adoption des producteurs J2 ouverte. |
| 8 | `5a39a18` — admission centralisée | Transition d'arrêt et admission atomiques ; code public stable `app-shutting-down` | J1 | Mécanisme remplacé : ne pas restaurer `AppWorkPermit`, qui ne prouvait pas la fin. Reprendre l'atomicité et le code public dans le nouveau guard suivi. | Fermé J1 par `b055e69`/`b86499d` et revalidé par `f30ece8` : garde non clonable, preuve de fin, code stable et propriétaire unique de la première fermeture. |
| 9 | `1967c45` — inscription transactionnelle des flux | Générations de flux, activation après persistance, diagnostic finalisé en cas de refus, ancien flux incapable de retirer le nouveau, attente du remplacement annulable | J2 | Réimplémenter autour du superviseur sans perdre `StreamRegistration` comme unité transactionnelle. Rejouer les six scénarios de course ajoutés par le commit. | Ouvert J2 |
| 10 | `b20bd83` — registre borné de tâches | Capacité dure, fermeture permanente, annulation, attente de libération des ressources et abandon borné | J1 | Réimplémenter dans le registre global de 128 slots avec génération. Les cinq tests du registre restent obligatoires, complétés par panique et réutilisation de slot. | Fermé J1 par `b055e69` : 128 slots fixes, générations, annulation et attente absolue ; sept `registry_tests`, répétés dix fois. |
| 11 | `eab61c1` — scheduler attendu | Boucle et exécutions enregistrées, admission fermée à l'arrêt, réveil ponctuel revendiqué avant appel provider | J2 | Réimplémenter avec le registre interne du scheduler. Ne pas reprendre l'absence de notification après `claim_once` ; les issues typées et la notification obligatoire viennent de `e835ef7`. | Ouvert J2 |
| 12 | `34e02af` — setup Ollama lié à la fermeture | Une seule installation suivie, annulation et attente, refus d'un setup tardif, démarrage annulé distinct de « déjà lancé » | J3 | Réimplémenter dans le gestionnaire transactionnel unique. Ne pas reprendre le délai local de 10 s ni l'état global séparé. | Ouvert J3 |
| 13 | `0221c96` — démarrages tardifs atomiques | Gateway et téléchargements refusés sans création d'entrée ni reconfiguration d'audit après fermeture | J2 | Réimplémenter avec une admission acquise avant tout effet de bord. Rejouer les tests de refus sans mutation. | Ouvert J2 |
| 14 | `c266c9c` — ordre explicite | Publication de fin de sous-agent après sauvegarde, parcours de processus borné/dédupliqué, ordre feuilles-vers-racine indépendant des PID | J2 | Réimplémenter avec un index parent/enfants construit une fois et le signal de groupe préalable. Conserver les tests d'ordre, cycles, borne et événements sous-agent. | Ouvert J2. `191de15` stabilise uniquement la preuve sauvegarde/publication du test ; aucun code de parcours processus n'est repris ni déclaré fermé. |
| 15 | `a563d73` — dernières courses | Inscription de flux regroupée, génération du setup Ollama empêchant un ancien cleanup d'effacer le nouveau, arbres Unix profonds complets | J2 pour flux/processus ; J3 pour setup | Réimplémenter. Ne pas restaurer les deux registres concurrents ; porter les scénarios de génération dans les autorités uniques des jalons 2 et 3. | Ouvert J2/J3 |
| 16 | `c92620b` — première installation transactionnelle | Staging séparé, validation avant commit, installation incomplète supprimée, installation déjà commitée conservée si le premier démarrage est annulé | J3 | Réimplémenter avec les noms modernes et le journal durable. Garder les tests avant/après commit et échec d'écriture du marqueur de version. | Ouvert J3 |
| 17 | `f3cdf63` — reprise de mise à jour Ollama | Conservation des deux versions pendant validation, succès qui nettoie seulement la sauvegarde, échec réel qui rollback, interruption qui diffère la décision | J3 | Mécanisme remplacé : abandonner l'inférence par présence de dossiers, reprendre tous les scénarios avec le journal à cinq phases et la sonde possédée. | Ouvert J3 |
| 18 | `e835ef7` — issues des réveils ponctuels | Résultats explicites exécuté/désactivé/annulé, annulation après consommation journalisée, désactivation bénigne silencieuse, notification après mutation, affichage traduit en sept langues | J2 | Réimplémenter intégralement ; ce correctif est absent de `main`. Rejouer les tests « inactif sans dispatch » et « annulé après claim », plus chaque mutation suivie d'une notification. | Ouvert J2 |
| 19 | `77d1efa` — nettoyage coordonné borné | Une seule échéance globale, phases parallèles, temps réservé au forcé, groupe Unix signalé avant l'instantané, index de processus borné | J1 pour les budgets ; J2 pour les processus | Mécanisme remplacé par `8/10/13/15`, un watchdog de processus et un tueur ultime précréé indépendant de ses appels OS. Le test ancien cède la main et ne prouve pas un vrai blocage : le remplacer par une opération synchrone réellement bloquée, puis bloquer aussi le watchdog de processus. Conserver les preuves de budget unique et d'ordre groupe/descendants. | Budgets J1 repris par `ae83ff4`, `fccb581`, `1b45777`, `b86499d` et `f30ece8` : appel bloquant, panique, CEF bloqué et watchdog bloqué couverts. Signalement/processus J2 ouverts. |
| 20 | `887afe5` — erreurs de flux traduites | Codes stables `active-stream-limit-reached` et `stream-replaced`, liste publique fermée côté interface, fallback générique, sept langues | J2 | Réutilisation isolée possible, mais le diff Rust dépend du nouveau flux du commit `1967c45`. Reproduire d'abord le défaut encore présent sur `main`, puis tester codes connus et erreur inconnue masquée. | Ouvert J2 |
| 21 | `7a8fef0` — annulation Ollama centralisée | Classification typée et partagée de fermeture/annulation/erreur ; suppression des comparaisons de textes et de cinq helpers divergents | J1 pour le vrai guard ; J3 pour Ollama | Mécanisme partiellement remplacé : ne pas reprendre le nom trompeur `AppRunToken` ni un jeton sans preuve de fin. Conserver l'autorité typée unique côté Ollama. | Garde suivie J1 reprise par `b055e69`/`b86499d`. Classification et transaction Ollama J3 restent ouvertes. |
| 22 | `cf322ef` — erreurs de revendication et validation Ollama récupérable | Scheduler : erreur de `claim_once` manquée journalisée au lieu d'être ignorée, tandis que l'état inactif reste bénin. Ollama : échec de spawn traité comme un échec de validation réel, démon externe distinct sans valider ni rollback, même décision au démarrage et à la commande. | J2 pour le scheduler ; J3 pour Ollama | Réimplémenter le chemin scheduler avec une issue typée : erreur journalisée et visible sans détail interne, état inactif silencieux. Pour Ollama, employer `OwnedStarted`, `OwnedAlreadyRunning`, `ExternalAvailable` et la sonde isolée. Rejouer les tests de revendication échouée/inactive, spawn impossible, démon externe et rollback repris. | Ouvert J2/J3 |

## Contrôle des neuf commits documentaires

Les sept premiers documents historiques restent consultables mais sont remplacés par le contrat unifié :

- `c6624c8`, `ca788a5` ;
- `393812c`, `29ff2c4`, `5a9899f` ;
- `ca140ad`, `beeeb66`.

Les commits `9f0e032` et `42823ba` portent le contrat initial et ses quatre jalons. Le durcissement `16931b1` puis le présent amendement conservent ce contrat, ajoutent le jalon 1B et deviennent la documentation de départ du jalon 1 avec le présent inventaire. Aucun ancien plan d'implémentation n'a autorité sur eux.

## Fermeture d'une ligne

Pour fermer une ligne, la Git note du jalon indique :

1. le hash du commit de référence ;
2. le comportement repris ou la raison précise de son abandon ;
3. le ou les tests qui échouaient sur le `main` de départ puis réussissent ; si le comportement est abandonné, la preuve reproductible qu'il est devenu impossible ou hors périmètre ;
4. les tests voisins exécutés pour vérifier l'absence de régression ;
5. le commit du jalon qui porte la correction.

Le jalon 4 compare enfin les 22 lignes à la tête finale de `main`. Une ligne partiellement reprise, un test supprimé sans équivalent ou une décision sans justification bloque la fusion finale.
