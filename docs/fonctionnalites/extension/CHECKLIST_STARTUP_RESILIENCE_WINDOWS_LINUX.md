# Validation différée — résilience des extensions sur Windows et Linux

Créée le 6 septembre 2026. **Statut : NON EXÉCUTÉ sur les deux systèmes.**

Cette checklist est l’autorité de suivi des essais restant à faire pour les correctifs `dd73e70a` et `4a350be1`. La revue du code est verte ; elle ne remplace pas ces essais avec un binaire installé. Les tests automatisés macOS déjà exécutés ne valident aucune case Windows/Linux.

Décisions et preuves précédentes : `git notes show dd73e70a` et `git notes show 4a350be1`. Si les notes manquent dans un nouveau clone : `git fetch origin refs/notes/commits:refs/notes/commits` (ne pas forcer si le clone possède déjà ses propres notes).

## Préparer chaque environnement

- [ ] Windows : machine ou VM disponible, instantané de départ, compte de test distinct des données personnelles.
- [ ] Linux : machine ou VM disponible, instantané de départ, compte de test distinct des données personnelles.
- [ ] Installer un véritable paquet Windows / Linux construit depuis un commit contenant les deux correctifs ; vérifier le SHA source dans les preuves de build, pas seulement le numéro de version.
- [ ] Noter OS exact, architecture, type de session graphique Linux (X11/Wayland), version de Beaver, format du paquet, SHA source et SHA-256 de l’installateur.
- [ ] Préparer une extension de test contrôlée avec outil, skill et ressource ; préparer séparément une extension de test remplaçant un outil natif. Aucun code tiers non vérifié.
- [ ] Disposer d’un fournisseur cloud réellement utilisable et d’un modèle Ollama installé. Prévoir un petit budget pour les appels réels ; un transport simulé ne valide pas ces cases.
- [ ] Repérer le répertoire réellement utilisé par `services::paths::data_dir()` et sauvegarder le profil de test complet. Actuellement : `.local/share/cl-go-dash` sous le répertoire personnel sur les deux OS ; ne pas supposer `%APPDATA%` sous Windows.
- [ ] Arrêter Beaver avant chaque préparation de fichier sur disque. Conserver les fichiers originaux et les droits pour restauration ; ne jamais préparer une panne sur le profil personnel.

Pour les registres refusés, conserver avant/après les octets ou SHA-256 de `extensions.json`, des sauvegardes présentes et des fichiers de l’extension de test. Chaque ligne doit avoir une preuve distincte par OS : capture, identifiant de session et diagnostic expurgé des secrets. Remplacer « À faire » par « Réussi », « Échec » ou « Bloqué — raison », avec un lien vers la preuve. « Non testé » ne vaut jamais « Réussi ».

## Matrice à exécuter deux fois

| ID | Préparation et action | Résultat attendu | Windows | Linux |
|---|---|---|---|---|
| 01 | Premier lancement sans registre ; envoyer un message puis utiliser un outil natif. | Démarrage normal, réponse reçue et outil exécuté. Aucune fausse alerte de compatibilité. | À faire | À faire |
| 02 | Profil existant v2 valide ; charger l’extension de test, inspecter puis utiliser son outil, son skill et sa ressource. Fermer et rouvrir Beaver. | Contributions utilisables et choix conservés après redémarrage ; absence de dégradation injustifiée. | À faire | À faire |
| 03 | Sur des copies distinctes de vrais profils v0 puis v1, lancer le nouveau binaire. | Migration ascendante réussie ; sauvegarde exacte du fichier source au premier démarrage. Suppression de cette sauvegarde seulement au démarrage suivant réussi. | À faire | À faire |
| 04 | Registre JSON valide avec version future (ex. 256), plus un contenu que le lecteur actuel ne comprend pas. Redémarrer ; envoyer deux messages successifs. | Extensions indisponibles, avertissement traduit demandant une version compatible ; conversation native utilisable. Registre, sauvegardes et fichiers d’extensions intacts après les deux tours et un nouveau redémarrage. | À faire | À faire |
| 05 | Introduire un `kind` inconnu dans un registre v2, à côté d’entrées connues. Refaire avec copies v0 et v1. | Refus du registre entier avant migration/nettoyage, même explication de compatibilité et repli natif. Aucune entrée ou fichier supprimé ; aucune nouvelle sauvegarde de migration créée. | À faire | À faire |
| 06 | JSON tronqué ; puis, dans un essai séparé, registre inaccessible en lecture avec les droits propres à l’OS. | Cause adaptée et générique, pas de chemin système ni de `request_start (unknown)`. Réponse native possible si la conversation reste enregistrable. Aucun remplacement du registre par un registre vide. | À faire | À faire |
| 07 | Pendant que le registre est refusé, tenter une activation, une désactivation et une installation depuis les réglages. | Aucune mutation du registre refusé ne réussit. Les refus sont visibles ; fichier et sauvegardes inchangés. | À faire | À faire |
| 08 | Refaire les cas 04–06 avec un fournisseur cloud réel, puis avec Ollama réel. Demander la lecture d’un petit fichier de test connu. | Réponse réellement reçue et contenu du fichier correct. Outils d’extensions, découverte, ressources et outils dynamiques inconnus/MCP non exposés dans le repli ; aucune extension exécutée. Conserver les preuves des deux transports. | À faire | À faire |
| 09 | Préparer une session existante avec extension remplaçant un outil natif, puis empêcher uniquement l’écriture de son fichier `extension-session-state/<id>.json`. Laisser `agent-sessions` inscriptible. | Avertissement propre à l’état d’extensions de la conversation ; réponse et outil natif fonctionnels avec schéma natif. Anciennes autorisations d’extensions non réutilisées. Restaurer les droits après l’essai. | À faire | À faire |
| 10 | Dans ce tour dégradé, vérifier les outils optionnels désactivés et le sous-ensemble autorisé d’un sous-agent. | Aucun outil ajouté par le repli et aucun contournement des permissions. Le plafond d’outils du fournisseur reste respecté. | À faire | À faire |
| 11 | Provoquer l’échec d’enregistrement du fichier de conversation dans un profil jetable avant la préparation du tour dégradé. | Le tour s’arrête ; aucun outil n’est exécuté sans possibilité d’enregistrer sa conversation. Ce refus est volontaire et distinct d’une simple notice non livrée. | À faire | À faire |
| 12 | Restaurer les fichiers/droits compatibles, redémarrer Beaver et reprendre la même conversation. | Retour à un catalogue valide ; extensions utilisables de nouveau après inspection normale ; aucune désactivation persistante créée par le seul garde du tour. | À faire | À faire |
| 13 | Afficher les avertissements en thèmes sombre et clair, en français puis dans les six autres langues. | Niveau avertissement visible, texte lisible et traduit, aucune clé brute ni erreur système. Les notices ordinaires restent de niveau information. | À faire | À faire |
| 14 | Après un tour dégradé réussi, fermer et rouvrir l’application puis consulter conversation et diagnostics. | Réponse conservée ; cause stable du repli retrouvée dans les diagnostics. Un tour réussi ne devient pas un échec de flux au rechargement. | À faire | À faire |
| 15 | Profil de test dont le chemin personnel contient espaces et caractères accentués ; refaire 02 et 09. | Lecture, écriture atomique et reprise fonctionnent sur les chemins réels de l’OS. Aucun fichier créé au mauvais endroit. | À faire | À faire |
| 16 | Quitter normalement après ces essais puis relancer. Vérifier les processus appartenant à Beaver. | Pas d’hôte d’extensions abandonné, pas de verrou résiduel empêchant le prochain démarrage. Ne pas arrêter de processus appartenant à d’autres applications. | À faire | À faire |

## Cas nécessitant une injection contrôlée

Ces cas ne doivent pas être déclarés réussis sur simple lecture de code. Si le binaire installé ne permet pas de produire la panne de façon contrôlée, noter « Bloqué » avec le moyen de test à préparer, sans modifier les protections du produit.

| ID | Cas à provoquer | Résultat attendu | Windows | Linux |
|---|---|---|---|---|
| 17 | Échec d’envoi de la seule notice de dégradation, journal de conversation encore fonctionnel. | L’échec isolé de notice ne bloque pas le tour ; avertissement technique expurgé et cause persistée. Une panne permanente de tout le canal UI est un autre cas. | À faire | À faire |
| 18 | Enregistrement des préférences de découverte réussi puis reconstruction du catalogue en échec. | Ancien catalogue retiré, mutations refusées, aucun ancien ordre servi comme valide ; restauration au redémarrage avec les préférences enregistrées. | À faire | À faire |
| 19 | Retour du registre pendant un tour déjà dégradé ; si le harnais le permet, chevauchement de deux détenteurs du garde. | Les extensions restent interdites pour la durée du tour ; la fin d’un détenteur ne libère pas le garde de l’autre. Le prochain tour réévalue normalement l’état. | À faire | À faire |

## Fiche de preuve à dupliquer par OS

- OS / architecture / session graphique : à renseigner.
- Date / testeur : à renseigner.
- Version / SHA source / SHA-256 du paquet : à renseigner.
- Mode d’installation et chemin du profil de test : à renseigner (expurger les informations personnelles avant partage).
- Fournisseur cloud / modèle et modèle Ollama utilisés : à renseigner, sans clé ni token.
- Résultat par ID et emplacement des preuves : compléter la matrice.
- Écarts, issue ou commit correctif associé : à renseigner.
- Profil, droits et fichiers restaurés après essais : à confirmer.

## Critère de clôture

Les deux colonnes sont indépendantes. Clore la validation d’un OS seulement quand les parcours requis sont exécutés sur son binaire installé et leurs preuves relues. Tout échec ou cas bloqué reste explicitement ouvert. Ne pas reporter automatiquement le succès d’un OS sur l’autre, ni d’un binaire de développement sur un paquet installé.

Cette liste porte sur la résilience des extensions ; elle ne prétend pas remplacer toute la recette multiplateforme de Beaver. Le binaire 1.2.1 historique ne reçoit pas rétroactivement les correctifs.
