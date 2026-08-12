# Shutdown Milestone 2 Final Fixes Design

## Goal

Tu fermes les quatre régressions découvertes pendant la re-review sans recréer de travail détaché, de processus non possédé ni de seconde autorité.

## 1. Audit des refus gateway hors du fil réseau

Tu remplaces l’écriture d’audit directe de `try_enqueue` par une file bornée propre au run du gateway. Le fil réseau fait uniquement un `try_send` non bloquant. Un unique worker, enregistré dans `GatewayWorkServices`, vide la file et effectue l’écriture bloquante hors du runtime async.

Tu bornes cette file à 64 refus. Quand elle est pleine, tu incrémentes un compteur saturant ; tu ne lances aucun worker supplémentaire et tu n’écris aucun avertissement par message. Cette borne empêche un flood externe de créer du travail ou de la mémoire sans limite.

Tu fais passer le même expéditeur aux trois adaptateurs et au consommateur de messages. Ainsi, tous les refus gateway ont une seule autorité et aucun chemin chaud n’appelle directement le disque ou le coffre.

Le test bloque volontairement l’écrivain d’audit, attend qu’il soit réellement entré dans l’écriture, puis vérifie depuis un autre thread qu’un refus de file revient immédiatement. Il prouve la séparation réelle, pas seulement la présence de texte ou l’échec préalable du coffre.

## 2. Résultat de fermeture sans court-circuit

Tu ajoutes une petite autorité de combinaison qui attend toujours la fin du registre de travail avant de combiner son résultat avec celui des processus. Extensions et MCP passent par cette fonction.

La règle est : tu déclenches toutes les étapes de nettoyage prévues ; tu retournes `true` uniquement si chacune a réussi. Un échec précoce ne saute jamais une étape ultérieure.

## 3. Hôte d’extensions : échec fermé

Tu interdis toute mutation du registre d’extensions et tout redémarrage si `stop_host` retourne `false`. La désinstallation, la mise à jour et le redémarrage automatique renvoient alors l’erreur publique générique existante.

Tu conserves l’ancien hôte dans le slot tant que sa mort n’est pas confirmée. Cette propriété empêche un appel concurrent ou ultérieur de créer un second processus Node. Le premier arrêt confirmé libère le slot ; un arrêt incomplet reste réessayable.

Les tests injectent un arrêt refusé et prouvent que la mutation et le redémarrage ne sont jamais appelés.

## 4. Mesure GPU : une seule autorité

Tu gardes une seule opération de mesure : la sonde possédée, annulable et bornée. Lorsqu’elle réussit, elle publie un instantané `(total, utilisé)` dans le module `gpu_vram`. Tous les consommateurs synchrones lisent uniquement cet instantané ; ils ne lancent jamais de commande système.

Tu amorces l’instantané avec la sonde possédée avant le premier démarrage automatique d’Ollama. Le polling le rafraîchit ensuite. Si aucune mesure n’est disponible, les comportements sûrs existants restent inchangés : contexte minimal, compatibilité de modèle non bloquée et profil matériel inconnu.

Tu supprimes les anciennes fonctions de plateforme qui lançaient directement PowerShell, `nvidia-smi`, `sysctl` ou `system_profiler`. Le module GPU reste l’unique propriétaire de la mesure et de son cache.

## Erreurs et arrêt

Tu classes une file d’audit pleine comme récupérable : le message gateway est déjà refusé, le compteur local conserve le diagnostic et le fil réseau continue. Tu classes un hôte non confirmé mort comme fatal pour l’opération d’extension en cours : aucune mutation destructive ni relance n’est autorisée.

Tu dérives les attentes de l’échéance déjà fournie. Tu n’ajoutes aucun délai local concurrent.

## Acceptation

- Tu observes chaque nouveau test échouer avant son correctif.
- Un écrivain d’audit bloqué ne retarde pas le heartbeat ni les boucles réseau.
- Extensions et MCP attendent leur registre même après un échec de processus.
- Un arrêt d’hôte non confirmé empêche toute mutation et tout nouveau spawn.
- Aucun consommateur GPU hors `gpu_vram` ne lance de sonde système.
- Les fichiers de code gardent une responsabilité unique et restent sous 230 lignes.
- Les tests ciblés, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` et les suites du jalon passent avant commit final.
