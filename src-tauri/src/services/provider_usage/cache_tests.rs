use super::*;

#[tokio::test]
async fn cache_is_bounded_to_connection_limit() {
    clear().await;
    for index in 0..(CONNECTION_LIMIT + 4) {
        let connection_id = format!("provider-{index}");
        put(
            &connection_id,
            0,
            RemoteData {
                fetched_at: index as i64,
                ..Default::default()
            },
        )
        .await;
    }
    assert_eq!(len().await, CONNECTION_LIMIT);
}

#[tokio::test]
async fn removes_only_the_selected_connection() {
    clear().await;
    let xai_generation = super::super::credential_epoch::current("xai").unwrap();
    let moonshot_generation = super::super::credential_epoch::current("moonshot").unwrap();
    put("xai", xai_generation, RemoteData::default()).await;
    put("moonshot", moonshot_generation, RemoteData::default()).await;
    remove("xai").await;
    assert!(get("xai").await.is_none());
    assert!(get("moonshot").await.is_some());
}

#[tokio::test]
async fn old_generation_is_hidden_immediately() {
    clear().await;
    let generation = super::super::credential_epoch::current("deepseek").unwrap();
    put("deepseek", generation, RemoteData::default()).await;
    super::super::credential_epoch::invalidate("deepseek").unwrap();
    assert!(get("deepseek").await.is_none());
}
