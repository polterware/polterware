//! The fixture the MCP tests share.
//!
//! The rows the assertions read are written here directly: two Rules, two docs,
//! and a card.

use serde_json::{Map, Value, json};

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::knowledge;
use crate::domains::work;
use crate::providers::sqlite::Store;
use crate::providers::sqlite::test_support::TempStore;

use super::{Result, Server, TOOL_POLTER_QUERY};

/// The roadmap the apply tests share (Go's `polter_test.go` constant of the
/// same name).
pub(crate) const ROADMAP: &str = r#"---
polter: 1
summary: Plan git import
rationale: Slice 5 needs a card before coding
---

## git-import: Git metadata import
stage: plan

Ship commit evidence into the canon.

- [ ] T1 Upsert commit evidence
? Include merge commits? (yes | no)

behavior: Read HEAD, branch, and recent commits as evidence.

plan: Follow the commitref provider.
"#;

/// The cards every test starts with, applied as one audited write.
const CARDS: &str = "---
polter: 1
summary: Seed demo cards
rationale: The MCP tests read a card
---

## commits-panel: Commits panel
stage: build

The panel that lists the commits a card did.
";

/// A project in its own temporary database, with the demo rows.
pub(crate) struct TestStore {
    pub temp: TempStore,
    pub project_id: String,
}

impl TestStore {
    pub fn new() -> TestStore {
        let temp = TempStore::new();
        let project = wedge::init_project(
            &temp.store,
            wedge::InitInput {
                name: "Demo".to_string(),
                ..wedge::InitInput::default()
            },
        )
        .expect("init project");
        let project_id = project.id;
        seed(&temp.store, &project_id);
        TestStore { temp, project_id }
    }

    pub fn store(&self) -> &Store {
        &self.temp.store
    }
}

/// Writes the demo rows: two Rules, two docs, and a card.
fn seed(store: &Store, project_id: &str) {
    let rules = [
        (
            "decision-sqlite-local",
            "One local core owns SQLite.",
            "Decision 0001: Local SQLite authority",
        ),
        (
            "fact-no-network",
            "Imports never call the network.",
            "Decision 0002: Deterministic import",
        ),
    ];
    let ops: Vec<work::Op> = rules
        .iter()
        .map(|(id, statement, label)| {
            let mut fields = Map::new();
            fields.insert("statement".into(), json!(statement));
            work::Op {
                entity: work::ENTITY_RULE.to_string(),
                kind: work::OP_CREATE.to_string(),
                entity_id: (*id).to_string(),
                card_slug: (*label).to_string(),
                label: (*label).to_string(),
                fields,
                ..work::Op::default()
            }
        })
        .collect();
    store
        .apply_work_ops(project_id, "", "seed", &ops)
        .expect("seed rules");
    save_doc(
        store,
        project_id,
        "doc-architecture",
        "System architecture",
        "The Desktop reads the core over Connect-JSON.",
    );
    save_doc(
        store,
        project_id,
        "rb-service",
        "Service recovery",
        "Restart the core; it replays the WAL.",
    );

    let result = wedge::apply_polter(
        store,
        wedge::ApplyInput {
            project_id: project_id.to_string(),
            text: CARDS.to_string(),
            auto_start_label: "demo-seed".to_string(),
            ..wedge::ApplyInput::default()
        },
    );
    let result = match result {
        Ok(result) => result,
        Err(failure) => panic!("seed cards: {failure} {:?}", failure.result.diagnostics),
    };
    assert!(
        result.diagnostics.is_empty(),
        "seed cards: {:?}",
        result.diagnostics
    );
}

fn save_doc(store: &Store, project_id: &str, id: &str, title: &str, body: &str) {
    let item = knowledge::new_item(id, project_id, title, body, knowledge::KIND_DOC, Vec::new())
        .expect("doc");
    store.save_knowledge(item).expect("save doc");
}

/// A server with no in/out streams, over the fixture store.
pub(crate) struct TestServer {
    pub server: Server,
    pub fixture: TestStore,
}

impl TestServer {
    pub fn new() -> TestServer {
        TestServer {
            server: Server::new(),
            fixture: TestStore::new(),
        }
    }

    pub fn store(&self) -> &Store {
        self.fixture.store()
    }

    pub fn project_id(&self) -> &str {
        &self.fixture.project_id
    }

    /// Tool arguments with the fixture's project id already set.
    pub fn args(&self, pairs: &[(&str, Value)]) -> Value {
        let mut map = Map::new();
        map.insert(
            "project_id".to_string(),
            Value::String(self.fixture.project_id.clone()),
        );
        for (key, value) in pairs {
            map.insert((*key).to_string(), value.clone());
        }
        Value::Object(map)
    }

    /// The fixture's one query argument, `query`.
    pub fn query_args(&self, text: &str) -> Value {
        self.args(&[("query", json!(text))])
    }

    pub fn call(&mut self, name: &str, args: Value) -> Result<String> {
        let args = match args {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        let TestServer { server, fixture } = self;
        server.call_tool(&fixture.temp.store, name, args)
    }

    /// One Polter read; a failure is a broken fixture, so it panics.
    pub fn query(&mut self, text: &str) -> String {
        let args = self.query_args(text);
        match self.call(TOOL_POLTER_QUERY, args) {
            Ok(text) => text,
            Err(error) => panic!("query {text:?}: {error}"),
        }
    }

    /// Completes the run this session holds open.
    pub fn close_session(&mut self) {
        let TestServer { server, fixture } = self;
        server.close(&fixture.temp.store);
    }

    /// Lists the project's runs; the seeded ones are what the tests subtract.
    pub fn list_runs(&self) -> Vec<agentrun::Run> {
        self.store()
            .list_agent_runs(&self.fixture.project_id, 50)
            .expect("list runs")
    }

    /// The project's runs that were not open before `seeded` was taken.
    pub fn new_runs(&self, seeded: &[agentrun::Run]) -> Vec<agentrun::Run> {
        let known: Vec<String> = seeded.iter().map(|run| run.id.clone()).collect();
        self.list_runs()
            .into_iter()
            .filter(|run| !known.contains(&run.id))
            .collect()
    }
}
