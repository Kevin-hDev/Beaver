use super::*;
use crate::services::agent_local::session_archive::sort_for_display;
use crate::services::agent_local::types_session::AgentSessionMeta;
use chrono::{TimeZone, Utc};

fn meta(id: &str, minutes_ago: i64) -> AgentSessionMeta {
    let base = Utc.with_ymd_and_hms(2026, 8, 15, 12, 0, 0).unwrap();
    AgentSessionMeta {
        id: id.to_string(),
        name: id.to_string(),
        created_at: base - chrono::Duration::minutes(minutes_ago),
        updated_at: Some(base - chrono::Duration::minutes(minutes_ago)),
        archived_at: None,
        model: "m".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        message_count: 0,
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        parent_session_id: None,
        subagent_type: None,
        subagent_status: None,
        subagent_run_id: None,
        subagent_description: None,
        subagent_color_key: None,
        subagent_summary: None,
        subagent_last_activity: None,
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_root_session_id: None,
        git_branch: None,
    }
}

fn ids(metas: &[AgentSessionMeta]) -> Vec<String> {
    metas.iter().map(|m| m.id.clone()).collect()
}

#[test]
fn sans_rang_manuel_le_plus_recent_passe_devant() {
    let mut metas = vec![meta("vieux", 100), meta("recent", 1), meta("moyen", 50)];

    sort_for_display(&mut metas, &HashMap::new());

    assert_eq!(ids(&metas), vec!["recent", "moyen", "vieux"]);
}

#[test]
fn les_rangs_manuels_sont_respectes() {
    let mut metas = vec![meta("a", 1), meta("b", 50), meta("c", 100)];
    let ranks = HashMap::from([
        ("c".to_string(), 0),
        ("a".to_string(), 1),
        ("b".to_string(), 2),
    ]);

    sort_for_display(&mut metas, &ranks);

    assert_eq!(ids(&metas), vec!["c", "a", "b"]);
}

#[test]
fn une_conversation_jamais_placee_passe_devant_la_liste_rangee() {
    let mut metas = vec![meta("range_1", 100), meta("neuve", 1), meta("range_2", 50)];
    let ranks = HashMap::from([("range_1".to_string(), 0), ("range_2".to_string(), 1)]);

    sort_for_display(&mut metas, &ranks);

    assert_eq!(ids(&metas), vec!["neuve", "range_1", "range_2"]);
}

/* Deux listes numérotent leurs rangs à partir de zéro chacune. Le tri est
   global, mais l'affichage filtre par projet : ce qui compte est que l'ordre
   relatif tienne à l'intérieur de chaque liste une fois filtrée. */
#[test]
fn deux_listes_gardent_leur_ordre_apres_filtrage() {
    let mut metas = vec![
        meta("projet_second", 10),
        meta("hors_second", 20),
        meta("projet_premier", 30),
        meta("hors_premier", 40),
    ];
    let ranks = HashMap::from([
        ("projet_premier".to_string(), 0),
        ("projet_second".to_string(), 1),
        ("hors_premier".to_string(), 0),
        ("hors_second".to_string(), 1),
    ]);

    sort_for_display(&mut metas, &ranks);

    let projet: Vec<String> = ids(&metas)
        .into_iter()
        .filter(|id| id.starts_with("projet"))
        .collect();
    let hors: Vec<String> = ids(&metas)
        .into_iter()
        .filter(|id| id.starts_with("hors"))
        .collect();
    assert_eq!(projet, vec!["projet_premier", "projet_second"]);
    assert_eq!(hors, vec!["hors_premier", "hors_second"]);
}

#[test]
fn un_identifiant_invalide_est_refuse() {
    let refus = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(set(None, vec!["../ailleurs".to_string()]));

    assert!(refus.is_err());
}
