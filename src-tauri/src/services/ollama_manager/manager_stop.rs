impl OllamaManager {
    async fn stop_impl(
        &self,
        operation_deadline: Instant,
        process_deadline: Instant,
    ) -> Result<(), OllamaErrorCode> {
        // Un arrêt tardif doit encore nettoyer un démon inactif. L'échéance courte
        // borne seulement l'attente d'une opération concurrente, pas le reap final.
        let _operation = match self.inner().operation_lock.try_lock() {
            Ok(operation) => operation,
            Err(_) => {
                let remaining = operation_deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(OllamaErrorCode::OllamaSetupTimeout);
                }
                tokio::time::timeout(remaining, self.inner().operation_lock.lock())
                    .await
                    .map_err(|_| OllamaErrorCode::OllamaSetupTimeout)?
            }
        };
        if matches!(self.status().await.daemon, DaemonState::External { .. }) {
            return Ok(());
        }
        let process = self
            .inner()
            .owned_process
            .lock()
            .map_err(|_| OllamaErrorCode::OllamaInternal)?
            .take();
        let Some(process) = process else {
            return Ok(());
        };
        if Instant::now() >= process_deadline {
            self.inner()
                .owned_process
                .lock()
                .map_err(|_| OllamaErrorCode::OllamaInternal)?
                .replace(process);
            return Err(OllamaErrorCode::OllamaSetupTimeout);
        }
        let result =
            tokio::task::spawn_blocking(move || stop_owned_process(process, process_deadline))
                .await
                .map_err(|_| OllamaErrorCode::OllamaStopFailed)?;
        match result {
            Ok(()) => {
                self.publish_daemon(DaemonState::Unavailable);
                Ok(())
            }
            Err(error) => {
                let (process, code) = *error;
                self.inner()
                    .owned_process
                    .lock()
                    .map_err(|_| OllamaErrorCode::OllamaInternal)?
                    .replace(process);
                Err(code)
            }
        }
    }
}

fn stop_owned_process(
    mut process: OwnedOllamaProcess,
    deadline: Instant,
) -> Result<(), Box<(OwnedOllamaProcess, OllamaErrorCode)>> {
    if let Err(error) = process.terminate() {
        return Err(Box::new((process, map_process_error(error))));
    }
    if let Err(error) = process.reap(deadline) {
        return Err(Box::new((process, map_process_error(error))));
    }
    Ok(())
}
