use super::{error_codes, types::HOST_LOAD_STAGES};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct HostLoadTracker {
    active: Mutex<Option<ActiveLoad>>,
}

struct ActiveLoad {
    extension_id: String,
    next_stage: usize,
}

impl HostLoadTracker {
    pub async fn arm(&self, extension_id: &str) -> Result<(), String> {
        super::validation::identifier(extension_id)?;
        let mut active = self.active.lock().await;
        if active.is_some() {
            return Err(error_codes::HOST_BUSY.to_string());
        }
        *active = Some(ActiveLoad {
            extension_id: extension_id.to_string(),
            next_stage: 0,
        });
        Ok(())
    }

    pub async fn advance(&self, stage: &str) -> Result<String, String> {
        let mut active = self.active.lock().await;
        let load = active
            .as_mut()
            .ok_or_else(|| error_codes::REQUEST_INVALID.to_string())?;
        if HOST_LOAD_STAGES.get(load.next_stage).copied() != Some(stage) {
            return Err(error_codes::REQUEST_INVALID.to_string());
        }
        load.next_stage += 1;
        Ok(load.extension_id.clone())
    }

    pub async fn clear(&self) {
        self.active.lock().await.take();
    }
}
