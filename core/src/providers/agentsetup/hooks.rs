//! The plugin's switchable behaviors. `hooks.json` stays fixed; each
//! `polterware hook <name>` asks the core.

/// One behavior of the plugin that the operator can switch off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hook {
    pub name: &'static str,
    pub label: &'static str,
    pub note: &'static str,
}

/// The fixed list of switches, in display order.
pub static HOOKS: [Hook; 5] = [
    Hook {
        name: "status",
        label: "Session-start status",
        note: "Adds the project status to the session context.",
    },
    Hook {
        name: "stop",
        label: "Stop handoff reminder",
        note: "Reminds the agent to write a handoff before it ends.",
    },
    Hook {
        name: "lint",
        label: ".pw.md lint",
        note: "Lints Polter files the agent writes.",
    },
    Hook {
        name: "context",
        label: "File context",
        note: "Adds the brief of the Structure node a file belongs to when the agent first reads or edits it in a session.",
    },
    Hook {
        name: "session",
        label: "Session AgentRun",
        note: "Groups the session's writes into one AgentRun.",
    },
];

/// Reports whether `name` is a switch.
pub fn valid_hook(name: &str) -> bool {
    HOOKS.iter().any(|hook| hook.name == name)
}

/// Maps a `polterware hook` subcommand to its switch.
pub fn switch_for(subcommand: &str) -> String {
    match subcommand {
        "session-start" | "session-end" => "session".to_string(),
        _ => subcommand.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::agentsetup::plugin::PLUGIN_FILES;
    use serde_json::Value;

    fn hooks_json() -> String {
        PLUGIN_FILES
            .iter()
            .find(|(rel, _)| *rel == "hooks/hooks.json")
            .map(|(_, body)| (*body).to_string())
            .expect("embedded hooks.json")
    }

    #[test]
    fn hooks_json_calls_only_known_switches() {
        let body = hooks_json();
        for sub in [
            "session-start",
            "session-end",
            "status",
            "stop",
            "lint",
            "context",
        ] {
            assert!(
                body.contains(&format!("polterware hook {sub}")),
                "hooks.json does not call {sub}"
            );
            assert!(valid_hook(&switch_for(sub)), "{sub} has no switch");
        }
    }

    #[test]
    fn context_hook_runs_after_reads_and_edits() {
        let config: Value = serde_json::from_str(&hooks_json()).expect("hooks.json");
        let groups = config
            .get("hooks")
            .and_then(|hooks| hooks.get("PostToolUse"))
            .and_then(Value::as_array)
            .expect("PostToolUse hooks");
        for group in groups {
            let matcher = group
                .get("matcher")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let hooks = group
                .get("hooks")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for hook in hooks {
                let command = hook
                    .get("command")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if command.ends_with("polterware hook context") {
                    assert_eq!(matcher, "Read|Edit|Write|MultiEdit", "context matcher");
                    return;
                }
            }
        }
        panic!("hooks.json has no PostToolUse context hook");
    }

    #[test]
    fn switches_name_the_subcommands_they_cover() {
        assert_eq!(switch_for("session-start"), "session");
        assert_eq!(switch_for("session-end"), "session");
        assert_eq!(switch_for("lint"), "lint");
        assert!(!valid_hook("session-start"));
        assert_eq!(HOOKS.len(), 5);
    }
}
