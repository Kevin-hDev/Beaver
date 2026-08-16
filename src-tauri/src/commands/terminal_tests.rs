//! Contrat des commandes du terminal : elles ne tiennent pas le fil qui dessine.
//!
//! Ouvrir un terminal ferme d'abord ceux dont le shell est déjà mort, et chaque
//! fermeture attend l'arrêt de son arbre de processus — plusieurs centaines de
//! millisecondes en tout. Tauri exécute une commande déclarée sans `async` sur
//! le fil qui dessine la fenêtre : l'application y restait figée pendant ce
//! ménage, et le panneau du terminal ne commençait à se déplier qu'ensuite, une
//! à deux secondes après le clic.
//!
//! Le mot `async` seul ne suffit pas : le travail à l'intérieur est bloquant, et
//! une tâche bloquante posée sur les fils du planificateur les affame. Il doit
//! donc partir sur les fils prévus pour attendre.

const SOURCE: &str = include_str!("terminal.rs");

#[test]
fn ouvrir_un_terminal_ne_tient_pas_le_fil_qui_dessine() {
    assert!(
        SOURCE.contains("pub async fn pty_spawn"),
        "pty_spawn doit être async, sinon Tauri l'exécute sur le fil de la fenêtre"
    );
}

#[test]
fn fermer_un_terminal_ne_tient_pas_le_fil_qui_dessine() {
    assert!(
        SOURCE.contains("pub async fn pty_kill"),
        "pty_kill doit être async : fermer coûte la même attente qu'ouvrir"
    );
}

#[test]
fn l_attente_part_sur_les_fils_prevus_pour_attendre() {
    let blocking = SOURCE.matches("spawn_blocking").count();
    assert!(
        blocking >= 2,
        "ouvrir et fermer sont bloquants : chacun doit passer par spawn_blocking, \
         vu {blocking} fois"
    );
}

/* Écrire et redimensionner ne touchent qu'un descripteur déjà ouvert : les
   rendre asynchrones ajouterait un aller-retour de planification à chaque
   touche frappée. */
#[test]
fn ecrire_et_redimensionner_restent_immediats() {
    assert!(SOURCE.contains("pub fn pty_write"));
    assert!(SOURCE.contains("pub fn pty_resize"));
}
