# Licences, cloud et matériel

## Filtre la licence

Tu vérifies séparément :

- licence du code ;
- licence des poids ;
- licence des sorties ;
- usage personnel, recherche ou commercial ;
- seuils liés à l'entreprise ;
- obligations d'attribution ;
- restrictions de redistribution.

Tu exclus par défaut MOIRAI R Small et TabPFN-TS-3 d'un usage commercial sans droit explicite. Tu vérifies les conditions propres de TiRex.

Tu ne déduis jamais la licence des poids depuis celle du dépôt.

## Filtre le cloud

Avant TimeGPT :

- Tu vérifies `allow_cloud_in_auto`.
- Tu vérifies que les données peuvent quitter la machine.
- Tu vérifies la clé, le compte et la variante.
- Tu vérifies région, contrat, rétention et opt-out.
- Tu estimes coût, quota et latence.
- Tu conserves une alternative locale si elle répond au besoin.

Tu ne promets pas une confidentialité que le contrat ne garantit pas.

## Évalue le matériel

Tu utilises les mesures vivantes avant les valeurs de catalogue :

- RAM libre ;
- VRAM libre ;
- stockage ;
- CPU ;
- GPU, MPS ou CUDA ;
- temps de chargement ;
- latence p50 et p95 ;
- débit ;
- taux d'échec.

Tu gardes une marge. Tu ne remplis pas toute la mémoire théorique.

## Choisis l'accélérateur

Tu ne préfères pas automatiquement un GPU. Pour un petit modèle ou un batch de un, le CPU peut être plus rapide.

Tu vérifies :

- OS réellement testé ;
- version PyTorch ;
- pilotes ;
- architecture ;
- précision ;
- kernels requis.

## Distingue estimation et mesure

Tu étiquettes :

- poids exact vérifié ;
- RAM estimée ;
- VRAM estimée ;
- mesure locale ;
- mesure fournisseur ;
- mesure sur un autre OS.

Tu ne compares pas directement des latences provenant de machines différentes.

## Gère le repli

Si le matériel devient insuffisant :

1. Tu relances `forecast_models`.
2. Tu choisis un candidat retourné plus compact.
3. Tu réduis le tournoi, pas le niveau de validation.
4. Tu proposes le cloud seulement s'il est autorisé.
5. Tu t'abstiens si aucune option sûre ne reste.
