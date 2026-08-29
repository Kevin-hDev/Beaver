use super::endpoint_policy::resolve;
use super::route_profile::EndpointPolicy;

const REGIONS: &[(&str, &str)] = &[
    ("us", "https://us.example.com/v1"),
    ("eu", "https://eu.example.com/v1"),
];

#[tokio::test]
async fn endpoint_policy_validates_region_workspace_and_pinned_backend() {
    assert_eq!(
        resolve(
            EndpointPolicy::RegionAllowlist { regions: REGIONS },
            Some("eu")
        )
        .await
        .unwrap(),
        "https://eu.example.com/v1"
    );
    assert!(resolve(
        EndpointPolicy::RegionAllowlist { regions: REGIONS },
        Some("xx")
    )
    .await
    .is_err());

    let workspace = resolve(
        EndpointPolicy::Workspace {
            host_suffix: "api.example.com",
        },
        Some("team-42"),
    )
    .await
    .unwrap();
    assert_eq!(workspace, "https://team-42.api.example.com");
    for invalid in ["", "../team", "team_name", &"x".repeat(65)] {
        assert!(resolve(
            EndpointPolicy::Workspace {
                host_suffix: "api.example.com",
            },
            Some(invalid),
        )
        .await
        .is_err());
    }

    let pinned = EndpointPolicy::PinnedBackend {
        base_url: "https://oauth.example.com/v1",
    };
    assert!(resolve(pinned, Some("https://attacker.example/v1"))
        .await
        .is_err());
}

#[tokio::test]
async fn endpoint_policy_rejects_unsafe_remote_urls_before_use() {
    for invalid in [
        "http://example.com/v1",
        "file:///tmp/provider",
        "https://user:pass@example.com/v1",
        "https://example.com/v1?token=x",
        "https://example.com/v1#fragment",
        "https://127.0.0.1/v1",
        "https://169.254.169.254/latest/meta-data",
        "https://metadata.google.internal/computeMetadata/v1",
    ] {
        assert!(
            resolve(EndpointPolicy::ValidatedHttps, Some(invalid))
                .await
                .is_err(),
            "{invalid}"
        );
    }
}
