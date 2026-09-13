use std::sync::OnceLock;

pub(crate) const WORKER_STACK_BYTES: usize = 8 * 1024 * 1024;

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub(crate) fn configure() -> Result<(), ()> {
    if RUNTIME.get().is_some() {
        return Ok(());
    }
    let runtime = build()?;
    let handle = runtime.handle().clone();
    RUNTIME.set(runtime).map_err(|_| ())?;
    tauri::async_runtime::set(handle);
    Ok(())
}

fn build() -> Result<tokio::runtime::Runtime, ()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("beaver-async")
        .thread_stack_size(WORKER_STACK_BYTES)
        .build()
        .map_err(|_| ())
}

#[cfg(test)]
#[path = "runtime_async_tests.rs"]
mod tests;
