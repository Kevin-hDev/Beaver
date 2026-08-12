use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation,
    ServiceWorkDiagnostics, ServiceWorkSupervisor,
};
use std::future::Future;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

pub(crate) const GATEWAY_MESSAGE_QUEUE_CAPACITY: usize = 256;
pub(crate) const MAX_GATEWAY_ACCOUNTS_PER_CHANNEL: usize = 16;
pub(crate) const MAX_ACTIVE_GATEWAY_CHANNELS: usize = MAX_GATEWAY_ACCOUNTS_PER_CHANNEL * 3;
pub(crate) const MAX_ACTIVE_GATEWAY_MESSAGES: usize = 64;
const GATEWAY_CONSUMERS: usize = 1;

type GatewayConsumerWork = ServiceWorkSupervisor<GATEWAY_CONSUMERS>;
type GatewayChannelWork = ServiceWorkSupervisor<MAX_ACTIVE_GATEWAY_CHANNELS>;
type GatewayMessageWork = ServiceWorkSupervisor<MAX_ACTIVE_GATEWAY_MESSAGES>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GatewayMessageAdmissionError {
    ShuttingDown,
    Busy,
}

impl GatewayMessageAdmissionError {
    pub(crate) fn public_code(self) -> &'static str {
        match self {
            Self::ShuttingDown => "gateway-shutting-down",
            Self::Busy => "gateway-busy",
        }
    }

    pub(crate) fn audit_code(self) -> &'static str {
        match self {
            Self::ShuttingDown => "gateway_shutting_down",
            Self::Busy => "gateway_busy",
        }
    }
}

#[derive(Clone)]
pub(crate) struct GatewayWorkServices {
    cancel: CancellationToken,
    consumer: GatewayConsumerWork,
    channels: GatewayChannelWork,
    messages: GatewayMessageWork,
}

impl GatewayWorkServices {
    pub(crate) fn new(app: AppWorkSupervisor) -> Self {
        Self {
            cancel: CancellationToken::new(),
            consumer: GatewayConsumerWork::new(app.clone()),
            channels: GatewayChannelWork::new(app.clone()),
            messages: GatewayMessageWork::new(app),
        }
    }

    pub(crate) fn cancellation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    pub(crate) fn spawn_consumer<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.consumer.spawn(work)
    }

    pub(crate) fn spawn_channel<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.channels.spawn(work)
    }

    pub(crate) fn try_admit_message(
        &self,
    ) -> Result<ServiceWorkAdmission<MAX_ACTIVE_GATEWAY_MESSAGES>, GatewayMessageAdmissionError>
    {
        self.messages.try_admit().map_err(map_message_error)
    }

    pub(crate) fn spawn_message<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), GatewayMessageAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future + Send + 'static,
    {
        self.try_admit_message()?
            .spawn(work)
            .map_err(map_message_error)
    }

    pub(crate) fn message_diagnostics(&self) -> ServiceWorkDiagnostics {
        self.messages.diagnostics()
    }

    #[cfg(test)]
    pub(crate) fn consumer_phase(&self) -> crate::services::work_registry::ServiceWorkPhase {
        self.consumer.phase()
    }

    #[cfg(test)]
    pub(crate) fn channel_phase(&self) -> crate::services::work_registry::ServiceWorkPhase {
        self.channels.phase()
    }

    #[cfg(test)]
    pub(crate) fn message_phase(&self) -> crate::services::work_registry::ServiceWorkPhase {
        self.messages.phase()
    }

    pub(crate) async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.begin_closing();
        let (consumer, channels, messages) = tokio::join!(
            self.consumer.stop_and_wait(deadline),
            self.channels.stop_and_wait(deadline),
            self.messages.stop_and_wait(deadline),
        );
        consumer && channels && messages
    }

    pub(crate) fn begin_closing(&self) {
        // Le jeton du run arrête aussi les opérations internes qui ne portent
        // pas directement le jeton de leur registre local.
        self.cancel.cancel();
        self.consumer.begin_closing();
        self.channels.begin_closing();
        self.messages.begin_closing();
    }
}

fn map_message_error(error: ServiceWorkAdmissionError) -> GatewayMessageAdmissionError {
    match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => {
            GatewayMessageAdmissionError::ShuttingDown
        }
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            GatewayMessageAdmissionError::Busy
        }
    }
}
