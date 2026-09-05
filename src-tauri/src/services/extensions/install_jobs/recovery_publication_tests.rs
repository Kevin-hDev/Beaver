use super::*;

#[tokio::test]
async fn registry_publication_wins_a_crash_before_job_completion() {
    let (root, original, id, source) = interrupted(true).await;
    let mut record = original.lock().unwrap().jobs[0]
        .checkpoint
        .as_ref()
        .unwrap()
        .record
        .clone()
        .unwrap();
    // Distinct identity isolates this publication from other recovery fixtures.
    record.manifest.id = format!("test-{}", uuid::Uuid::new_v4().simple());
    let manifest_path = source.join("beaver-extension.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["id"] = serde_json::Value::String(record.manifest.id.clone());
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    record.fingerprint = Some(super::super::super::fingerprint::calculate(&record).unwrap());
    {
        let mut state = original.lock().unwrap();
        state.jobs[0].checkpoint.as_mut().unwrap().record = Some(record.clone());
        original.persist(&state).unwrap();
    }
    super::super::super::registry_managed::add(record.clone()).unwrap();
    let recovered = store(
        &root.path().join("jobs.json"),
        Arc::new(AtomicUsize::new(0)),
    );
    assert_eq!(
        recovered.snapshot().unwrap().jobs[0].status,
        InstallStatus::Completed
    );
    recovered.dismiss_reconciled(&id).await.unwrap();
    assert!(source.join("index.mjs").exists());
    assert!(super::super::super::registry::find(&record.manifest.id).is_ok());
    super::super::super::registry::remove(&record.manifest.id).unwrap();
}

#[tokio::test]
async fn interruption_on_either_side_of_managed_rename_cleans_only_owned_paths() {
    for renamed in [false, true] {
        let (root, original, id, user_source) = interrupted(true).await;
        let mut state = original.lock().unwrap();
        let checkpoint = state.jobs[0].checkpoint.as_mut().unwrap();
        let staging = super::super::super::managed_store::prepare_owned(&checkpoint.token).unwrap();
        for name in ["beaver-extension.json", "index.mjs"] {
            std::fs::copy(user_source.join(name), staging.path().join(name)).unwrap();
        }
        let mut record =
            super::super::super::manifest::load_local(staging.path().to_str().unwrap())
                .unwrap()
                .record;
        record.origin = Some(super::super::super::types::ExtensionOrigin {
            kind: super::super::super::types::ExtensionOriginKind::Git,
            locator: "https://example.invalid/test.git".into(),
            revision: Some("ab".repeat(20)),
        });
        let destination = super::super::super::managed_store::root()
            .join(&record.manifest.id)
            .join(&checkpoint.token);
        super::super::super::managed_store::rewrite_source(
            &mut record,
            staging.path(),
            &destination,
        )
        .unwrap();
        checkpoint.record = Some(record.clone());
        checkpoint.safe_phase = None;
        original.persist(&state).unwrap();
        drop(state);
        if renamed {
            staging.commit(&record.manifest.id).unwrap();
        } else {
            drop(staging);
        }
        let recovered = store(
            &root.path().join("jobs.json"),
            Arc::new(AtomicUsize::new(0)),
        );
        assert!(!recovered.snapshot().unwrap().jobs[0].can_resume);
        recovered.dismiss_reconciled(&id).await.unwrap();
        assert!(!destination.exists());
        assert!(user_source.join("index.mjs").exists());
    }
}

#[tokio::test]
async fn historical_terminal_results_never_offer_resume() {
    for status in [
        InstallStatus::Completed,
        InstallStatus::Cancelled,
        InstallStatus::Failed,
    ] {
        let (root, original, _id, _source) = interrupted(true).await;
        {
            let mut state = original.lock().unwrap();
            state.jobs[0].view.status = status;
            original.persist(&state).unwrap();
        }
        let restored = store(
            &root.path().join("jobs.json"),
            Arc::new(AtomicUsize::new(0)),
        );
        let view = restored.snapshot().unwrap().jobs.remove(0);
        assert_eq!(view.status, status);
        assert!(!view.can_resume);
    }
}
