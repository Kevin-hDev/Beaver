#[test]
fn api_and_ollama_drain_before_each_request_and_again_before_no_tool_exit() {
    for request in [
        include_str!("../llm/agent_loop_request.rs"),
        include_str!("agent_loop_ollama_request.rs"),
    ] {
        assert!(request.contains(".prepare_for_model_request(params.messages)"));
    }
    let boundary = include_str!("subagent_orchestration.rs");
    let prepare = boundary
        .find("pub async fn prepare_for_model_request")
        .unwrap();
    let first_drain = boundary[prepare..]
        .find("subagent_instruction_delivery::drain")
        .map(|offset| prepare + offset)
        .expect("drain before model request");
    let no_tool = boundary
        .find("pub async fn continue_after_no_tool_turn")
        .unwrap();
    let second_drain = boundary[no_tool..]
        .find("subagent_instruction_delivery::drain")
        .map(|offset| no_tool + offset)
        .expect("drain before no-tool exit");
    let wait = boundary[no_tool..]
        .find("self.after_no_tool_turn(messages, cancel)")
        .map(|offset| no_tool + offset)
        .expect("wait after second drain");
    assert!(prepare < first_drain);
    assert!(first_drain < no_tool);
    assert!(second_drain < wait);
}

#[test]
fn correction_arriving_with_tool_calls_cannot_skip_tool_execution() {
    for (request_source, execution_source, runner, executor) in [
        (
            include_str!("../llm/agent_loop.rs"),
            include_str!("../llm/agent_loop.rs"),
            "agent_loop_tools::execute_tool_batch",
            "agent_loop_tools::execute_tool_batch",
        ),
        (
            include_str!("agent_loop.rs"),
            include_str!("agent_loop_tool_turn.rs"),
            "agent_loop_tool_turn::run",
            "agent_loop_tool_batch::execute",
        ),
    ] {
        let request = request_source
            .find("let request_output")
            .expect("model request");
        let no_tool = request_source
            .find("if result.tool_calls.is_empty()")
            .expect("no-tool branch");
        let before_no_tool = &request_source[request..no_tool];

        assert!(!before_no_tool.contains("subagent_instruction_delivery::drain"));
        assert!(request_source[no_tool..].contains(runner));
        assert!(execution_source.contains(executor));
    }
}
