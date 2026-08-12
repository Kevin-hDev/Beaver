use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use tauri::Emitter;
use tokio::sync::{mpsc, Mutex, RwLock};

use super::agent_bridge::GatewayAgentBridge;
use super::channels::InboundMessage;
use super::refusal_audit::RefusalAudit;
use super::security::audit;
use super::security::rate_state::GatewayRateLimiters;
use super::service_channels::start_channel_accounts;
use super::service_consumer::consume_messages;
use super::service_state::{build_health, shared_state, GatewayState};
use super::types::GatewayHealth;
use super::work_supervision::{GatewayWorkServices, GATEWAY_MESSAGE_QUEUE_CAPACITY};
use crate::app_exit::AppWorkSupervisor;
use crate::models::GatewayConfig;

pub(super) const GATEWAY_CONTROL_STOP_TIMEOUT: Duration = Duration::from_secs(5);

pub struct GatewayService {
    pub(crate) state: Arc<RwLock<GatewayState>>,
    pub(super) app_work: AppWorkSupervisor,
    pub(super) run: Mutex<Option<GatewayWorkServices>>,
    pub(super) active_cancel: StdMutex<tokio_util::sync::CancellationToken>,
    pub(super) app: StdMutex<Option<tauri::AppHandle>>,
}

impl GatewayService {
    pub fn new(app_work: AppWorkSupervisor) -> Self {
        let inactive = tokio_util::sync::CancellationToken::new();
        inactive.cancel();
        Self {
            state: shared_state(),
            app_work,
            run: Mutex::new(None),
            active_cancel: StdMutex::new(inactive),
            app: StdMutex::new(None),
        }
    }

    pub async fn start(&self, config: GatewayConfig, app: tauri::AppHandle) -> Result<(), String> {
        super::config_validation::validate(&config)?;
        if !config.enabled {
            return Err("Gateway désactivé".to_string());
        }
        let mut current_run = self.run.lock().await;
        if !self
            .stop_locked(
                &mut current_run,
                std::time::Instant::now() + GATEWAY_CONTROL_STOP_TIMEOUT,
            )
            .await
        {
            return Err("gateway-shutting-down".to_string());
        }

        audit::configure(&config.audit);
        let work = GatewayWorkServices::new(self.app_work.clone());
        let (refusal_audit, audit_receiver) = RefusalAudit::channel();
        work.spawn_audit(move |cancel| super::refusal_audit::run(audit_receiver, cancel))
            .map_err(|error| error.public_code().to_string())?;
        *self
            .active_cancel
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = work.cancellation_token();
        *self.app.lock().unwrap_or_else(|error| error.into_inner()) = Some(app.clone());
        let (sender, receiver) = mpsc::channel::<InboundMessage>(GATEWAY_MESSAGE_QUEUE_CAPACITY);
        let bridge;
        let channels_started;
        {
            let mut state = self.state.write().await;
            state.cancel = work.cancellation_token();
            state.channels.clear();
            state.adapters.clear();
            state.config = config.clone();
            state.limits = Arc::new(Mutex::new(GatewayRateLimiters::new(&config.rate_limits)));
            channels_started = start_channel_accounts(
                &mut state,
                &self.state,
                &config,
                &sender,
                &app,
                &work,
                &refusal_audit,
            );
            bridge = Arc::new(GatewayAgentBridge::new(
                state.limits.clone(),
                config.max_sessions as usize,
            ));
            let _ = app.emit("gateway-status-changed", build_health(&state));
        }

        if let Err(error) = channels_started {
            let _ = work
                .stop_and_wait(std::time::Instant::now() + GATEWAY_CONTROL_STOP_TIMEOUT)
                .await;
            return Err(error);
        }
        drop(sender);

        let message_work = work.clone();
        let run_cancel = work.cancellation_token();
        let state = Arc::clone(&self.state);
        let consumer_app = app.clone();
        let consumer_audit = refusal_audit.clone();
        if let Err(error) = work.spawn_consumer(move |consumer_cancel| async move {
            consume_messages(
                receiver,
                state,
                bridge,
                consumer_app,
                message_work,
                run_cancel,
                consumer_cancel,
                consumer_audit,
            )
            .await;
        }) {
            let _ = work
                .stop_and_wait(std::time::Instant::now() + GATEWAY_CONTROL_STOP_TIMEOUT)
                .await;
            return Err(error.public_code().to_string());
        }

        *current_run = Some(work);
        Ok(())
    }

    pub async fn health(&self) -> GatewayHealth {
        let state = self.state.read().await;
        build_health(&state)
    }

    pub async fn is_enabled(&self) -> bool {
        let state = self.state.read().await;
        state.config.enabled && !state.cancel.is_cancelled()
    }

    pub async fn config(&self) -> GatewayConfig {
        self.state.read().await.config.clone()
    }

    pub async fn update_config(&self, config: GatewayConfig) {
        audit::configure(&config.audit);
        self.state.write().await.config = config;
    }

    #[cfg(test)]
    pub(super) fn set_active_cancel_for_test(&self, cancel: tokio_util::sync::CancellationToken) {
        *self
            .active_cancel
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = cancel;
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
