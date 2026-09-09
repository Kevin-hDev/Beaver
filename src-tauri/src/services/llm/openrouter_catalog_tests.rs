use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn catalog_body(id: &str) -> serde_json::Value {
    serde_json::json!({"data":[{
        "id":id,
        "architecture":{"output_modalities":["text"]},
        "supported_parameters":["tools"]
    }]})
}

async fn concurrent_refresh_case(list_first: bool, list_second: bool, status: u16) {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(
            ResponseTemplate::new(status)
                .set_delay(std::time::Duration::from_millis(30))
                .set_body_json(catalog_body("vendor/concurrent")),
        )
        .expect(1)
        .mount(&server)
        .await;
    let url = format!("{}/models", server.uri());
    let request = |listing| {
        let url = &url;
        async move {
            if listing {
                super::openrouter_catalog::list_models_from_url_for_test(url)
                    .await
                    .map(|_| ())
            } else {
                super::openrouter_catalog::ensure_model_from_url_for_test("vendor/concurrent", url)
                    .await
                    .map(|_| ())
            }
        }
    };
    let (first, second) = tokio::join!(request(list_first), request(list_second));
    assert_eq!(first.is_ok(), status == 200);
    assert_eq!(second.is_ok(), status == 200);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn root_catalog_regression_concurrent_lists_share_one_download() {
    concurrent_refresh_case(true, true, 200).await;
}

#[tokio::test]
async fn root_catalog_regression_admission_then_list_share_one_download() {
    concurrent_refresh_case(false, true, 200).await;
}

#[tokio::test]
async fn root_catalog_regression_failed_concurrent_refresh_is_not_repeated() {
    concurrent_refresh_case(false, false, 503).await;
}

#[tokio::test]
async fn root_catalog_regression_public_failure_does_not_accuse_the_key() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;
    let error = super::openrouter_catalog::ensure_model_from_url_for_test(
        "vendor/unavailable",
        &server.uri(),
    )
    .await
    .unwrap_err();
    assert!(matches!(
        error,
        super::types::LlmError::KnownProvider(
            super::provider_error::ProviderErrorCode::ModelCatalogUnavailable
        )
    ));
}

#[tokio::test]
async fn failed_shared_refresh_does_not_poison_the_next_attempt() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(&server)
        .await;
    assert!(super::openrouter_catalog::ensure_model_from_url_for_test(
        "vendor/recovered",
        &server.uri(),
    )
    .await
    .is_err());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    server.reset().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(catalog_body("vendor/recovered")))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(
        super::openrouter_catalog::ensure_model_from_url_for_test(
            "vendor/recovered",
            &server.uri(),
        )
        .await
        .unwrap()
        .id,
        "vendor/recovered"
    );
}

#[tokio::test]
async fn public_catalog_dispatch_reuses_the_inflight_public_list() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(30))
                .set_body_json(catalog_body("vendor/dispatch")),
        )
        .expect(1)
        .mount(&server)
        .await;
    let url = server.uri();
    let (first, dispatched) = tokio::join!(
        super::openrouter_catalog::list_models_from_url_for_test(&url),
        super::model_catalog::list_models_for("openrouter"),
    );
    assert_eq!(first.unwrap()[0].id, "vendor/dispatch");
    assert_eq!(dispatched.unwrap()[0].id, "vendor/dispatch");
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn public_catalog_get_has_no_secret_and_atomically_replaces_stale_models() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(catalog_body("vendor/fresh")))
        .expect(1)
        .mount(&server)
        .await;
    super::runtime_models::replace_provider(
        "openrouter",
        &[super::openai_compat_parsing::parse_models_list(
            &catalog_body("vendor/stale"),
            "openrouter",
        )
        .unwrap()
        .remove(0)],
    )
    .unwrap();

    let models = super::openrouter_catalog::list_models_from_url_for_test(&format!(
        "{}/models",
        server.uri()
    ))
    .await
    .unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "vendor/fresh");
    assert!(super::runtime_models::lookup("openrouter", "vendor/fresh").is_some());
    assert!(super::runtime_models::lookup("openrouter", "vendor/stale").is_none());
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert!(!requests[0].headers.contains_key("authorization"));
}

#[tokio::test]
async fn failed_public_refresh_keeps_the_previous_valid_registry() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("not-json", "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let previous = super::openai_compat_parsing::parse_models_list(
        &catalog_body("vendor/stable"),
        "openrouter",
    )
    .unwrap();
    super::runtime_models::replace_provider("openrouter", &previous).unwrap();

    let result = super::openrouter_catalog::list_models_from_url_for_test(&format!(
        "{}/models",
        server.uri()
    ))
    .await;

    assert!(matches!(
        result,
        Err(super::types::LlmError::KnownProvider(
            super::provider_error::ProviderErrorCode::ModelCatalogUnavailable
        ))
    ));
    assert!(super::runtime_models::lookup("openrouter", "vendor/stable").is_some());
}

#[tokio::test]
async fn concurrent_cold_lookups_share_one_public_initialization() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(30))
                .set_body_json(catalog_body("vendor/shared")),
        )
        .expect(1)
        .mount(&server)
        .await;
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
    let url = format!("{}/models", server.uri());

    let (first, second) = tokio::join!(
        super::openrouter_catalog::ensure_model_from_url_for_test("vendor/shared", &url),
        super::openrouter_catalog::ensure_model_from_url_for_test("vendor/shared", &url),
    );

    assert_eq!(first.unwrap().id, "vendor/shared");
    assert_eq!(second.unwrap().id, "vendor/shared");
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn ui_list_and_cold_admission_share_the_same_initialization() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(30))
                .set_body_json(catalog_body("vendor/shared-ui")),
        )
        .expect(1)
        .mount(&server)
        .await;
    super::runtime_models::replace_provider("openrouter", &[]).unwrap();
    let url = format!("{}/models", server.uri());

    let listing = tokio::spawn({
        let url = url.clone();
        async move { super::openrouter_catalog::list_models_from_url_for_test(&url).await }
    });
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    let admitted =
        super::openrouter_catalog::ensure_model_from_url_for_test("vendor/shared-ui", &url).await;

    assert_eq!(listing.await.unwrap().unwrap()[0].id, "vendor/shared-ui");
    assert_eq!(admitted.unwrap().id, "vendor/shared-ui");
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn an_evicted_openrouter_catalog_is_reloaded_once_for_admission() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let model = super::openai_compat_parsing::parse_models_list(
        &catalog_body("vendor/evicted"),
        "openrouter",
    )
    .unwrap();
    super::runtime_models::replace_provider("openrouter", &model).unwrap();
    for suffix in 'a'..='p' {
        super::runtime_models::replace_provider(&format!("fixture-{suffix}"), &[]).unwrap();
    }
    assert!(super::runtime_models::lookup("openrouter", "vendor/evicted").is_none());
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(catalog_body("vendor/evicted")))
        .expect(1)
        .mount(&server)
        .await;

    let admitted = super::openrouter_catalog::ensure_model_from_url_for_test(
        "vendor/evicted",
        &format!("{}/models", server.uri()),
    )
    .await
    .unwrap();

    assert_eq!(admitted.id, "vendor/evicted");
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}
