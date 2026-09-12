static READY: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
const INITIALIZATION_ERROR: &str = "Contrôle de commande indisponible.";

pub(crate) async fn initialize() -> Result<(), String> {
    initialize_with(&READY, |task| {
        std::thread::Builder::new()
            .name("beaver-command-check".to_string())
            .stack_size(8 * 1024 * 1024)
            .spawn(task)
            .map(|_| ())
    })
    .await
}

async fn initialize_with<F>(ready: &tokio::sync::OnceCell<()>, spawn: F) -> Result<(), String>
where
    F: FnOnce(Box<dyn FnOnce() + Send>) -> std::io::Result<()>,
{
    ready
        .get_or_try_init(|| async move {
            let (send, receive) = tokio::sync::oneshot::channel();
            spawn(Box::new(move || {
                super::permission_bash::initialize_safe_patterns();
                super::security::initialize_destructive_patterns();
                let _ = send.send(());
            }))
            .map_err(|_| INITIALIZATION_ERROR.to_string())?;
            receive
                .await
                .map_err(|_| INITIALIZATION_ERROR.to_string())
        })
        .await
        .map(|_| ())
}

#[cfg(test)]
pub(super) async fn initialize_for_test<F>(
    ready: &tokio::sync::OnceCell<()>,
    spawn: F,
) -> Result<(), String>
where
    F: FnOnce(Box<dyn FnOnce() + Send>) -> std::io::Result<()>,
{
    initialize_with(ready, spawn).await
}
