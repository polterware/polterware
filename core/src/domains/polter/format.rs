//! Port of `backend/domains/polter/format.go`: the shipped format spec and the
//! v1 query language.

use crate::domains::work;

use super::model::go_quote;

/// The part of the Polter format every agent needs: Work cards, tasks,
/// questions, invariants, handoffs, claims, the binding Rules, and where to
/// read first.
///
/// `polterware format`, the plugin skill, and `polter.format` print it; the
/// rest is in [`spec_topics`], read on demand.
pub const SPEC_CORE: &str = concat!(
    "Polter format v1: markdown that writes Work cards. One apply is one audited change.\n",
    "\n",
    "---\n",
    "polter: 1\n",
    "summary: One line saying what changes\n",
    "rationale: Why it changes\n",
    "---\n",
    "\n",
    "## git-import: Git metadata import\n",
    "stage: build · under: core/import · milestone: beta · decisions: Decision-0007\n",
    "\n",
    "Intent: what we want and why.\n",
    "\n",
    "- [ ] T1 Read commits per checkout `backend/import/*`\n",
    "- [x] T2 Stamp revision on facts\n",
    "? Include merge commits? [blocking] (yes | no)\n",
    "! No network calls during import\n",
    "\n",
    "handoff:\n",
    "branch: feat/git-import\n",
    "next: wire into import\n",
    "\n",
    "Rules:\n",
    "- `## slug: Title` is a card; the lowercase-kebab slug is its identity. Stages: shape, plan, build, verify, delivered. under: is a Structure component path; milestone: is a milestone slug (empty is the Backlog).\n",
    "- Tasks: [ ] open, [x] done, [-] dropped; T<n> and a trailing `scope` glob are optional. `?` is a question (`? Question -> answer` answers it), `!` an invariant.\n",
    "- handoff: needs branch:; it also takes goal, done, in_progress, next, touched, agent.\n",
    "- Claim a task before editing it: `claim: T2`; `done: T2` finishes it, `release: T2` lets go.\n",
    "- A text changes only what it mentions. Remove with status: removed, [-], or a trailing [removed].\n",
    "- Read `polterware brief <slug|path|file>` before you edit; its Rules: bind you; raise a Conflict when a request breaks one. Find with `polterware search <terms>`. Start with `polterware status`; end with a handoff.\n",
    "- More: `polterware format structure|rules|feature|conflicts`. Check with `polterware lint <file>`; write with `polterware apply <file|-> [--dry-run]`.\n",
);

/// The size `polter.format` declares for [`SPEC_CORE`]; a test keeps it within
/// 10% of the measured text.
pub const SPEC_CORE_TOKENS: i32 = 360;

/// The rest of the format, one block per topic.
pub fn spec_topics() -> [(&'static str, &'static str); 4] {
    [
        (
            "structure",
            concat!(
                "Polter Structure nodes.\n",
                "\n",
                "# core/import: Import\n",
                "level: component · under: core · kind: service · tech: Go · repo: api · path: src/import\n",
                "\n",
                "Intent paragraph.\n",
                "\n",
                "! No network calls during import\n",
                "\n",
                "- `# path: Title` is a Structure node. level: domain or component. A domain sits under the project (no under:); a component sits under a domain (under: names its path) or loose under the project. A different under: on an existing component moves it and rewrites its path.\n",
                "- kind: says what a component is, one or more comma separated (frontend, service, api, worker, database, cache, queue, storage, auth, llm, and the other canvas component kinds; only components have one, empty clears); tech: lists its technologies in free text, comma separated (Go, Postgres; empty clears).\n",
                "- How nodes relate is drawn on a canvas, not declared here. deploys: lists env=provider:resource[@id] links (comma separated; a present line replaces them, empty clears); @id is the platform's own id for the resource, which the core finds by name when it is left out, and `polterware deploys` shows each link's latest deploy.\n",
                "- Only a component has a checkout (a domain organizes components; it refuses repo: and path:, and a component that becomes a domain clears them). dir: <folder> links a local clone: it creates the repository (or reuses the one cloned there, or with the same GitHub remote) and becomes repo: <name>. repo: names a repository (`polterware repo list`); with one repository, none is needed. path: is a folder inside it, not the node path. A brief of a file finds the component with the longest repo + path prefix. A repository no node uses is removed.\n",
                "- migrating: old design -> new design marks a node in transition (empty clears it). A card with `kind: architecture` is an Architecture Change: its under: names the node it changes; clear migrating: when the card is delivered.\n",
                "- A `!` line under a node is a Rule for that node and every node below it (see `polterware format rules`); `[removed]` retires it.\n",
                "- A card under: names a component path once any component exists.\n",
            ),
        ),
        (
            "rules",
            concat!(
                "Polter Rules: the law agents follow.\n",
                "\n",
                "#### rule local-only: Never call the network from the core.\n",
                "scope: core, core/import\n",
                "status: active\n",
                "why: The core reads the local store only.\n",
                "Offline installs and tests must not need a network.\n",
                "\n",
                "- `#### rule slug: Statement` creates a Rule or updates it by its lowercase-kebab slug (or id). A Rule is one short imperative constraint, and the only law: a card cites Rules, not decisions.\n",
                "- `scope:` lists the Structure node paths it holds for, comma separated; a node's Rules also hold for every node below it. None (or empty) is the whole project.\n",
                "- `why:` is the optional Markdown reason; the lines after it are its body, so it may span lines, and the body ends at the next `#` to `####` heading (a `#####` or `######` heading stays text). It does not enter the brief.\n",
                "- `status:` is proposed, active, under review (under_review), replaced, or removed. Proposed does not bind; active and under review do; removed retires the Rule.\n",
                "- `replaced-by:` names the Rule (slug or id) that replaced it and sets the status to replaced; a Rule cannot replace itself. Replacing needs a live Rule.\n",
                "- `polterware query \"rules [status]\"` lists them and `polterware query \"show <rule>\"` renders one; `polterware brief` opens with every binding Rule that holds for its target, never cut by the budget, and the generated AGENTS.md block lists them. Follow them; change one only with a rationale saying why.\n",
            ),
        ),
        (
            "conflicts",
            concat!(
                "Polter Conflicts: when a request goes against a Rule.\n",
                "\n",
                "#### conflict cache-in-core: Add a Redis cache to the core\n",
                "rule: local-only\n",
                "card: git-import\n",
                "options: Keep it local | Allow Redis for this card\n",
                "\n",
                "- Do not act against a Rule: raise a Conflict with `#### conflict slug: <the request>` and `rule:` (slug or id). card: and options: (a | b) are optional. One key per line.\n",
                "- A person answers it in the Inbox. Go on with work it does not block; `polterware conflict wait <slug>` blocks until the answer, and `polterware query \"conflict <slug>\"`, status, and the card's brief show it.\n",
                "- `status: withdrawn` drops one you no longer need. `polterware conflict raise|decide|list` are shortcuts.\n",
            ),
        ),
        (
            "feature",
            concat!(
                "Polter feature flow, tasks, and claims.\n",
                "\n",
                "- New feature: `polterware feature new \"<intent>\"`, then `feature shape|contract|plan <slug>` (`--accept` / `--approve` pass the gate); check with `polterware verify <slug>`.\n",
                "- The spec is three sections: `behavior:` says what the feature does, `edge-cases:` the cases it must handle, and `plan:` its architecture, APIs, and files touched. Each keeps the lines after it as a Markdown body, so it may span lines. A body runs until the next section, a `#` or `##` heading, or a `#### rule` / `#### conflict` block: `###` to `######` headings, `!` lines, `?` questions, and `- [ ]` tasks inside it stay text, so write the tasks, questions, and invariants before the sections. The `!` invariants are the acceptance criteria.\n",
                "- The contract gate needs a non-empty behavior: and the plan gate a non-empty plan: plus an open task; the gate names what is missing.\n",
                "- Confirm what you checked: `confirm: ! <criterion>` on the card (one acceptance criterion) or `confirm: rule <slug or id>` (one Rule) stores the confirmation; `polterware feature confirm <slug> --criterion \"...\"` and `--rule <slug>` are the shortcuts. `polterware verify <slug>` lists a confirmed item as pass and the rest as check, and a card in the verify stage with unconfirmed items raises one Inbox item.\n",
                "- A card also takes owner: and decisions: (comma-separated refs). Tasks take `[repo: api]`; a scope may carry its checkout: `api:src/auth/*`.\n",
                "- Claims: `claim: T2` in a card or `polterware task claim slug#T2`. The holder is the session agent; a task or overlapping scope held by another agent is refused. Scopes in different checkouts never overlap.\n",
                "- handoff: records one session on branch: with goal, done, in_progress, next, touched (comma separated), and agent. End a session with `polterware handoff --card <slug> --note \"...\"`.\n",
                "- Milestones are created in the Desktop; `polterware query milestones` lists them and `query roadmap` the active ones with their goal and cards.\n",
            ),
        ),
    ]
}

/// The topics in the order the help prints them.
pub const SPEC_TOPIC_NAMES: [&str; 4] = ["structure", "rules", "feature", "conflicts"];

/// Returns [`SPEC_CORE`] for an empty topic, or one topic's block.
pub fn format_spec(topic: &str) -> Result<String, String> {
    let topic = topic.trim();
    if topic.is_empty() {
        return Ok(SPEC_CORE.to_string());
    }
    if let Some((_, text)) = spec_topics().into_iter().find(|(name, _)| *name == topic) {
        return Ok(text.to_string());
    }
    Err(format!(
        "unknown format topic {}: use {}",
        go_quote(topic),
        SPEC_TOPIC_NAMES.join(", ")
    ))
}

/// The feature flow steps. Go declares these in `feature.go`; the feature
/// planner lands later and re-uses them.
pub const FEATURE_STEP_SHAPE: &str = "shape";
pub const FEATURE_STEP_CONTRACT: &str = "contract";
pub const FEATURE_STEP_PLAN: &str = "plan";

/// Reports whether `step` is a feature flow step.
pub fn valid_feature_step(step: &str) -> bool {
    step == FEATURE_STEP_SHAPE || step == FEATURE_STEP_CONTRACT || step == FEATURE_STEP_PLAN
}

/// A parsed read request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    pub kind: String,
    /// The card, node, or decision the query names; for brief and search it is
    /// the target or the terms.
    pub slug: String,
    pub budget: i32,
    /// The feature flow step for "feature" queries.
    pub step: String,
    /// The point in time for "structure ... at <point>" queries.
    pub at: String,
    /// Asks for the list of change batches ("structure history").
    pub history: bool,
}

pub const QUERY_CARDS: &str = "cards";
pub const QUERY_CARD: &str = "card";
pub const QUERY_STATUS: &str = "status";
pub const QUERY_INBOX: &str = "inbox";
pub const QUERY_HANDOFF: &str = "handoff";
pub const QUERY_BRIEF: &str = "brief";
pub const QUERY_SEARCH: &str = "search";
pub const QUERY_SHOW: &str = "show";
pub const QUERY_COMMITS: &str = "commits";
pub const QUERY_CONFLICTS: &str = "conflicts";
pub const QUERY_CONFLICT: &str = "conflict";
pub const QUERY_DEPLOYS: &str = "deploys";
pub const QUERY_STRUCTURE: &str = "structure";
pub const QUERY_FEATURE: &str = "feature";
pub const QUERY_VERIFY: &str = "verify";

pub const QUERY_RULES: &str = "rules";
pub const QUERY_CANVAS: &str = "canvas";
pub const QUERY_MILESTONES: &str = "milestones";
pub const QUERY_ROADMAP: &str = "roadmap";

const QUERY_USAGE: &str = r##"use "cards", "card <slug>", "status", "inbox", "handoff <slug>", "brief <slug|path|file>", "search <terms>", "show <id>", "commits <slug>", "conflicts", "conflict <slug>", "deploys [<path>]", "structure [<path>] [at <point>]", "structure history", "rules [proposed|active|under_review|replaced|removed]", "feature <slug> [shape|contract|plan]", "verify <slug|slug#T2>", "canvas", "canvas <name|id>", "milestones", or "roadmap""##;

/// The reads retired when Decisions and Facts became Rules, with the read that
/// replaced each. They answer an error, never an alias.
const RETIRED_QUERIES: [(&str, &str); 2] = [
    (
        "decisions",
        "\"rules [proposed|active|under_review|replaced|removed]\"",
    ),
    ("decision", "\"show <rule>\""),
];

/// Parses the v1 query language.
pub fn parse_query(text: &str) -> Result<Query, String> {
    let fields: Vec<&str> = text.split_whitespace().collect();
    if fields.is_empty() {
        return Err(format!(
            "unknown query {}: {QUERY_USAGE}",
            go_quote(text.trim())
        ));
    }
    if let Some((_, replacement)) = RETIRED_QUERIES.iter().find(|(name, _)| fields[0] == *name) {
        return Err(format!(
            "decisions are Rules now: use {replacement} (polterware format rules)"
        ));
    }
    match fields[0] {
        QUERY_BRIEF | QUERY_SEARCH => parse_budget_query(fields[0], &fields[1..]),
        QUERY_SHOW => {
            if fields.len() < 2 {
                return Err(
                    "show needs an id: use \"show <card slug|rule ref|doc|entity|run id>\""
                        .to_string(),
                );
            }
            Ok(Query {
                kind: QUERY_SHOW.to_string(),
                slug: fields[1..].join(" "),
                ..Query::default()
            })
        }
        QUERY_FEATURE => {
            if fields.len() == 2 {
                return Ok(Query {
                    kind: QUERY_FEATURE.to_string(),
                    slug: fields[1].to_string(),
                    step: FEATURE_STEP_SHAPE.to_string(),
                    ..Query::default()
                });
            }
            if fields.len() == 3 && valid_feature_step(fields[2]) {
                return Ok(Query {
                    kind: QUERY_FEATURE.to_string(),
                    slug: fields[1].to_string(),
                    step: fields[2].to_string(),
                    ..Query::default()
                });
            }
            Err(format!(
                "unknown query {}: use \"feature <slug> [shape|contract|plan]\"",
                go_quote(text.trim())
            ))
        }
        QUERY_RULES => {
            if fields.len() == 1 {
                return Ok(Query {
                    kind: QUERY_RULES.to_string(),
                    ..Query::default()
                });
            }
            if fields.len() == 2 && work::valid_rule_status(fields[1]) {
                return Ok(Query {
                    kind: QUERY_RULES.to_string(),
                    slug: fields[1].to_string(),
                    ..Query::default()
                });
            }
            Err(format!(
                "unknown query {}: use \"rules [proposed|active|under_review|replaced|removed]\"",
                go_quote(text.trim())
            ))
        }
        QUERY_VERIFY => {
            if fields.len() != 2 {
                return Err(format!(
                    "unknown query {}: use \"verify <slug>\" or \"verify <slug>#T2\"",
                    go_quote(text.trim())
                ));
            }
            Ok(Query {
                kind: QUERY_VERIFY.to_string(),
                slug: fields[1].to_string(),
                ..Query::default()
            })
        }
        QUERY_CANVAS => {
            if fields.len() == 1 {
                return Ok(Query {
                    kind: QUERY_CANVAS.to_string(),
                    ..Query::default()
                });
            }
            Ok(Query {
                kind: QUERY_CANVAS.to_string(),
                slug: fields[1..].join(" "),
                ..Query::default()
            })
        }
        QUERY_STRUCTURE => {
            let mut args: Vec<&str> = fields[1..].to_vec();
            let mut query = Query {
                kind: QUERY_STRUCTURE.to_string(),
                ..Query::default()
            };
            if args.len() >= 2 && args[args.len() - 2] == "at" {
                query.at = args[args.len() - 1].to_string();
                args.truncate(args.len() - 2);
            }
            match args.len() {
                0 => Ok(query),
                1 if args[0] == "history" && query.at.is_empty() => {
                    query.history = true;
                    Ok(query)
                }
                1 => {
                    query.slug = args[0].to_string();
                    Ok(query)
                }
                _ => Err(format!(
                    "unknown query {}: use \"structure [<path>] [at <point>]\" or \"structure history\"",
                    go_quote(text.trim())
                )),
            }
        }
        _ => match fields.as_slice() {
            [one] if *one == QUERY_CARDS => Ok(Query {
                kind: QUERY_CARDS.to_string(),
                ..Query::default()
            }),
            [one] if *one == QUERY_STATUS => Ok(Query {
                kind: QUERY_STATUS.to_string(),
                ..Query::default()
            }),
            [one] if *one == QUERY_INBOX => Ok(Query {
                kind: QUERY_INBOX.to_string(),
                ..Query::default()
            }),
            [one] if *one == QUERY_MILESTONES => Ok(Query {
                kind: QUERY_MILESTONES.to_string(),
                ..Query::default()
            }),
            [one] if *one == QUERY_ROADMAP => Ok(Query {
                kind: QUERY_ROADMAP.to_string(),
                ..Query::default()
            }),
            [first, slug] if *first == QUERY_COMMITS => Ok(Query {
                kind: QUERY_COMMITS.to_string(),
                slug: (*slug).to_string(),
                ..Query::default()
            }),
            [first, ..] if *first == QUERY_DEPLOYS && fields.len() <= 2 => Ok(Query {
                kind: QUERY_DEPLOYS.to_string(),
                slug: fields[1..].concat(),
                ..Query::default()
            }),
            [one] if *one == QUERY_CONFLICTS => Ok(Query {
                kind: QUERY_CONFLICTS.to_string(),
                ..Query::default()
            }),
            [first, slug] if *first == QUERY_CONFLICT => Ok(Query {
                kind: QUERY_CONFLICT.to_string(),
                slug: (*slug).to_string(),
                ..Query::default()
            }),
            [first, slug] if *first == QUERY_CARD => Ok(Query {
                kind: QUERY_CARD.to_string(),
                slug: (*slug).to_string(),
                ..Query::default()
            }),
            [first, slug] if *first == QUERY_HANDOFF => Ok(Query {
                kind: QUERY_HANDOFF.to_string(),
                slug: (*slug).to_string(),
                ..Query::default()
            }),
            _ => Err(format!(
                "unknown query {}: {QUERY_USAGE}",
                go_quote(text.trim())
            )),
        },
    }
}

/// Reads `<kind> <words...> [budget:N]`: the words are the target or the
/// search terms.
fn parse_budget_query(kind: &str, fields: &[&str]) -> Result<Query, String> {
    let mut budget = 0;
    let mut words: Vec<&str> = Vec::new();
    for field in fields {
        let Some(rest) = field.strip_prefix("budget:") else {
            words.push(field);
            continue;
        };
        let parsed = rest.parse::<i64>();
        let valid = parsed.as_ref().is_ok_and(|n| *n >= 1);
        if !valid {
            return Err(format!(
                "invalid budget {}: use a positive number",
                go_quote(field)
            ));
        }
        budget = i32::try_from(parsed.expect("validated")).unwrap_or(i32::MAX);
    }
    if words.is_empty() {
        if kind == QUERY_SEARCH {
            return Err("search needs terms: use \"search <terms> [budget:N]\"".to_string());
        }
        return Err(
            "brief needs a target: use \"brief <card slug|Structure path|file path> [budget:N]\""
                .to_string(),
        );
    }
    Ok(Query {
        kind: kind.to_string(),
        slug: words.join(" "),
        budget,
        ..Query::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::context::estimate_tokens;

    /// The Claude Code plugin skill embeds the core verbatim; keep them in
    /// sync.
    const PLUGIN_SKILL: &str = include_str!("../../../plugin/skills/polter/SKILL.md");

    #[test]
    fn spec_core_matches_its_declared_size() {
        let got = estimate_tokens(SPEC_CORE);
        assert!(
            got * 10 >= SPEC_CORE_TOKENS * 9 && got * 10 <= SPEC_CORE_TOKENS * 11,
            "SpecCore is {got} tokens; SpecCoreTokens declares {SPEC_CORE_TOKENS} (keep them within 10%)"
        );
        for name in SPEC_TOPIC_NAMES {
            let text = format_spec(name).expect("a known topic");
            assert!(!text.is_empty(), "topic {name} is empty");
        }
        assert_eq!(
            spec_topics().len(),
            SPEC_TOPIC_NAMES.len(),
            "SpecTopicNames lists every topic"
        );
        assert!(format_spec("nope").is_err(), "an unknown topic must fail");
    }

    #[test]
    fn the_plugin_skill_embeds_the_spec() {
        assert!(
            PLUGIN_SKILL.contains(SPEC_CORE),
            "core/plugin/skills/polter/SKILL.md does not contain polter.SpecCore verbatim; copy the SpecCore text into its markdown fence"
        );
    }

    #[test]
    fn parses_the_query_language() {
        let query = parse_query("cards").expect("cards");
        assert_eq!(query.kind, QUERY_CARDS);
        let query = parse_query("brief git-import budget:900").expect("a budget");
        assert_eq!(query.kind, QUERY_BRIEF);
        assert_eq!(query.slug, "git-import");
        assert_eq!(query.budget, 900);
        let query = parse_query("feature git-import").expect("the first step");
        assert_eq!(query.step, FEATURE_STEP_SHAPE);
        let query = parse_query("structure core/import at 2026-01-01").expect("a point");
        assert_eq!(query.at, "2026-01-01");
        assert_eq!(query.slug, "core/import");
        let query = parse_query("structure history").expect("history");
        assert!(query.history);
        let query = parse_query("rules under_review").expect("a status");
        assert_eq!(query.kind, QUERY_RULES);
        assert_eq!(query.slug, "under_review");
        let query = parse_query("deploys core/import").expect("a path");
        assert_eq!(query.slug, "core/import");
    }

    #[test]
    fn refuses_unknown_queries() {
        for (text, fragment) in [
            ("", "unknown query"),
            ("nope", "unknown query"),
            ("show", "show needs an id"),
            ("search", "search needs terms"),
            ("brief", "brief needs a target"),
            ("brief x budget:0", "invalid budget"),
            ("verify a b", "use \"verify <slug>\""),
            ("decisions doing", "decisions are Rules now"),
            ("decision 7", "decisions are Rules now"),
        ] {
            let err = parse_query(text).expect_err(text);
            assert!(err.contains(fragment), "{text}: {err}");
        }
    }
}
