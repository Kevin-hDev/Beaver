use super::host_identity::HostIdentity;
use std::collections::{BTreeMap, BTreeSet};

impl super::runtime::ExtensionRuntime {
    pub(super) async fn configure_event_deliveries(
        &self,
        subscriptions: BTreeMap<HostIdentity, BTreeSet<String>>,
    ) -> Result<(), String> {
        for (identity, events) in subscriptions {
            if events.is_empty() {
                self.work.event_router().clear(&identity);
                continue;
            }
            let Some((generation, process, cancel)) = self.event_target(&identity).await else {
                continue;
            };
            if self
                .work
                .event_router()
                .configured(&identity, generation, &events)
            {
                continue;
            }
            let delivery = super::event_delivery::EventDelivery::start(process, cancel, &self.work)
                .map_err(|error| error.public_code().to_string())?;
            self.work
                .event_router()
                .install(identity, generation, events, delivery);
        }
        Ok(())
    }

    async fn event_target(
        &self,
        identity: &HostIdentity,
    ) -> Option<(
        u64,
        std::sync::Arc<super::host_process::HostProcess>,
        tokio_util::sync::CancellationToken,
    )> {
        let hosts = self.hosts.lock().await;
        let (_, generation, _) = hosts.usable_snapshot(identity)?;
        let (process, cancel) = hosts.event_target(identity, generation)?;
        Some((generation, process, cancel))
    }
}
