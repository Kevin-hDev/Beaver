//! Déduplication atomique des invocations DevTools d'une fixture live.

use std::collections::VecDeque;
use std::future::Future;
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

const MAX_SEEN_RUN_IDS: usize = 64;

static SEEN_RUN_IDS: OnceLock<Mutex<VecDeque<Uuid>>> = OnceLock::new();

pub async fn start_once<T, F, Fut>(run_id: &str, starter: F) -> Result<T, String>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    claim(run_id)?;
    starter().await
}

fn claim(run_id: &str) -> Result<(), String> {
    let id = parse_run_id(run_id)?;
    let mut seen = seen_run_ids().lock().map_err(|_| unavailable())?;
    if seen.contains(&id) {
        return Err(unavailable());
    }
    if seen.len() == MAX_SEEN_RUN_IDS {
        seen.pop_front();
    }
    seen.push_back(id);
    Ok(())
}

fn parse_run_id(run_id: &str) -> Result<Uuid, String> {
    if run_id.len() != 36 {
        return Err(unavailable());
    }
    let id = Uuid::parse_str(run_id).map_err(|_| unavailable())?;
    (id.get_version_num() == 4 && id.hyphenated().to_string() == run_id)
        .then_some(id)
        .ok_or_else(unavailable)
}

fn seen_run_ids() -> &'static Mutex<VecDeque<Uuid>> {
    SEEN_RUN_IDS.get_or_init(|| Mutex::new(VecDeque::with_capacity(MAX_SEEN_RUN_IDS)))
}

fn unavailable() -> String {
    "Rapport de fixture indisponible".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn run_id(value: u128) -> String {
        Uuid::from_u128(value | (4_u128 << 76) | (2_u128 << 62))
            .hyphenated()
            .to_string()
    }

    #[tokio::test]
    async fn same_run_id_starts_once_while_distinct_ids_are_admitted() {
        let calls = AtomicUsize::new(0);
        let id = run_id(1);
        start_once(&id, || async {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })
        .await
        .unwrap();
        assert!(
            start_once(&id, || async {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .await
            .is_err()
        );
        start_once(&run_id(2), || async {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })
        .await
        .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn cache_is_bounded_and_evicts_oldest_run_id() {
        let mut seen = VecDeque::new();
        for value in 0..=MAX_SEEN_RUN_IDS as u128 {
            if seen.len() == MAX_SEEN_RUN_IDS {
                seen.pop_front();
            }
            seen.push_back(Uuid::parse_str(&run_id(value)).unwrap());
        }
        assert_eq!(seen.len(), MAX_SEEN_RUN_IDS);
        assert!(!seen.contains(&Uuid::parse_str(&run_id(0)).unwrap()));
    }

    #[test]
    fn malformed_or_non_v4_run_ids_fail_closed() {
        assert!(parse_run_id("not-a-uuid").is_err());
        assert!(parse_run_id(&Uuid::nil().hyphenated().to_string()).is_err());
    }
}
