use super::HistoryEntry;
use std::path::{Path, PathBuf};

pub(crate) async fn append_with_atomic_writer<Writer, Future>(
    path: &Path,
    entry: HistoryEntry,
    writer: Writer,
) -> Result<(), String>
where
    Writer: FnOnce(PathBuf, Vec<u8>) -> Future,
    Future: std::future::Future<Output = Result<(), String>>,
{
    super::history_store::append_inner(path, entry, writer, || {}).await
}

pub(crate) async fn append_with_read_observer<Observer>(
    path: &Path,
    entry: HistoryEntry,
    observer: Observer,
) -> Result<(), String>
where
    Observer: FnMut(),
{
    super::history_store::append_inner(
        path,
        entry,
        |path, bytes| async move {
            crate::services::private_store::atomic_write_async(path, bytes).await
        },
        observer,
    )
    .await
}
