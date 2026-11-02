use rmcp::{
    RoleClient, ServiceExt,
    model::{ClientJsonRpcMessage, ServerJsonRpcMessage, ServerResult},
    transport::{IntoTransport, Transport},
};
use std::{path::Path, process::Command};
use tempfile::TempDir;
use vexen::{mcp::VexenServer, store::Store};

fn message(raw: &str) -> ClientJsonRpcMessage {
    serde_json::from_str(raw).unwrap()
}
async fn receive(client: &mut impl Transport<RoleClient>) -> ServerResult {
    match client.receive().await.unwrap() {
        ServerJsonRpcMessage::Response(r) => r.result,
        other => panic!("unexpected MCP response {other:?}"),
    }
}
async fn start() -> (impl Transport<RoleClient>, tokio::task::JoinHandle<()>) {
    let (server_io, client_io) = tokio::io::duplex(16 * 1024);
    let task = tokio::spawn(async move {
        let service = VexenServer::new().serve(server_io).await.unwrap();
        service.waiting().await.unwrap();
    });
    (
        IntoTransport::<RoleClient, _, _>::into_transport(client_io),
        task,
    )
}
fn repository() -> (TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let init = Command::new("git")
        .arg("init")
        .arg("-q")
        .arg(root)
        .status()
        .unwrap();
    assert!(init.success());
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["config", "user.email", "mcp@example.invalid"])
        .status()
        .unwrap();
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["config", "user.name", "MCP test"])
        .status()
        .unwrap();
    std::fs::write(root.join("main.rs"), "fn main() {}\n").unwrap();
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["add", "main.rs"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["commit", "-qm", "initial"])
            .status()
            .unwrap()
            .success()
    );
    Store::initialize(root).unwrap();
    let canonical = std::fs::canonicalize(root)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    (dir, canonical)
}
async fn call_check(client: &mut impl Transport<RoleClient>, root: &str, id: u64) -> ServerResult {
    call_tool(
        client,
        "vexen_check",
        serde_json::json!({"repo_root":root,"paths":["main.rs"]}),
        id,
    )
    .await
}
async fn call_tool(
    client: &mut impl Transport<RoleClient>,
    name: &str,
    args: serde_json::Value,
    id: u64,
) -> ServerResult {
    let request = serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":args,"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"vexen-test","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}});
    client
        .send(serde_json::from_value(request).unwrap())
        .await
        .unwrap();
    receive(client).await
}
async fn call_check_legacy(
    client: &mut impl Transport<RoleClient>,
    root: &str,
    id: u64,
) -> ServerResult {
    let request = serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"vexen_check","arguments":{"repo_root":root,"paths":["main.rs"]}}});
    client
        .send(serde_json::from_value(request).unwrap())
        .await
        .unwrap();
    receive(client).await
}

#[tokio::test]
async fn current_discover_lifecycle_lists_sorted_schemas_and_calls_explicit_root() {
    let (_repo, root) = repository();
    let (mut client, task) = start().await;
    client.send(message(r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"vexen-test","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#)).await.unwrap();
    assert!(matches!(
        receive(&mut client).await,
        ServerResult::DiscoverResult(_)
    ));
    client.send(message(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"vexen-test","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#)).await.unwrap();
    let ServerResult::ListToolsResult(list) = receive(&mut client).await else {
        panic!("expected tools/list")
    };
    let names: Vec<_> = list.tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(
        names,
        [
            "vexen_check",
            "vexen_explain",
            "vexen_outcome",
            "vexen_record"
        ]
    );
    assert!(list.tools.iter().all(|t| t.output_schema.is_some()));
    assert_eq!(
        list.tools[0].annotations.as_ref().unwrap().read_only_hint,
        Some(true)
    );
    let ServerResult::CallToolResult(result) = call_check(&mut client, &root, 4).await else {
        panic!("expected check response")
    };
    let structured = result.structured_content.unwrap();
    assert_eq!(structured["repo_id"].as_str().unwrap().len(), 64);
    assert_eq!(structured["status"], "no_relevant_evidence");
    let content = serde_json::to_string(&result.content).unwrap();
    assert!(content.len() <= 1800);
    assert_eq!(
        Store::open_read_only(Path::new(&root))
            .unwrap()
            .events()
            .unwrap()
            .len(),
        0
    );
    let repo = Path::new(&root);
    let bytes = std::fs::read(repo.join("main.rs")).unwrap();
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let commit = String::from_utf8(
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    let ServerResult::CallToolResult(recorded) = call_tool(
        &mut client,
        "vexen_record",
        serde_json::json!({"repo_root":root,"record":{"idempotency_key":"mcp-test-record","kind":"accepted_decision","summary":"Keep the existing parsing boundary","anchor_path":"main.rs","anchor_sha256":hash,"git_commit":commit,"conditions":[]}}),
        5,
    )
    .await
    else {
        panic!("expected record response")
    };
    let record_result = recorded.structured_content.unwrap();
    assert_eq!(record_result["status"], "accepted_decision");
    let event_id = record_result["items"][0]["event_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let ServerResult::CallToolResult(outcome) = call_tool(
        &mut client,
        "vexen_outcome",
        serde_json::json!({"repo_root":root,"event_id":event_id,"outcome":{"idempotency_key":"mcp-test-outcome","test_id":"unit:smoke","exit_code":0,"output_sha256":null}}),
        6,
    )
    .await
    else {
        panic!("expected outcome response")
    };
    assert_eq!(outcome.structured_content.unwrap()["status"], "observed");
    assert_eq!(
        Store::open_read_only(repo).unwrap().events().unwrap().len(),
        2
    );
    task.abort();
}

#[tokio::test]
async fn legacy_initialize_lifecycle_rejects_non_absolute_root_truthfully() {
    let (mut client, task) = start().await;
    client.send(message(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"legacy-test","version":"1"}}}"#)).await.unwrap();
    assert!(matches!(
        receive(&mut client).await,
        ServerResult::InitializeResult(_)
    ));
    client
        .send(message(
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        ))
        .await
        .unwrap();
    let ServerResult::CallToolResult(result) =
        call_check_legacy(&mut client, "relative/repo", 2).await
    else {
        panic!("expected check response")
    };
    assert_eq!(result.is_error, Some(true));
    task.abort();
}

/// On Windows `fs::canonicalize` returns a `\?\C:\...` path. Agents pass the
/// ordinary `C:\...` form, which must be accepted for the same directory.
#[cfg(windows)]
#[tokio::test]
async fn accepts_an_ordinary_windows_path_for_repo_root() {
    let (_repo, root) = repository();
    let plain = root
        .strip_prefix(r"\\?\")
        .expect("verbatim root")
        .to_owned();
    let (mut client, task) = start().await;
    client.send(message(r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"vexen-test","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#)).await.unwrap();
    let _ = receive(&mut client).await;
    let ServerResult::CallToolResult(result) = call_check(&mut client, &plain, 2).await else {
        panic!("expected check response")
    };
    assert_ne!(result.is_error, Some(true), "{:?}", result.content);
    assert_eq!(
        result.structured_content.unwrap()["status"],
        "no_relevant_evidence"
    );
    drop(client);
    let _ = task.await;
}

/// Claude Code warns on JSON Schema formats it does not know, such as the
/// `uint32` that schemars emits for unsigned integers. Keep to portable ones.
#[tokio::test]
async fn tool_schemas_use_only_portable_integer_formats() {
    let (_repo, _root) = repository();
    let (mut client, task) = start().await;
    client.send(message(r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"vexen-test","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#)).await.unwrap();
    let _ = receive(&mut client).await;
    client.send(message(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"vexen-test","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#)).await.unwrap();
    let ServerResult::ListToolsResult(list) = receive(&mut client).await else {
        panic!("expected tools/list")
    };
    let json = serde_json::to_string(&list.tools).unwrap();
    assert!(
        !json.contains("\"uint"),
        "unsigned formats in schemas: {json}"
    );
    drop(client);
    let _ = task.await;
}

/// A client may expose only `content` to its model, so the lesson must survive
/// the actual check -> response path, including eligibility and stale filtering.
#[tokio::test]
async fn text_only_check_clients_can_read_a_lesson_until_its_anchor_changes() {
    use rmcp::handler::server::wrapper::Parameters;
    use sha2::{Digest, Sha256};
    use vexen::{
        mcp::CheckInput,
        store::{EventKind, NewEvent, NewOutcome},
    };

    let (_repo, root) = repository();
    let path = Path::new(&root).join("main.rs");
    let hash = format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap()));
    let commit = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let store = Store::open(Path::new(&root)).unwrap();
    let id = store
        .append(NewEvent {
            idempotency_key: "text-only-lesson".into(),
            kind: EventKind::Attempt,
            summary: "Default rounding failed: 0.125 must round to 0.13 with ROUND_HALF_UP.".into(),
            anchor_path: "main.rs".into(),
            anchor_sha256: hash,
            git_commit: Some(String::from_utf8(commit.stdout).unwrap().trim().into()),
            conditions: vec![],
        })
        .unwrap();
    store
        .outcome(
            id,
            NewOutcome {
                idempotency_key: "text-only-outcome".into(),
                test_id: "rounding".into(),
                exit_code: 1,
                output_sha256: None,
            },
        )
        .unwrap();
    let server = VexenServer::new();
    let input = || {
        Parameters(CheckInput {
            repo_root: root.clone(),
            paths: Some(vec!["main.rs".into()]),
            diff_ref: None,
            query: None,
            conditions: None,
        })
    };
    let result = server.check(input()).await;
    assert_ne!(result.is_error, Some(true));
    let text = serde_json::to_string(&result.content).unwrap();
    assert!(
        text.contains("ROUND_HALF_UP"),
        "lesson missing from text: {text}"
    );
    assert!(text.contains("not a guarantee"));
    assert_eq!(
        result.structured_content.unwrap()["status"],
        "observed_failure"
    );

    std::fs::write(&path, "fn main() { /* changed */ }\n").unwrap();
    let stale = server.check(input()).await;
    assert_eq!(
        stale.structured_content.unwrap()["status"],
        "no_relevant_evidence"
    );
    assert!(
        !serde_json::to_string(&stale.content)
            .unwrap()
            .contains("ROUND_HALF_UP")
    );
}
