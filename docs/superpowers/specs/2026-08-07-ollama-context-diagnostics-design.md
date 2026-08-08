# Ollama Context Diagnostics Design

## Objectif

Rendre les dépassements de contexte Ollama compréhensibles sans modifier les outils choisis par l'utilisateur, et porter le palier matériel intermédiaire de 16 384 à 24 576 tokens.

## Périmètre

- Conserver strictement la sélection actuelle des outils natifs, extensions et outils découverts.
- Calculer les chiffres du diagnostic à chaque requête, après construction du prompt système final et après sélection finale des outils.
- Ne jamais coder en dur la taille du prompt ou des outils dans le message utilisateur.
- Ne pas entreprendre ici la future refonte de la stratégie de capacité ou de sélection des outils.

## Comportement attendu

Quand le contexte obligatoire ne tient pas, le backend calcule séparément :

- les tokens estimés du prompt système réellement injecté ;
- les tokens estimés des rapports obligatoires réellement injectés ;
- les tokens estimés des définitions des outils réellement envoyées ;
- leur total obligatoire ;
- la fenêtre de contexte configurée ;
- la limite d'entrée après la réserve de réponse.

Il renvoie un code d'erreur stable accompagné uniquement de ces nombres sûrs. Le frontend reconnaît ce code et affiche un message localisé indiquant les valeurs réelles. Toute erreur inconnue conserve le message générique existant afin de ne pas exposer une erreur interne brute.

Exemple de forme, avec valeurs dynamiques :

> Le contexte obligatoire dépasse la capacité du modèle : prompt système {systemTokens} tokens + rapports obligatoires {reportTokens} tokens + outils actifs {toolTokens} tokens = {requiredTokens} tokens, pour une limite d'entrée de {maxInputTokens} tokens sur une fenêtre de {contextWindow} tokens.

La partie « rapports obligatoires » est omise lorsqu'elle vaut zéro, afin de ne plus faire croire qu'un sous-agent est impliqué dans une session qui n'en utilise pas.

## Limite matérielle Windows

Le palier utilisé lorsque la VRAM détectée est comprise entre 12 Go inclus et 24 Go exclus passe de 16 384 à 24 576 tokens. Les paliers bas (8 192) et haut (32 768) restent inchangés.

## Sécurité et internationalisation

- Aucun chemin, contenu de prompt, définition d'outil, nom de fichier ou erreur technique brute n'est affiché.
- Les valeurs numériques sont bornées et proviennent du calcul interne de la requête.
- Le texte visible est fourni dans les sept langues prises en charge.
- Les erreurs inconnues restent génériques.

## Tests

- Test Rust du nouveau palier intermédiaire à 24 576.
- Tests Rust du calcul dynamique avec et sans rapport obligatoire.
- Test Rust prouvant que les outils désactivés ne sont pas comptés puisqu'ils ne figurent pas dans la liste finale.
- Tests frontend du décodage validé du diagnostic et du repli générique pour toute valeur invalide ou erreur inconnue.
- Vérification des sept traductions.

