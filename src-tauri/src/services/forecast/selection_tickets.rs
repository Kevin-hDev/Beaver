use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use subtle::ConstantTimeEq;
use uuid::Uuid;

use super::auto_selection::{AutoCandidate, AutoSelection};
use super::evaluation::types::BacktestIndexSummary;
use super::hardware_profile::ResourceFit;
use super::limits::MAX_SELECTION_TICKETS;

const TICKET_LIFETIME: Duration = Duration::from_secs(30 * 60);
static TICKETS: LazyLock<Mutex<VecDeque<SelectionTicket>>> =
    LazyLock::new(|| Mutex::new(VecDeque::with_capacity(MAX_SELECTION_TICKETS)));

struct SelectionTicket {
    id: Uuid,
    session_id: String,
    profile_id: String,
    fingerprint: String,
    basis: &'static str,
    candidates: Vec<CandidateProof>,
    created_at: Instant,
}

#[derive(Clone)]
struct CandidateProof {
    model_id: String,
    resource_fit: ResourceFit,
    reasons: Vec<String>,
    backtest: Option<BacktestIndexSummary>,
}

pub struct SelectionProof {
    pub basis: &'static str,
    pub resource_fit: ResourceFit,
    pub reasons: Vec<String>,
    pub backtest: Option<BacktestIndexSummary>,
}

pub fn proof_for_candidate(basis: &'static str, candidate: &AutoCandidate) -> SelectionProof {
    let candidate = candidate_proof(candidate);
    SelectionProof {
        basis,
        resource_fit: candidate.resource_fit,
        reasons: candidate.reasons,
        backtest: candidate.backtest,
    }
}

pub fn issue(
    session_id: &str,
    profile_id: &str,
    fingerprint: &str,
    selection: &AutoSelection,
) -> Result<String, String> {
    if Uuid::parse_str(session_id).is_err()
        || Uuid::parse_str(profile_id).is_err()
        || fingerprint.len() != 64
        || selection.candidates.len() > super::limits::MAX_AUTO_CANDIDATES
    {
        return Err(invalid());
    }
    let candidates = selection
        .candidates
        .iter()
        .map(candidate_proof)
        .collect::<Vec<_>>();
    let id = Uuid::new_v4();
    let mut tickets = TICKETS
        .lock()
        .map_err(|_| "Sélection Auto indisponible".to_string())?;
    evict_expired(&mut tickets);
    if tickets.len() == MAX_SELECTION_TICKETS {
        tickets.pop_front();
    }
    tickets.push_back(SelectionTicket {
        id,
        session_id: session_id.to_string(),
        profile_id: profile_id.to_string(),
        fingerprint: fingerprint.to_string(),
        basis: selection.basis,
        candidates,
        created_at: Instant::now(),
    });
    Ok(id.to_string())
}

pub fn consume(
    raw_id: &str,
    session_id: &str,
    profile_id: &str,
    fingerprint: &str,
    model_id: &str,
) -> Result<SelectionProof, String> {
    let id = Uuid::parse_str(raw_id).map_err(|_| invalid())?;
    let mut tickets = TICKETS.lock().map_err(|_| invalid())?;
    evict_expired(&mut tickets);
    let position = tickets
        .iter()
        .position(|ticket| ticket.id.as_bytes().ct_eq(id.as_bytes()).into())
        .ok_or_else(invalid)?;
    let ticket = tickets.remove(position).ok_or_else(invalid)?;
    if ticket.session_id != session_id
        || ticket.profile_id != profile_id
        || ticket.fingerprint != fingerprint
    {
        return Err(invalid());
    }
    let candidate = ticket
        .candidates
        .into_iter()
        .find(|candidate| candidate.model_id == model_id)
        .ok_or_else(|| "Le modèle ne faisait pas partie des candidats Auto".to_string())?;
    Ok(SelectionProof {
        basis: ticket.basis,
        resource_fit: candidate.resource_fit,
        reasons: candidate.reasons,
        backtest: candidate.backtest,
    })
}

fn candidate_proof(candidate: &AutoCandidate) -> CandidateProof {
    CandidateProof {
        model_id: candidate.model_id.clone(),
        resource_fit: candidate.resource_fit,
        reasons: candidate
            .reasons
            .iter()
            .map(|reason| (*reason).to_string())
            .collect(),
        backtest: candidate.evidence.clone(),
    }
}

fn evict_expired(tickets: &mut VecDeque<SelectionTicket>) {
    tickets.retain(|ticket| ticket.created_at.elapsed() <= TICKET_LIFETIME);
}

fn invalid() -> String {
    "Sélection Auto expirée ou invalide".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::forecast::auto_selection::{AutoCandidate, AutoSelection};
    use crate::services::forecast::hardware_profile::ResourceFit;

    #[test]
    fn unknown_ticket_fails_closed() {
        assert!(consume(
            &Uuid::new_v4().to_string(),
            "session",
            "profile",
            "fingerprint",
            "model"
        )
        .is_err());
    }

    #[test]
    fn ticket_is_session_bound_and_single_use() {
        let selection = AutoSelection {
            candidates: vec![AutoCandidate {
                model_id: "chronos-bolt-tiny".into(),
                compatibility: "recommended",
                resource_fit: ResourceFit::Comfortable,
                reasons: vec!["resources_checked"],
                interval_capability: super::super::interval_capability::for_model(
                    "chronos-bolt-tiny",
                ),
                backtest: None,
                evidence: None,
                estimated_ram_mb: 1_000,
            }],
            basis: "capabilities_and_resources",
            requested_model: None,
        };
        let session = Uuid::new_v4().to_string();
        let profile = Uuid::new_v4().to_string();
        let fingerprint = "a".repeat(64);
        let id = issue(&session, &profile, &fingerprint, &selection).unwrap();

        assert!(consume(&id, "other", &profile, &fingerprint, "chronos-bolt-tiny").is_err());
        assert!(consume(&id, &session, &profile, &fingerprint, "chronos-bolt-tiny").is_err());

        let id = issue(&session, &profile, &fingerprint, &selection).unwrap();
        let proof = consume(&id, &session, &profile, &fingerprint, "chronos-bolt-tiny").unwrap();
        assert_eq!(proof.resource_fit, ResourceFit::Comfortable);
        assert!(consume(&id, &session, &profile, &fingerprint, "chronos-bolt-tiny").is_err());
    }
}
