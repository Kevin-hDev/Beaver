use std::time::Instant;

pub(super) async fn run_until<Operation>(deadline: Instant, operation: Operation) -> bool
where
    Operation: FnOnce() + Send + 'static,
{
    let (operation_sender, operation_receiver) = std::sync::mpsc::sync_channel::<Operation>(1);
    let (completed, observed) = tokio::sync::oneshot::channel();
    let worker = std::thread::Builder::new()
        .name("beaver-terminal-close".to_string())
        .spawn(move || {
            if let Ok(operation) = operation_receiver.recv() {
                operation();
                let _ = completed.send(());
            }
        });
    if worker.is_err() {
        ::log::error!("[terminal] close worker unavailable");
        // Le destructeur est précisément l'opération qui peut bloquer :
        // l'ultime sortie du processus récupérera cette ressource conservée.
        std::mem::forget(operation);
        return false;
    }
    if let Err(error) = operation_sender.send(operation) {
        ::log::error!("[terminal] close worker refused work");
        std::mem::forget(error.0);
        return false;
    }
    // Les sessions gardent leurs admissions jusqu'à la fin de ce fil. S'il
    // bloque, le registre global reste non vide et le garde ultime fait foi.
    tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), observed)
        .await
        .is_ok_and(|completion| completion.is_ok())
}
