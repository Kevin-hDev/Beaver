use std::future::Future;

pub(crate) async fn combine_with_work(
    processes_stopped: bool,
    work: impl Future<Output = bool>,
) -> bool {
    // Le registre est toujours attendu : un échec processus ne doit jamais
    // court-circuiter l'annulation et la récolte du travail du domaine.
    let work_stopped = work.await;
    processes_stopped && work_stopped
}
