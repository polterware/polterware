//! The Polter reads and the audited
//! write that joins the session's run.

use serde_json::{Map, Value};

use crate::compositions::wedge;
use crate::domains::polter;
use crate::providers::sqlite::Store;

use super::helpers::{bool_from, scope_refs_from, string_from};
use super::{Error, Result, TOOL_POLTER_APPLY};

impl super::Server {
    /// Plans Polter text and, unless `dry_run`, applies it as one audited
    /// write. Without `agent_run_id` the write joins this MCP session's run.
    pub(crate) fn polter_apply(
        &mut self,
        store: &Store,
        args: &Map<String, Value>,
    ) -> Result<String> {
        let project_id = string_from(args.get("project_id"));
        if project_id.is_empty() {
            return Err(Error::message("project_id is required"));
        }
        let text = string_from(args.get("text"));
        if text.trim().is_empty() {
            return Err(Error::message(
                "text is required: Polter format, see polter.format",
            ));
        }
        let dry_run = bool_from(args.get("dry_run"));
        let input = wedge::ApplyInput {
            project_id,
            text,
            dry_run,
            summary: string_from(args.get("summary")),
            rationale: string_from(args.get("rationale")),
            agent_run_id: string_from(args.get("agent_run_id")),
            tool: TOOL_POLTER_APPLY.to_string(),
            agent: self.session.label.clone(),
            ..wedge::ApplyInput::default()
        };
        let result = match wedge::apply_in_session(store, &mut self.session, input) {
            Ok(result) => result,
            Err(failure) => return Err(polter_error(&failure.result, &failure.error)),
        };
        let lines = diagnostic_lines(&result.diagnostics);
        let hints = if lines.is_empty() {
            String::new()
        } else {
            format!("{}\n", lines.join("\n"))
        };
        if dry_run {
            return Ok(hints + &polter::format_plan(&result.ops));
        }
        if result.ops.is_empty() {
            return Ok(hints + "no changes\n");
        }
        Ok(format!(
            "{hints}applied {} (agent_run_id {})\n",
            polter::summarize(&result.ops),
            result.agent_run_id
        ))
    }

    /// Answers one Polter read inside the resolved scope.
    pub(crate) fn polter_query(
        &mut self,
        store: &Store,
        args: &Map<String, Value>,
    ) -> Result<String> {
        let project_id = string_from(args.get("project_id"));
        if project_id.is_empty() {
            return Err(Error::message("project_id is required"));
        }
        let mut query = string_from(args.get("query"));
        if query.is_empty() {
            query = polter::QUERY_CARDS.to_string();
        }
        let scope = wedge::resolve_scope_refs(
            store,
            &project_id,
            &scope_refs_from(args.get("scope")),
            bool_from(args.get("scope_exact")),
        )
        .map_err(|error| Error::message(error.to_string()))?;
        wedge::query_polter_scoped(store, &project_id, &query, &scope)
            .map_err(|error| Error::message(error.to_string()))
    }
}

/// Turns invalid text into one line per diagnostic, each with its fix.
fn polter_error(result: &wedge::ApplyResult, error: &wedge::Error) -> Error {
    if !error.is_polter_invalid() {
        return Error::message(error.to_string());
    }
    Error::message(diagnostic_lines(&result.diagnostics).join("\n"))
}

/// Renders one line per diagnostic; hints are marked so they read as advice,
/// not failures.
fn diagnostic_lines(diags: &[polter::Diagnostic]) -> Vec<String> {
    diags
        .iter()
        .map(|diagnostic| {
            let mut message = diagnostic.message.clone();
            if diagnostic.severity == polter::SEVERITY_HINT {
                message = format!("hint: {message}");
            }
            if diagnostic.line > 0 {
                format!("line {}: {message}", diagnostic.line)
            } else {
                message
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::test_support::{ROADMAP, TestServer};
    use super::super::{TOOL_POLTER_APPLY, TOOL_POLTER_FORMAT};
    use crate::domains::agentrun;
    use crate::domains::polter::{SPEC_CORE, spec_topics};

    #[test]
    fn polter_format_returns_the_spec() {
        let mut server = TestServer::new();
        let text = server
            .call(TOOL_POLTER_FORMAT, json!({}))
            .expect("format core");
        assert_eq!(text, SPEC_CORE);

        let topic = server
            .call(TOOL_POLTER_FORMAT, json!({"topic": "rules"}))
            .expect("format rules");
        let want = spec_topics()
            .into_iter()
            .find(|(name, _)| *name == "rules")
            .map(|(_, text)| text)
            .expect("rules topic");
        assert_eq!(topic, want);

        assert!(
            server
                .call(TOOL_POLTER_FORMAT, json!({"topic": "nope"}))
                .is_err(),
            "an unknown topic must fail"
        );
    }

    #[test]
    fn polter_writes_and_reads_a_rule_with_every_field() {
        let mut server = TestServer::new();
        server.server.session.label = "claude-code".to_string();
        let text = concat!(
            "---\npolter: 1\nsummary: Add a Rule\nrationale: The law lives in Rules\n---\n\n",
            "#### rule no-network: Never call the network.\n",
            "why: Offline installs must work.\n",
            "\n",
            "#### rule next-law: Read the local store.\n",
            "status: proposed\n",
        );
        let args = server.args(&[("text", json!(text))]);
        let applied = server.call(TOOL_POLTER_APPLY, args).expect("apply");
        assert!(applied.starts_with("applied 2"), "apply: {applied}");

        let list = server.query("rules");
        assert!(list.contains("proposed 1 · active 3"), "{list}");
        assert!(
            list.contains("- no-network · active · Never call the network."),
            "{list}"
        );
        let proposed = server.query("rules proposed");
        assert!(proposed.contains("next-law"), "{proposed}");
        assert!(!proposed.contains("no-network ·"), "{proposed}");

        let one = server.query("show no-network");
        assert!(
            one.contains("rule no-network [active] Never call the network."),
            "{one}"
        );
        assert!(one.contains("Offline installs must work."), "{one}");
    }

    #[test]
    fn polter_apply_uses_one_session_run() {
        let mut server = TestServer::new();
        server.server.session.label = "claude-code".to_string();
        let seeded = server.list_runs();

        let args = server.args(&[("text", json!(ROADMAP)), ("dry_run", json!(true))]);
        let plan = server.call(TOOL_POLTER_APPLY, args).expect("dry run");
        assert!(plan.contains("+ card git-import"), "dry run plan: {plan}");
        let opened = server.new_runs(&seeded);
        assert_eq!(opened.len(), 0, "dry run opened {} runs", opened.len());

        let args = server.args(&[("text", json!(ROADMAP))]);
        server.call(TOOL_POLTER_APPLY, args).expect("apply");
        let stage = "---\npolter: 1\nsummary: Start build\nrationale: Plan approved\n---\n\n## git-import\nstage: build\n";
        let args = server.args(&[("text", json!(stage))]);
        server.call(TOOL_POLTER_APPLY, args).expect("move stage");
        let args = server.args(&[("text", json!(stage))]);
        let again = server.call(TOOL_POLTER_APPLY, args).expect("reapply");
        assert!(again.contains("no changes"), "reapply: {again}");

        let runs = server.new_runs(&seeded);
        assert_eq!(
            runs.len(),
            1,
            "expected one session run, got {}",
            runs.len()
        );
        assert_eq!(runs[0].status, agentrun::STATUS_OPEN);
        assert_eq!(runs[0].agent_label, "claude-code");
        let record = server
            .store()
            .get_agent_run_record(&runs[0].id)
            .expect("run record");
        assert_eq!(record.steps.len(), 2, "steps: {:?}", record.steps);

        let card = server.query("card git-import");
        assert!(card.contains("stage: build"), "query card: {card}");

        server.close_session();
        let run = server
            .store()
            .get_agent_run(&runs[0].id)
            .expect("run after close");
        assert_eq!(
            run.status,
            agentrun::STATUS_COMPLETED,
            "session run not completed"
        );
    }

    #[test]
    fn polter_apply_rejects_invalid_text_without_a_run() {
        let mut server = TestServer::new();
        let seeded = server.list_runs();
        let bad = "---\npolter: 1\nsummary: x\nrationale: y\n---\n\n- [ ] orphan task\n";
        let args = server.args(&[("text", json!(bad))]);
        let error = match server.call(TOOL_POLTER_APPLY, args) {
            Ok(text) => panic!("invalid text applied: {text}"),
            Err(error) => error,
        };
        assert!(
            error.0.contains("line 7"),
            "expected line diagnostic, got {error}"
        );

        let missing = ROADMAP.replace("summary: Plan git import\n", "");
        let args = server.args(&[("text", json!(missing))]);
        assert!(
            server.call(TOOL_POLTER_APPLY, args).is_err(),
            "expected missing summary error"
        );
        let runs = server.new_runs(&seeded);
        assert_eq!(runs.len(), 0, "invalid writes opened {} runs", runs.len());
    }

    #[test]
    fn a_write_without_a_project_is_refused() {
        let mut server = TestServer::new();
        let error = match server.call(TOOL_POLTER_APPLY, json!({"text": ROADMAP})) {
            Ok(text) => panic!("a write without project_id answered {text}"),
            Err(error) => error,
        };
        assert_eq!(error.0, "project_id is required");
        let error = match server.call(TOOL_POLTER_APPLY, json!({"project_id": "p", "text": " "})) {
            Ok(text) => panic!("empty text answered {text}"),
            Err(error) => error,
        };
        assert_eq!(
            error.0,
            "text is required: Polter format, see polter.format"
        );
    }
}
