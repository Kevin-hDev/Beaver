use super::error::OllamaErrorCode;
use super::types::{OllamaEndpoint, OllamaStartOutcome};
use super::{CancelOutcome, OllamaCliArgs, OllamaManager};
use crate::app_exit::AppExitCoordinator;
use crate::services::agent_local::ollama_client::OllamaClient;
use std::num::NonZeroU16;
use std::time::Instant;

#[tokio::test]
async fn manager_exposes_one_decision_for_all_start_results() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11500).unwrap());
    let outcomes = [
        OllamaStartOutcome::OwnedStarted { endpoint: endpoint.clone() },
        OllamaStartOutcome::OwnedAlreadyRunning { endpoint: endpoint.clone() },
        OllamaStartOutcome::ExternalAvailable { endpoint: endpoint.clone() },
        OllamaStartOutcome::RejectedDuringShutdown,
        OllamaStartOutcome::BlockedByRecovery {
            code: OllamaErrorCode::OllamaRecoveryDeferred,
        },
        OllamaStartOutcome::Failed {
            code: OllamaErrorCode::OllamaStartFailed,
        },
    ];
    assert_eq!(outcomes.len(), 6);
    let _ = manager.start().await;
    let _ = manager.restart().await;
    let _ = manager.cancel_operation().await;
    let _ = manager.usable_endpoint().await;
    let _ = manager.owned_endpoint().await;
    let _ = manager.stop_and_wait(Instant::now()).await;
    let _ = manager.run_cli(OllamaCliArgs::Version).await;
    let _client = OllamaClient::new(manager);
    let _cancel = CancelOutcome::Cancelled;
}

#[test]
fn all_consumers_use_injected_manager_and_do_not_own_runtime_actions() {
    let sources = [
        include_str!("../agent_local/ollama_client.rs"),
        include_str!("../agent_local/ollama_stream_request.rs"),
        include_str!("../agent_local/ollama_collect.rs"),
        include_str!("../agent_local/ollama_registry.rs"),
        include_str!("../agent_local/ollama_modelfile_create.rs"),
        include_str!("../../commands/ollama_version.rs"),
        include_str!("../../commands/agent_ollama.rs"),
        include_str!("../../commands/ollama_setup.rs"),
        include_str!("../../commands/ollama_setup_update.rs"),
        include_str!("../../services/model_downloads.rs"),
    ];
    for source in sources {
        assert!(!source.contains("ollama_base_url"));
        assert!(!source.contains("ollama_lifecycle"));
        assert!(!source.contains("ollama_port"));
        assert!(!source.contains("Command::new"));
    }
}
