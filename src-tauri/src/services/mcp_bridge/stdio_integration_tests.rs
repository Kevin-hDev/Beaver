use super::process_manager;
use super::stdio::StdioTransport;
use super::transport::McpTransport;

#[tokio::test]
async fn stdio_transport_handshakes_and_calls_a_real_child_process() {
    let transport = StdioTransport::for_test_fixture();

    let tools = transport.list_tools().await.expect("tools/list response");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");

    let result = transport
        .call_tool("echo", serde_json::json!({ "value": "hello" }))
        .await
        .expect("tools/call response");
    assert_eq!(result.content, "hello");

    let pid = process_manager::process_id_for_test("__beaver_mcp_fixture")
        .expect("fixture process remains owned");
    process_manager::shutdown_one("__beaver_mcp_fixture").await;

    let mut processes = sysinfo::System::new();
    processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    assert!(processes.process(sysinfo::Pid::from_u32(pid)).is_none());
}

#[tokio::test]
async fn slow_ready_connector_uses_protocol_signal_instead_of_fixed_warmup() {
    // Le budget est derive du cout reel de lancement d'un enfant, mesure juste avant
    // par le meme chemin que la production. Ce cout passe de quelques dizaines de
    // millisecondes a plusieurs centaines quand la machine est saturee, ce qui rendait
    // un budget absolu instable en suite parallele. La marge reste sous les 500 ms de
    // l'ancien warmup fixe : sa reintroduction echoue toujours.
    let node = which::which("node").expect("runtime de test indisponible");
    let baseline = std::time::Instant::now();
    crate::services::background_command::new_tokio(&node)
        .args(["-e", "0"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .expect("lancement de reference");
    let child_startup = baseline.elapsed();

    let transport = StdioTransport::for_slow_test_fixture(650);
    let started = std::time::Instant::now();
    let tools = transport.list_tools().await.expect("slow fixture tools");
    let readiness = started.elapsed();

    assert_eq!(tools[0].name, "echo");
    assert!(readiness >= std::time::Duration::from_millis(600));
    assert!(
        readiness <= child_startup + std::time::Duration::from_millis(950),
        "disponibilite en {readiness:?} pour un lancement mesure a {child_startup:?}"
    );
    process_manager::shutdown_one("__beaver_mcp_slow_fixture").await;
}
