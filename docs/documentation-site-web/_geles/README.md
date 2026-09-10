# Chantiers gelés

Briefs mis de côté parce que la fonctionnalité correspondante n'est pas finalisée.

**Ne pas les publier.** Documenter un comportement qui va changer produit une documentation fausse le jour de la sortie, et personne ne pense à la relire.

Les fichiers portent l'extension `.gele` pour qu'aucun outil de génération ne les ramasse par erreur.

**État au 9 septembre 2026 — deux chantiers gelés, un dégelé.**

| Chantier | État |
|---|---|
| Mode Plan | **Gelé** — décision produit : le mode Plan va être modifié |
| Compression du contexte | **Dégelé le 10 septembre 2026** — refonte terminée, brief écrit dans `04-agent/compression.md` |
| Extensions (4 briefs) | **Dégelé le 9 septembre 2026, briefs écrits le jour même** — voir `07-integrations/extensions-*.md` |

La fiche Extensions reste dans ce fichier pour garder la trace du gel et de sa levée. Elle n'interdit plus rien.

---

## Mode Plan — `plan-mode.md.gele`

**Raison du gel** : le mode Plan doit être modifié. Dans son état actuel, il n'accorde pas assez d'autorisations au modèle : trop de choses sont bloquées pour qu'une exploration se déroule correctement.

**Ce que contient le brief** : le fonctionnement observé au moment de la rédaction — 19 outils autorisés, 3 conditionnels, les 7 états du parcours, les limites, le mécanisme de correction automatique.

**À refaire après modification** : la liste des outils autorisés est le cœur du sujet et va changer. Le reste du brief — parcours, limites, articulation avec les modes de permission — restera probablement valable.

**Le gel était justifié, et la liste a déjà bougé** (relevé le 9 septembre 2026, sans reprise du brief) : **21 outils autorisés** au lieu de 19, `search_extension_tools` a disparu du code au profit de trois entrées liées aux extensions, et une règle conditionnelle supplémentaire s'applique désormais par **effet d'extension**. Le reste du brief — sept états du parcours, quatre corrections automatiques, limites de titre et de contenu, vingt plans conservés, outils interdits, protection en Accès complet — a été revérifié et reste exact au chiffre près.

**Ce qui reste vrai quoi qu'il arrive** : le mode Plan protège même en Accès complet, les deux mécanismes se cumulent. C'est l'argument à conserver.

---

## Extensions — dégelé le 9 septembre 2026, quatre briefs à écrire

**Le gel est levé.** Il portait sur une implémentation interrompue en cours de route ; ce n'est plus le cas. Vérifié dans le dépôt :

- `EXTENSIONS.md`, à la racine, fait autorité sur le sujet et **doit être la source principale** des quatre briefs ;
- le module `services/extensions/` existe et est largement couvert de tests ;
- les deux outils de découverte d'extensions sont publiés et **intégrés au mode Plan** (`services/extensions/mod.rs`, repris dans `agent_local/tool_plan_guard.rs`) ;
- la politique de permission **par effet d'extension** est en place et testée : en mode Plan, seuls les outils d'extension en lecture seule sont autorisés.

**Fait le 9 septembre 2026** : les quatre briefs sont écrits (voir `07-integrations/`). Attention au troisième : la page « prompt système par une extension » a été **recadrée** — l'enquête dans le code a montré qu'aucune API d'extension ne touche au prompt système ; le brief documente ce qui est réellement possible et porte en tête une décision produit à trancher.

**Briefs prévus** :

- `extensions-centre.md` — découverte, catalogue, installation, mise à jour, désactivation, suppression
- `extensions-remplacer-un-outil.md` — substitution d'un outil natif, masquage, priorité, diagnostics
- `extensions-prompt-systeme.md` — réécriture du prompt système par une extension, portée, précédence
- `extensions-ecrire.md` — structure, hôte, canal de communication, source Git, limites et sécurité

**Niveau d'exigence attendu** : le plus élevé du site. Installer du code tiers qui remplace des outils et réécrit le prompt système engage la sécurité de l'utilisateur. Il devra être guidé pas à pas, avec les risques énoncés explicitement plutôt que mentionnés en note de bas de page. **Cette exigence reste entièrement valable maintenant que le gel est levé.**

---

## Comment reprendre un chantier gelé

1. Vérifier que la fonctionnalité est finalisée.
2. Relire le brief gelé : ce qui reste vrai, ce qui a changé.
3. **Relire le code**, sans se fier au brief — c'est précisément parce que le code allait changer qu'on a gelé.
4. Réécrire, retirer l'extension `.gele`, remettre le fichier à sa place et cocher la case dans `differents-points-a-traiter.md`.

---

## Compression du contexte — DÉGELÉ le 10 septembre 2026

**Gel levé** : Kevin a déclaré la refonte terminée le 10 septembre 2026. Le brief `04-agent/compression.md` a été écrit le jour même sur l'état du code post-refonte, et la page du site `agent-compression` est publiée. Cette entrée est conservée pour l'historique.

**Brief prévu** : `04-agent/compression.md` — quand la compression se déclenche, ce qu'elle résume, ce qu'elle conserve, ce que l'utilisateur voit, et ce qu'il peut régler.

**À noter** : le brief `04-agent/contexte.md` reste rédigeable et couvre le budget de contexte, l'élagage et l'écran d'usage. Il ne doit pas décrire la compression, seulement y renvoyer.
