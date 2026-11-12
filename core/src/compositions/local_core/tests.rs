//! Wire tests: the checks the routes answer on, and the replay of the reply
//! fixtures kept in `testdata/`.

use std::time::{Duration, Instant};

use super::connect::{Code, END_STREAM_FLAG};
use super::lifecycle;
use super::server;
use super::test_support::{Harness, OpenStream, TEST_TOKEN, mask};
use crate::local_v1;

const PROJECT_SERVICE: &str = "/polterware.local.v1.ProjectService";
const MAINTENANCE_SERVICE: &str = "/polterware.local.v1.MaintenanceService";
const HEALTH_SERVICE: &str = "/polterware.local.v1.HealthService";
const WORK_SERVICE: &str = "/polterware.local.v1.WorkService";
const WATCH_PATH: &str = "/polterware.local.v1.ChangeService/WatchChanges";

#[test]
fn mint_token_is_random_hex() {
    let first = server::mint_token();
    let second = server::mint_token();
    assert_eq!(first.len(), 64);
    assert_ne!(first, second);
    assert!(
        first
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "token is not lowercase hex: {first}"
    );
}

#[test]
fn listen_binds_an_ephemeral_loopback_port() {
    let (listener, url) = server::listen("127.0.0.1:0").expect("bind");
    assert!(
        url.starts_with("http://127.0.0.1:") && !url.ends_with(":0"),
        "url = {url}"
    );
    drop(listener);
}

#[test]
fn health_check_needs_no_token() {
    let harness = Harness::start();
    harness.core.set_build("build-under-test");
    let reply = harness.unary(&format!("{HEALTH_SERVICE}/Check"), None, "{}");
    assert_eq!(reply.status, 200);
    let body = reply.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["build"], "build-under-test");
    assert_eq!(body["version"], server::VERSION);
    assert!(body["pid"].as_i64().unwrap_or_default() > 0);
    assert!(body["uptimeSec"].is_number());
}

#[test]
fn services_reject_missing_or_wrong_token() {
    let harness = Harness::start();
    for (name, token) in [
        ("no token", None),
        ("wrong token", Some("nope")),
        ("prefix only", Some(&TEST_TOKEN[..5])),
    ] {
        let reply = harness.unary(&format!("{PROJECT_SERVICE}/ListProjects"), token, "{}");
        assert_eq!(reply.status, 401, "{name}");
        assert_eq!(reply.error_code(), "unauthenticated", "{name}");
        assert_eq!(
            reply.text(),
            r#"{"code":"unauthenticated","message":"invalid or missing session token"}"#,
            "{name}"
        );
        assert_eq!(reply.content_type, "application/json", "{name}");
    }
}

#[test]
fn services_accept_the_session_token() {
    let harness = Harness::start();
    let reply = harness.unary(
        &format!("{PROJECT_SERVICE}/ListProjects"),
        Some(TEST_TOKEN),
        "{}",
    );
    assert_eq!(reply.status, 200);
    let ids: Vec<String> = reply.json()["projects"]
        .as_array()
        .expect("a projects list")
        .iter()
        .map(|project| project["id"].as_str().unwrap_or_default().to_string())
        .collect();
    assert!(
        ids.iter()
            .any(|id| id == crate::providers::seed::PROJECT_ID),
        "a fresh core must seed defaults, got {ids:?}"
    );
}

#[test]
fn mcp_endpoint_requires_the_token() {
    let harness = Harness::start();
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"cursor","version":"1"}}}"#;
    let unauthorized = harness.request(
        "POST",
        "/mcp",
        None,
        Some("application/json"),
        body.as_bytes(),
    );
    assert_eq!(unauthorized.status, 401);
    assert_eq!(
        unauthorized.text(),
        "invalid or missing session token\n",
        "Go's http.Error writes a plain-text line"
    );
    let authorized = harness.request(
        "POST",
        "/mcp",
        Some(TEST_TOKEN),
        Some("application/json"),
        body.as_bytes(),
    );
    assert_eq!(authorized.status, 200, "{}", authorized.text());
    assert!(!authorized.session_id.is_empty(), "no Mcp-Session-Id");
}

#[test]
fn shutdown_stops_the_core() {
    let harness = Harness::start();
    let reply = harness.unary(
        &format!("{MAINTENANCE_SERVICE}/Shutdown"),
        Some(TEST_TOKEN),
        "{}",
    );
    assert_eq!(reply.status, 200);
    assert!(harness.wait_stopped(Duration::from_secs(3)));
    harness.core.stop(); // stopping twice must not panic
}

#[test]
fn shutdown_requires_the_token() {
    let harness = Harness::start();
    let reply = harness.unary(&format!("{MAINTENANCE_SERVICE}/Shutdown"), None, "{}");
    assert_eq!(reply.status, 401);
    assert!(!harness.wait_stopped(Duration::from_millis(300)));
}

#[test]
fn idle_tracks_activity() {
    let harness = Harness::start();
    assert!(!harness.core.idle(Duration::from_secs(3600)));
    std::thread::sleep(Duration::from_millis(30));
    assert!(
        harness.core.idle(Duration::from_millis(10)),
        "a core with no activity must become idle"
    );
    let reply = harness.unary(
        &format!("{PROJECT_SERVICE}/ListProjects"),
        Some(TEST_TOKEN),
        "{}",
    );
    assert_eq!(reply.status, 200);
    assert!(
        !harness.core.idle(Duration::from_secs(1)),
        "an authenticated call must reset the idle clock"
    );
}

#[test]
fn unauthenticated_calls_do_not_keep_the_core_alive() {
    let harness = Harness::start();
    std::thread::sleep(Duration::from_millis(30));
    let _ = harness.unary(&format!("{PROJECT_SERVICE}/ListProjects"), None, "{}");
    assert!(
        harness.core.idle(Duration::from_millis(10)),
        "a rejected call must not count as activity"
    );
}

#[test]
fn store_health_is_reported_over_rpc() {
    let harness = Harness::start();
    let reply = harness.unary(
        &format!("{MAINTENANCE_SERVICE}/GetStoreHealth"),
        Some(TEST_TOKEN),
        "{}",
    );
    assert_eq!(reply.status, 200);
    assert_eq!(
        reply.text(),
        "{}",
        "a healthy store answers the zero message"
    );
}

#[test]
fn a_route_answers_each_connect_check() {
    let harness = Harness::start();
    let list = format!("{PROJECT_SERVICE}/ListProjects");

    let wrong_type = harness.request("POST", &list, Some(TEST_TOKEN), Some("text/plain"), b"{}");
    assert_eq!(wrong_type.status, 415);
    assert_eq!(
        wrong_type.accept_post, "application/json",
        "the port lists the codec it speaks; the Go core also lists protobuf and gRPC"
    );

    let wrong_method = harness.request("GET", &list, Some(TEST_TOKEN), None, b"");
    assert_eq!(wrong_method.status, 405);

    // Every route is ported now, so a call without a project reaches the
    // handler and fails its validation instead of answering `unimplemented`.
    let missing_project = harness.unary(
        &format!("{WORK_SERVICE}/ListWorkCards"),
        Some(TEST_TOKEN),
        "{}",
    );
    assert_eq!(missing_project.status, 400);
    assert!(
        missing_project.text().contains("project id is required"),
        "{}",
        missing_project.text()
    );
}

#[test]
fn a_health_check_reports_the_code_names_and_statuses() {
    assert_eq!(Code::Unimplemented.http_status(), 501);
    let harness = Harness::start();
    let reply = harness.unary(
        &format!("{PROJECT_SERVICE}/SaveProject"),
        Some(TEST_TOKEN),
        "{}",
    );
    assert_eq!(reply.status, 400);
    assert_eq!(
        reply.text(),
        r#"{"code":"invalid_argument","message":"project is required"}"#
    );
    let reply = harness.unary(
        &format!("{PROJECT_SERVICE}/ResolveProject"),
        Some(TEST_TOKEN),
        r#"{"id":"nope"}"#,
    );
    assert_eq!(reply.status, 404);
    assert_eq!(reply.error_code(), "not_found");
}

/// The messages a watcher resuming after `after_seq` receives, up to `count`,
/// before the test's client leaves.
///
/// A watch stays open for the changes that follow, as the Go core keeps it
/// open, so a test reads what it needs from the connection and drops it.
fn watch(harness: &Harness, after_seq: &str, count: usize) -> Vec<serde_json::Value> {
    let body = super::connect::envelope(0, format!("{{\"afterSeq\":\"{after_seq}\"}}").as_bytes());
    let mut stream = harness.open_stream(WATCH_PATH, Some(TEST_TOKEN), &body);
    let mut events = Vec::new();
    while events.len() < count {
        let Some((flags, payload)) = stream.next_frame(Duration::from_secs(5)) else {
            break;
        };
        assert_ne!(flags, END_STREAM_FLAG, "the stream ended early");
        events.push(serde_json::from_slice(&payload).unwrap_or(serde_json::Value::Null));
    }
    events
}

#[test]
fn a_stream_answers_the_sequence_it_starts_at() {
    let harness = Harness::start();
    let body = super::connect::envelope(0, br#"{"afterSeq":"0"}"#);
    let mut stream = harness.open_stream(WATCH_PATH, Some(TEST_TOKEN), &body);
    assert_eq!(stream.status(), 200);
    assert_eq!(stream.content_type(), "application/connect+json");
    let (flags, payload) = stream
        .next_frame(Duration::from_secs(5))
        .expect("a stream must answer its first message while it is open");
    assert_eq!(flags, 0, "the first frame carries a message, not the end");
    let first: serde_json::Value = serde_json::from_slice(&payload).expect("JSON");
    assert!(first["at"].is_string(), "{first}");

    // The stream stays open for the changes after it, so the core stopping is
    // what ends it, with a clean end-of-stream message.
    harness.core.stop();
    let mut last = None;
    while let Some(frame) = stream.next_frame(Duration::from_secs(5)) {
        last = Some(frame);
    }
    let (flags, payload) = last.expect("an end-of-stream message");
    assert_eq!(flags, END_STREAM_FLAG);
    assert_eq!(payload, b"{}", "a clean end carries the empty message");
}

#[test]
fn a_stream_without_the_token_answers_its_error_as_the_end_stream() {
    let harness = Harness::start();
    let body = super::connect::envelope(0, b"{}");
    let reply = harness.request(
        "POST",
        WATCH_PATH,
        None,
        Some("application/connect+json"),
        &body,
    );
    assert_eq!(
        reply.status, 200,
        "a stream reports its error in the stream"
    );
    assert_eq!(reply.content_type, "application/connect+json");
    assert_eq!(reply.accept_encoding, "gzip");
    let frames = reply.frames();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].0, END_STREAM_FLAG);
    assert_eq!(
        String::from_utf8_lossy(&frames[0].1),
        r#"{"error":{"code":"unauthenticated","message":"invalid or missing session token"}}"#
    );
}

#[test]
fn a_change_reaches_a_watcher() {
    let harness = Harness::start();
    let save = |id: &str| {
        let reply = harness.unary(
            &format!("{PROJECT_SERVICE}/SaveProject"),
            Some(TEST_TOKEN),
            &format!(r#"{{"project":{{"id":"{id}","name":"Watch"}}}}"#),
        );
        assert_eq!(reply.status, 200, "{}", reply.text());
        harness.core.bus.flush();
    };
    // One write, so the watcher has a sequence behind it; then the one it must
    // hear about.
    save("proj-watch-a");
    let start = harness.core.bus.subscribe(0).current;
    assert!(start > 0);

    save("proj-watch-b");

    // The stream replays the change that came after the watcher's sequence.
    let events = watch(&harness, &start.to_string(), 2);
    assert_eq!(
        events.len(),
        2,
        "the sequence the stream starts at, then the change: {events:?}"
    );
    let expected_seq = (start + 1).to_string();
    assert_eq!(
        events[0]["seq"].as_str(),
        Some(expected_seq.as_str()),
        "protojson writes int64 as a string: {events:?}"
    );
    assert!(
        events.iter().any(|event| event["kinds"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "projects"))),
        "a project write must reach a watcher: {events:?}"
    );
    assert!(
        events.iter().any(|event| event["origins"]
            .as_array()
            .is_some_and(|origins| origins.iter().any(|origin| origin == "desktop"))),
        "a Desktop write is named as its origin: {events:?}"
    );
}

#[test]
fn a_watcher_ahead_of_the_sequence_is_told_to_resync() {
    let harness = Harness::start();
    let reply = harness.unary(
        &format!("{PROJECT_SERVICE}/SaveProject"),
        Some(TEST_TOKEN),
        r#"{"project":{"id":"proj-resync","name":"Resync"}}"#,
    );
    assert_eq!(reply.status, 200, "{}", reply.text());
    harness.core.bus.flush();
    let current = harness.core.bus.subscribe(0).current;
    assert!(current > 0);

    // A client that saw a previous core's sequence must reload everything.
    let events = watch(&harness, &(current + 5).to_string(), 1);
    assert_eq!(events.len(), 1, "{events:?}");
    assert_eq!(events[0]["resync"], true, "{events:?}");
    assert!(events[0]["kinds"].is_null(), "{events:?}");
}

#[test]
fn a_watch_stream_pushes_a_change_before_it_ends() {
    let harness = Harness::start();
    let body = super::connect::envelope(0, br#"{"afterSeq":"0"}"#);
    let mut stream = harness.open_stream(WATCH_PATH, Some(TEST_TOKEN), &body);
    assert_eq!(stream.status(), 200);
    assert_eq!(stream.content_type(), "application/connect+json");

    let (flags, payload) = stream
        .next_frame(Duration::from_secs(5))
        .expect("the stream answers its first message while it is open");
    assert_eq!(flags, 0, "the first frame carries a message, not the end");
    let first: serde_json::Value = serde_json::from_slice(&payload).expect("JSON");
    assert!(first["at"].is_string(), "{first}");

    // A write on another connection becomes an event the open stream pushes.
    let save = harness.unary(
        &format!("{PROJECT_SERVICE}/SaveProject"),
        Some(TEST_TOKEN),
        r#"{"project":{"id":"proj-live","name":"Live"}}"#,
    );
    assert_eq!(save.status, 200, "{}", save.text());
    harness.core.bus.flush();

    let change = loop {
        let (flags, payload) = stream
            .next_frame(Duration::from_secs(5))
            .expect("a change arrives while the stream is open");
        assert_eq!(flags, 0, "the stream must not end before the change");
        let value: serde_json::Value = serde_json::from_slice(&payload).expect("JSON");
        if value["kinds"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "projects"))
        {
            break value;
        }
    };
    assert!(
        change["origins"]
            .as_array()
            .is_some_and(|origins| origins.iter().any(|origin| origin == "desktop")),
        "a Desktop write is named as its origin: {change}"
    );

    // Stopping the core ends the handler; the end-of-stream frame comes last.
    harness.core.stop();
    let mut last = None;
    while let Some(frame) = stream.next_frame(Duration::from_secs(5)) {
        last = Some(frame);
    }
    let (flags, payload) = last.expect("an end-of-stream frame");
    assert_eq!(flags, END_STREAM_FLAG);
    assert_eq!(payload, b"{}", "a clean end carries the empty message");
}

#[test]
fn a_client_that_leaves_cancels_the_stream_handler() {
    let harness = Harness::start();
    let body = super::connect::envelope(0, br#"{"afterSeq":"0"}"#);
    let mut stream = harness.open_stream(WATCH_PATH, Some(TEST_TOKEN), &body);
    assert!(stream.next_frame(Duration::from_secs(5)).is_some());
    assert_eq!(
        harness.core.bus.subscribers(),
        1,
        "the watch subscribes to the bus"
    );

    drop(stream);

    // The response is gone, so the handler sees its stream closed and returns.
    let started = Instant::now();
    while harness.core.bus.subscribers() > 0 && started.elapsed() < Duration::from_secs(3) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        harness.core.bus.subscribers(),
        0,
        "a client that leaves must cancel its handler"
    );
}

#[test]
fn the_package_routes_every_service_method_once() {
    let routes = super::routes();
    let mut paths: Vec<&str> = routes.iter().map(|route| route.path).collect();
    let total = paths.len();
    paths.sort();
    paths.dedup();
    assert_eq!(paths.len(), total, "two routes share a path");
    assert_eq!(total, 100, "every RPC of polterware/local/v1 is registered");
    assert!(paths.contains(&"/polterware.local.v1.HealthService/Check"));
    assert!(paths.contains(&"/polterware.local.v1.ChangeService/WatchChanges"));
}

#[test]
fn the_project_proto_reads_an_absent_stamp_as_empty() {
    let project = crate::domains::projects::Project {
        id: "proj-1".to_string(),
        name: "One".to_string(),
        status: String::new(),
        ..crate::domains::projects::Project::default()
    };
    let proto = server::project_proto(&project);
    assert_eq!(proto.status, "active");
    assert_eq!(proto.archived_at, "");
    assert_eq!(
        lifecycle::change_proto(&crate::platform::events::Change {
            seq: 3,
            project_ids: vec![],
            kinds: vec!["projects".to_string()],
            origins: vec!["core".to_string()],
            resync: false,
            at: time::OffsetDateTime::UNIX_EPOCH,
        })
        .seq,
        3
    );
    assert_eq!(
        local_v1::Project::default(),
        local_v1::Project {
            id: String::new(),
            ..local_v1::Project::default()
        }
    );
}

#[test]
fn a_project_lists_the_repository_its_component_checks_out() {
    use super::test_support::init_with_repo;
    let harness = Harness::start();
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("fixture-repo");
    std::fs::create_dir(&root).expect("create the clone folder");
    let project = init_with_repo(
        &harness.core.store(),
        "Fixture",
        root.to_str().expect("a UTF-8 path"),
    )
    .expect("a project with one repository");
    let reply = harness.unary(
        &format!("{PROJECT_SERVICE}/ListProjectRepos"),
        Some(TEST_TOKEN),
        &format!(r#"{{"projectId":"{}"}}"#, project.id),
    );
    assert_eq!(reply.status, 200, "{}", reply.text());
    let repos = reply.json()["repos"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(repos.len(), 1, "{}", reply.text());
    assert_eq!(repos[0]["name"], "fixture-repo");
    assert_eq!(repos[0]["projectId"], project.id.as_str());
    assert_eq!(repos[0]["localPath"], root.to_str().expect("a UTF-8 path"));
}

#[test]
fn fixtures_captured_from_the_go_core_replay_against_this_one() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("testdata/connect_fixtures.json"))
            .expect("the fixtures parse");
    let cases = fixtures["cases"].as_array().expect("cases");
    assert!(!cases.is_empty());
    let harness = Harness::start();
    for case in cases {
        let name = case["name"].as_str().expect("a case name");
        let method = case["method"].as_str().unwrap_or("POST");
        let path = case["path"].as_str().expect("a path");
        let token = match case["token"].as_str() {
            Some("test-session-token") => Some(TEST_TOKEN),
            Some(other) => Some(other),
            None => None,
        };
        let content_type = case["content_type"].as_str().unwrap_or("application/json");
        let body = match case["request_json"].as_str() {
            Some(json) => super::connect::envelope(0, json.as_bytes()),
            None => case["body"]
                .as_str()
                .unwrap_or_default()
                .as_bytes()
                .to_vec(),
        };
        let masked: Vec<String> = case["masked"]
            .as_array()
            .map(|keys| {
                keys.iter()
                    .filter_map(|key| key.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_else(|| {
                fixtures["masked"]
                    .as_array()
                    .map(|keys| {
                        keys.iter()
                            .filter_map(|key| key.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default()
            });
        // A watch stays open for the changes after the frames captured here, as
        // the Go core keeps it open, so its cases are read frame by frame and
        // the client leaves once it has them.
        if let Some(frames) = case["frames"].as_array() {
            let mut stream = harness.open_stream(path, token, &body);
            assert_eq!(
                i64::from(stream.status()),
                case["status"].as_i64().unwrap_or_default(),
                "{name}: status"
            );
            assert_eq!(
                stream.content_type(),
                case["response_content_type"].as_str().unwrap_or_default(),
                "{name}: content type"
            );
            let got = read_frames(&mut stream, frames.len());
            assert!(
                got.len() >= frames.len(),
                "{name}: {} frames, wanted at least {}",
                got.len(),
                frames.len()
            );
            for (index, expected) in frames.iter().enumerate() {
                let (flags, payload) = &got[index];
                assert_eq!(
                    i64::from(*flags),
                    expected["flags"].as_i64().unwrap_or_default(),
                    "{name}: frame {index} flags"
                );
                let mut want: serde_json::Value =
                    serde_json::from_str(expected["json"].as_str().unwrap_or("{}"))
                        .unwrap_or(serde_json::Value::Null);
                let mut got_json: serde_json::Value =
                    serde_json::from_slice(payload).unwrap_or(serde_json::Value::Null);
                mask(&mut want, &masked);
                mask(&mut got_json, &masked);
                assert_eq!(
                    got_json,
                    want,
                    "{name}: frame {index} was {}",
                    String::from_utf8_lossy(payload)
                );
            }
            continue;
        }
        let reply = harness.request(method, path, token, Some(content_type), &body);
        assert_eq!(
            i64::from(reply.status),
            case["status"].as_i64().unwrap_or_default(),
            "{name}: status, body {}",
            reply.text()
        );
        assert_eq!(
            reply.content_type,
            case["response_content_type"].as_str().unwrap_or_default(),
            "{name}: content type"
        );
        if let Some(allow) = case["allow"].as_str()
            && !allow.is_empty()
        {
            assert_eq!(reply.allow(), allow, "{name}: Allow");
        }
        if let Some(expected) = case["response"].as_str()
            && !expected.is_empty()
        {
            let mut want: serde_json::Value =
                serde_json::from_str(expected).unwrap_or(serde_json::Value::Null);
            let mut got = reply.json();
            mask(&mut want, &masked);
            mask(&mut got, &masked);
            assert_eq!(got, want, "{name}: body {}", reply.text());
        }
    }
}

/// The first `count` frames of an open stream, waiting at most five seconds for
/// each.
fn read_frames(stream: &mut OpenStream, count: usize) -> Vec<(u8, Vec<u8>)> {
    let mut frames = Vec::new();
    while frames.len() < count {
        match stream.next_frame(Duration::from_secs(5)) {
            Some(frame) => frames.push(frame),
            None => break,
        }
    }
    frames
}
