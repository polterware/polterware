//! The planner: the operations that make the store match a decoded document.
//!
//! The planner diffs the desired state in a decoded document against a store
//! snapshot and answers the operations that make the store match. A text only
//! touches what it mentions: absent cards, fields, and items stay as they are,
//! and removal is explicit. Plan output is terraform-style, one line each.

use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};

use crate::domains::{structure, work};
use crate::platform::ids;

use super::*;

/// Diffs `doc` against `snapshot` and returns the operations that make the
/// store match. A text only touches what it mentions: absent cards, fields, and
/// items are left as they are. Removal is explicit.
pub fn plan(doc: &Document, snapshot: &work::Snapshot) -> (Vec<work::Op>, Vec<Diagnostic>) {
    plan_as(doc, snapshot, "")
}

/// [`plan`] for a session agent: `holder` is the label claim, done, and release
/// lines act for. Without a holder those lines are errors.
pub fn plan_as(
    doc: &Document,
    snapshot: &work::Snapshot,
    holder: &str,
) -> (Vec<work::Op>, Vec<Diagnostic>) {
    let mut p = Planner::new(snapshot, holder);
    p.diags
        .extend(check_graph(doc, &snapshot.nodes, &snapshot.cards));
    p.diags.extend(check_repos(doc, &snapshot.repos));
    p.diags.extend(check_deploys(doc, &snapshot.environments));
    p.plan_nodes(doc);
    for node in &doc.cards {
        match snapshot.card(&node.slug) {
            Some(existing) => p.update_card(node, existing.clone()),
            None => p.create_card(node),
        }
    }
    // The Decision, Fact, Rule, and Conflict planners live in their own
    // submodules, next to the code each one mirrors in the Go package. They take
    // the planner's own state so a text plans in the same order Go runs it.
    plan_decisions(&mut p, doc);
    plan_facts(&mut p, doc);
    plan_rules(&mut p, doc);
    plan_conflicts(&mut p, doc);
    (p.ops, p.diags)
}

/// Decision blocks are retired: a `###` block now points at `#### rule`, which
/// is the only law. The codec still reads the syntax so old text gets a clear
/// error instead of an unknown heading.
pub fn plan_decisions(p: &mut Planner, doc: &Document) {
    for node in &doc.decisions {
        if p.failed(node.line) {
            continue;
        }
        p.errorf(
            node.line,
            "decisions are Rules now: use #### rule <slug>: Statement".to_string(),
        );
    }
}

/// Fact blocks are retired: a `#### slug: Statement` block now points at
/// `#### rule`.
pub fn plan_facts(p: &mut Planner, doc: &Document) {
    for node in &doc.facts {
        let mut failed = p.failed(node.line);
        for entry in &node.meta {
            failed = failed || p.failed(entry.line);
        }
        if failed {
            continue;
        }
        p.errorf(
            node.line,
            "facts are Rules now: use #### rule <slug>: Statement".to_string(),
        );
    }
}

/// The one index of active claims, by task id.
#[derive(Debug, Clone, PartialEq)]
pub struct HeldClaim {
    pub claim: work::Claim,
    pub card_slug: String,
    pub card_id: String,
    pub number: i32,
}

/// The state one plan builds: the snapshot it diffs, the ops and diagnostics it
/// accumulates, the ids it assigns, and the project state it keeps current as
/// it plans.
#[derive(Debug, Clone)]
pub struct Planner {
    pub snapshot: work::Snapshot,
    pub ops: Vec<work::Op>,
    pub diags: Vec<Diagnostic>,
    pub next_position: i32,
    pub next_node: i32,
    pub holder: String,
    /// The project's structure nodes by path, including ones this text creates.
    pub nodes: HashMap<String, structure::Node>,
    /// The project's active Rules, updated as ops are planned.
    pub rules: Vec<work::Rule>,
    /// The project's active claims by task id, updated as ops are planned.
    pub claims: HashMap<String, HeldClaim>,
    /// Task states this text changes, by task id.
    pub task_state: HashMap<String, String>,
    pub removed_cards: HashSet<String>,
}

impl Planner {
    /// The state one plan starts from `snapshot`: its nodes by path, its active
    /// claims by task id, and its Rules, with the ids the text will assign.
    pub(crate) fn new(snapshot: &work::Snapshot, holder: &str) -> Self {
        let mut p = Self {
            snapshot: snapshot.clone(),
            ops: Vec::new(),
            diags: Vec::new(),
            next_position: snapshot.cards.len() as i32,
            next_node: snapshot.nodes.len() as i32,
            holder: holder.trim().to_string(),
            nodes: HashMap::new(),
            rules: snapshot.rules.clone(),
            claims: HashMap::new(),
            task_state: HashMap::new(),
            removed_cards: HashSet::new(),
        };
        for node in &snapshot.nodes {
            p.nodes.insert(node.path.clone(), node.clone());
        }
        for card in &snapshot.cards {
            for task in &card.tasks {
                if let Some(claim) = &task.claim {
                    p.claims.insert(
                        task.id.clone(),
                        HeldClaim {
                            claim: claim.clone(),
                            card_slug: card.slug.clone(),
                            card_id: card.id.clone(),
                            number: task.number,
                        },
                    );
                }
            }
        }
        p
    }

    /// Adds an error diagnostic at `line`.
    pub fn errorf(&mut self, line: i32, message: impl Into<String>) {
        self.diags.push(Diagnostic::error(line, message));
    }

    /// Adds a hint diagnostic at `line`.
    pub fn hintf(&mut self, line: i32, message: impl Into<String>) {
        self.diags.push(Diagnostic::hint(line, message));
    }

    /// Reports whether `line` already carries an error.
    pub fn failed(&self, line: i32) -> bool {
        self.diags
            .iter()
            .any(|diag| diag.line == line && diag.severity == SEVERITY_ERROR)
    }

    /// Reports a decode error on the node's `deploys:` line. Those errors sit
    /// on the metadata line, not the heading, so `failed(node.line)` misses
    /// them. A bad line must not write the node.
    pub fn failed_deploys(&self, node: &StructureNode) -> bool {
        node.meta
            .iter()
            .any(|entry| entry.key == "deploys" && self.failed(entry.line))
    }

    fn plan_nodes(&mut self, doc: &Document) {
        for node in &doc.nodes {
            if self.failed(node.line) || self.failed_deploys(node) {
                continue;
            }
            let under = node.meta_value("under");
            let has_under = under.is_some();
            let (path, parent, mut from_path, message) =
                resolve_node(&node.path, under.unwrap_or_default(), has_under);
            if !message.is_empty() {
                continue;
            }
            if from_path.is_empty()
                && has_under
                && self.nodes.contains_key(&node.path)
                && path != node.path
            {
                from_path = node.path.clone();
            }
            let mut level = node.meta_value("level").unwrap_or_default().to_string();
            let lookup = if from_path.is_empty() {
                path.clone()
            } else {
                from_path.clone()
            };
            let existing = self.nodes.get(&lookup).cloned();
            if level.is_empty()
                && let Some(existing) = &existing
            {
                level = existing.level.clone();
            }
            let mut parent_id = String::new();
            if !parent.is_empty() {
                match self.nodes.get(&parent) {
                    Some(parent_node) if parent_node.status != structure::NODE_REMOVED => {
                        parent_id = parent_node.id.clone();
                    }
                    _ => continue,
                }
            }
            match existing {
                None => self.create_node(node, path, level, parent_id),
                Some(existing) => {
                    self.update_node(node, existing, path.clone(), level, parent_id);
                    if !from_path.is_empty() && from_path != path {
                        self.nodes.remove(&from_path);
                        self.rewrite_moved_paths(&from_path, &path);
                    }
                }
            }
        }
    }

    fn create_node(
        &mut self,
        node: &StructureNode,
        path: String,
        level: String,
        parent_id: String,
    ) {
        if node.title.is_empty() {
            self.errorf(
                node.line,
                format!(
                    "unknown node {}: add a title (# {}: Title)",
                    go_quote(&path),
                    node.path
                ),
            );
            return;
        }
        if node.meta_value("status") == Some(structure::NODE_REMOVED) {
            self.errorf(
                node.line,
                format!(
                    "cannot remove unknown node {}: check the path",
                    go_quote(&path)
                ),
            );
            return;
        }
        let decisions = node.meta_value("decisions").unwrap_or_default();
        let repo = node.meta_value("repo").unwrap_or_default();
        let folder = node.meta_value("path").unwrap_or_default();
        let kind = node.meta_value("kind").unwrap_or_default();
        let tech = node.meta_value("tech").unwrap_or_default();
        if !kind.is_empty() && level != structure::LEVEL_COMPONENT {
            self.errorf(
                node.meta_line("kind"),
                format!(
                    "kind {} on {} {}: only components have a kind",
                    go_quote(kind),
                    level,
                    go_quote(&path)
                ),
            );
            return;
        }
        // A domain organizes components; only a component has a checkout.
        if (!repo.is_empty() || !folder.is_empty()) && level != structure::LEVEL_COMPONENT {
            self.errorf(
                checkout_meta_line(node),
                format!(
                    "checkout on {} {}: only components have a repo or path",
                    level,
                    go_quote(&path)
                ),
            );
            return;
        }
        let migrating = node.meta_value("migrating").unwrap_or_default();
        let (migration_from, migration_to) = parse_migration(migrating).unwrap_or_default();
        let deploys = match resolve_deploys(node, &[], &self.snapshot.environments) {
            Ok((deploys, _)) => deploys,
            Err(err) => {
                self.errorf(deploys_line(node), err);
                return;
            }
        };
        let id = ids::new_ulid();
        let created = structure::Node {
            id: id.clone(),
            slug: node.slug.clone(),
            path: path.clone(),
            title: node.title.clone(),
            intent: node.intent.clone(),
            level: level.clone(),
            status: structure::NODE_ACTIVE.to_string(),
            parent_id: parent_id.clone(),
            position: self.next_node,
            repo: repo.to_string(),
            folder: folder.to_string(),
            kind: kind.to_string(),
            tech: tech.to_string(),
            migration_from,
            migration_to,
            deploys: deploys.clone(),
            ..structure::Node::default()
        };
        let mut fields = work::Fields::new();
        fields.insert("slug".to_string(), json!(node.slug));
        fields.insert("title".to_string(), json!(node.title));
        fields.insert("intent".to_string(), json!(node.intent));
        fields.insert("level".to_string(), json!(level));
        fields.insert("status".to_string(), json!(structure::NODE_ACTIVE));
        fields.insert("path".to_string(), json!(path));
        fields.insert("parent_id".to_string(), json!(parent_id));
        fields.insert("decisions".to_string(), json!(split_list(decisions)));
        fields.insert("position".to_string(), json!(self.next_node));
        fields.insert("repo".to_string(), json!(repo));
        fields.insert("folder".to_string(), json!(folder));
        fields.insert("kind".to_string(), json!(kind));
        fields.insert("tech".to_string(), json!(tech));
        fields.insert("migration_from".to_string(), json!(created.migration_from));
        fields.insert("migration_to".to_string(), json!(created.migration_to));
        fields.insert("deploys".to_string(), deployments_json(&deploys));
        self.ops.push(work::Op {
            entity: work::ENTITY_NODE.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: id,
            card_slug: path.clone(),
            label: node.title.clone(),
            fields,
            ..work::Op::default()
        });
        self.next_node += 1;
        self.nodes.insert(path.clone(), created.clone());
        node_rules(self, node, &created);
        self.nodes.insert(path, created);
    }

    fn update_node(
        &mut self,
        node: &StructureNode,
        mut existing: structure::Node,
        path: String,
        level: String,
        parent_id: String,
    ) {
        let mut fields = work::Fields::new();
        let mut before = work::Fields::new();
        if !node.title.is_empty() && node.title != existing.title {
            set_field(
                &mut fields,
                &mut before,
                "title",
                s(&existing.title),
                s(&node.title),
            );
        }
        if node.intent_set && node.intent != existing.intent {
            set_field(
                &mut fields,
                &mut before,
                "intent",
                s(&existing.intent),
                s(&node.intent),
            );
        }
        if !level.is_empty() && level != existing.level {
            set_field(
                &mut fields,
                &mut before,
                "level",
                s(&existing.level),
                s(&level),
            );
        }
        if parent_id != existing.parent_id {
            set_field(
                &mut fields,
                &mut before,
                "parent_id",
                s(&existing.parent_id),
                s(&parent_id),
            );
        }
        if path != existing.path {
            set_field(
                &mut fields,
                &mut before,
                "path",
                s(&existing.path),
                s(&path),
            );
        }
        if let Some(decisions) = node.meta_value("decisions") {
            let list = split_list(decisions);
            if list != existing.decisions {
                set_field(
                    &mut fields,
                    &mut before,
                    "decisions",
                    json!(existing.decisions),
                    json!(list),
                );
            }
        }
        // A present deploys: replaces the node's links; an empty value clears
        // them. An absent line leaves the stored links alone.
        match resolve_deploys(node, &existing.deploys, &self.snapshot.environments) {
            Err(err) => {
                self.errorf(deploys_line(node), err);
                return;
            }
            Ok((deploys, mentioned)) => {
                if mentioned {
                    if !same_deploys(&deploys, &existing.deploys) {
                        set_field(
                            &mut fields,
                            &mut before,
                            "deploys",
                            deployments_json(&existing.deploys),
                            deployments_json(&deploys),
                        );
                    }
                    existing.deploys = deploys;
                }
            }
        }
        // A present repo: or path: is authoritative, so an empty value moves
        // the node back to the project root or the checkout root.
        if let Some(repo) = node.meta_value("repo")
            && repo != existing.repo
        {
            set_field(&mut fields, &mut before, "repo", s(&existing.repo), s(repo));
        }
        if let Some(folder) = node.meta_value("path")
            && folder != existing.folder
        {
            set_field(
                &mut fields,
                &mut before,
                "folder",
                s(&existing.folder),
                s(folder),
            );
        }
        // A present kind: or tech: is authoritative; an empty value clears it.
        // A node that stops being a component must clear its kind in the same
        // block.
        let mut kind = existing.kind.clone();
        if let Some(value) = node.meta_value("kind") {
            kind = value.to_string();
            if kind != existing.kind {
                set_field(
                    &mut fields,
                    &mut before,
                    "kind",
                    s(&existing.kind),
                    s(&kind),
                );
            }
        }
        let effective = if level.is_empty() {
            existing.level.clone()
        } else {
            level.clone()
        };
        if !kind.is_empty() && effective != structure::LEVEL_COMPONENT {
            self.errorf(
                node.meta_line("kind"),
                format!(
                    "kind {} on {} {}: only components have a kind (write kind: to clear it)",
                    go_quote(&kind),
                    effective,
                    go_quote(&path)
                ),
            );
            return;
        }
        // Only a component has a checkout: a node that stops being one clears
        // its repo and path in the same block.
        let mut repo = existing.repo.clone();
        if let Some(value) = node.meta_value("repo") {
            repo = value.to_string();
        }
        let mut folder = existing.folder.clone();
        if let Some(value) = node.meta_value("path") {
            folder = value.to_string();
        }
        if (!repo.is_empty() || !folder.is_empty()) && effective != structure::LEVEL_COMPONENT {
            self.errorf(
                checkout_meta_line(node),
                format!(
                    "checkout on {} {}: only components have a repo or path (write repo: and path: to clear them)",
                    effective,
                    go_quote(&path)
                ),
            );
            return;
        }
        if let Some(tech) = node.meta_value("tech")
            && tech != existing.tech
        {
            set_field(&mut fields, &mut before, "tech", s(&existing.tech), s(tech));
        }
        // A present migrating: is authoritative; an empty value clears the
        // state.
        let mut migration_from = existing.migration_from.clone();
        let mut migration_to = existing.migration_to.clone();
        if let Some(value) = node.meta_value("migrating") {
            let (from, to) = parse_migration(value).unwrap_or_default();
            if from != existing.migration_from {
                set_field(
                    &mut fields,
                    &mut before,
                    "migration_from",
                    s(&existing.migration_from),
                    s(&from),
                );
            }
            if to != existing.migration_to {
                set_field(
                    &mut fields,
                    &mut before,
                    "migration_to",
                    s(&existing.migration_to),
                    s(&to),
                );
            }
            migration_from = from;
            migration_to = to;
        }
        let status = node.meta_value("status").unwrap_or_default().to_string();
        let has_status = node.meta_value("status").is_some();
        if has_status && status != existing.status {
            set_field(
                &mut fields,
                &mut before,
                "status",
                s(&existing.status),
                s(&status),
            );
        }
        if !fields.is_empty() {
            let kind =
                if fields.get("status").and_then(Value::as_str) == Some(structure::NODE_REMOVED) {
                    work::OP_REMOVE
                } else {
                    work::OP_UPDATE
                };
            self.ops.push(work::Op {
                entity: work::ENTITY_NODE.to_string(),
                kind: kind.to_string(),
                entity_id: existing.id.clone(),
                card_slug: path.clone(),
                label: existing.title.clone(),
                fields,
                before,
                ..work::Op::default()
            });
        }
        if status == structure::NODE_REMOVED {
            existing.status = structure::NODE_REMOVED.to_string();
            self.nodes.insert(path, existing);
            return;
        }
        if !level.is_empty() {
            existing.level = level;
        }
        existing.parent_id = parent_id;
        existing.path = path.clone();
        if let Some(value) = node.meta_value("repo") {
            existing.repo = value.to_string();
        }
        if let Some(value) = node.meta_value("path") {
            existing.folder = value.to_string();
        }
        existing.kind = kind;
        if let Some(value) = node.meta_value("tech") {
            existing.tech = value.to_string();
        }
        existing.migration_from = migration_from;
        existing.migration_to = migration_to;
        node_rules(self, node, &existing);
        self.nodes.insert(path, existing);
    }

    /// Updates descendants and work cards after a node changes path.
    fn rewrite_moved_paths(&mut self, old_path: &str, new_path: &str) {
        if old_path.is_empty() || old_path == new_path {
            return;
        }
        let prefix = format!("{old_path}/");
        let mut olds: Vec<String> = self
            .nodes
            .keys()
            .filter(|path| path.starts_with(&prefix))
            .cloned()
            .collect();
        olds.sort();
        for old in olds {
            let Some(mut node) = self.nodes.remove(&old) else {
                continue;
            };
            let next = format!("{new_path}{}", &old[old_path.len()..]);
            node.path = next.clone();
            self.nodes.insert(next.clone(), node.clone());
            self.ops.push(work::Op {
                entity: work::ENTITY_NODE.to_string(),
                kind: work::OP_UPDATE.to_string(),
                entity_id: node.id,
                card_slug: next.clone(),
                label: node.title,
                fields: one_field("path", s(&next)),
                before: one_field("path", s(&old)),
                ..work::Op::default()
            });
        }
        for card in &self.snapshot.cards {
            let under = card.under.clone();
            if under != old_path && !under.starts_with(&prefix) {
                continue;
            }
            let next = format!("{new_path}{}", &under[old_path.len()..]);
            self.ops.push(work::Op {
                entity: work::ENTITY_CARD.to_string(),
                kind: work::OP_UPDATE.to_string(),
                entity_id: card.id.clone(),
                card_id: card.id.clone(),
                card_slug: card.slug.clone(),
                label: card.title.clone(),
                fields: one_field("under", s(&next)),
                before: one_field("under", s(&under)),
                ..work::Op::default()
            });
        }
    }

    fn create_card(&mut self, node: &CardNode) {
        if node.title.is_empty() {
            self.errorf(
                node.line,
                format!(
                    "unknown card {}: add a title (## {}: Title)",
                    go_quote(&node.slug),
                    node.slug
                ),
            );
            return;
        }
        if node.meta_value("status") == Some(work::CARD_REMOVED) {
            self.errorf(
                node.line,
                format!(
                    "cannot remove unknown card {}: check the slug",
                    go_quote(&node.slug)
                ),
            );
            return;
        }
        let stage = node
            .meta_value("stage")
            .map(str::to_string)
            .unwrap_or_else(|| work::STAGE_SHAPE.to_string());
        let owner = node.meta_value("owner").unwrap_or_default();
        let under = node.meta_value("under").unwrap_or_default();
        let decisions = node.meta_value("decisions").unwrap_or_default();
        let kind = node
            .meta_value("kind")
            .map(str::to_string)
            .unwrap_or_else(|| work::KIND_FEATURE.to_string());
        let Some(milestone_id) = self.milestone_id(node) else {
            return;
        };
        let card = work::Card {
            id: ids::new_ulid(),
            slug: node.slug.clone(),
            ..work::Card::default()
        };
        let mut fields = work::Fields::new();
        fields.insert("slug".to_string(), json!(node.slug));
        fields.insert("title".to_string(), json!(node.title));
        fields.insert("intent".to_string(), json!(node.intent));
        fields.insert("behavior".to_string(), json!(node.behavior));
        fields.insert("edge_cases".to_string(), json!(node.edge_cases));
        fields.insert("plan".to_string(), json!(node.plan));
        fields.insert("stage".to_string(), json!(stage));
        fields.insert("status".to_string(), json!(work::CARD_ACTIVE));
        fields.insert("owner".to_string(), json!(owner));
        fields.insert("kind".to_string(), json!(kind));
        fields.insert("under".to_string(), json!(under));
        fields.insert("decisions".to_string(), json!(split_list(decisions)));
        fields.insert("position".to_string(), json!(self.next_position));
        if !milestone_id.is_empty() {
            fields.insert("milestone".to_string(), json!(milestone_id));
        }
        self.ops.push(work::Op {
            entity: work::ENTITY_CARD.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: card.id.clone(),
            card_id: card.id.clone(),
            card_slug: node.slug.clone(),
            label: node.title.clone(),
            fields,
            ..work::Op::default()
        });
        self.next_position += 1;
        self.children(node, &card);
    }

    /// Resolves the card's `milestone:` slug to a listed milestone id. An
    /// absent or empty line is `""` (the Backlog); an unknown slug is an error.
    fn milestone_id(&mut self, node: &CardNode) -> Option<String> {
        let slug = node.meta_value("milestone").unwrap_or_default();
        if slug.is_empty() {
            return Some(String::new());
        }
        match work::find_milestone(&self.snapshot.milestones, slug) {
            Some(milestone) => Some(milestone.id.clone()),
            None => {
                self.errorf(
                    card_meta_line(node, "milestone"),
                    format!(
                        "unknown milestone {}: list them with polterware query milestones, or create one in the Desktop",
                        go_quote(slug)
                    ),
                );
                None
            }
        }
    }

    fn update_card(&mut self, node: &CardNode, card: work::Card) {
        let mut fields = work::Fields::new();
        let mut before = work::Fields::new();
        if !node.title.is_empty() && node.title != card.title {
            set_field(
                &mut fields,
                &mut before,
                "title",
                s(&card.title),
                s(&node.title),
            );
        }
        if node.intent_set && node.intent != card.intent {
            set_field(
                &mut fields,
                &mut before,
                "intent",
                s(&card.intent),
                s(&node.intent),
            );
        }
        for (key, set, value, current) in [
            (
                "behavior",
                node.behavior_set,
                &node.behavior,
                &card.behavior,
            ),
            (
                "edge_cases",
                node.edge_cases_set,
                &node.edge_cases,
                &card.edge_cases,
            ),
            ("plan", node.plan_set, &node.plan, &card.plan),
        ] {
            if set && value != current {
                set_field(&mut fields, &mut before, key, s(current), s(value));
            }
        }
        for key in META_KEYS {
            let Some(value) = node.meta_value(key) else {
                continue;
            };
            match key {
                "stage" => {
                    if value != card.stage {
                        set_field(&mut fields, &mut before, key, s(&card.stage), s(value));
                    }
                }
                "status" => {
                    if value != card.status {
                        set_field(&mut fields, &mut before, key, s(&card.status), s(value));
                    }
                }
                "owner" => {
                    if value != card.owner {
                        set_field(&mut fields, &mut before, key, s(&card.owner), s(value));
                    }
                }
                "kind" => {
                    if value != card.kind {
                        set_field(&mut fields, &mut before, key, s(&card.kind), s(value));
                    }
                }
                "under" => {
                    if value != card.under {
                        set_field(&mut fields, &mut before, key, s(&card.under), s(value));
                    }
                }
                "milestone" => {
                    let Some(id) = self.milestone_id(node) else {
                        return;
                    };
                    if id != card.milestone_id {
                        set_field(&mut fields, &mut before, key, s(&card.milestone_id), s(&id));
                    }
                }
                "decisions" => {
                    let list = split_list(value);
                    if list != card.decisions {
                        set_field(
                            &mut fields,
                            &mut before,
                            key,
                            json!(card.decisions),
                            json!(list),
                        );
                    }
                }
                _ => {}
            }
        }
        if !fields.is_empty() {
            let kind = if fields.get("status").and_then(Value::as_str) == Some(work::CARD_REMOVED) {
                work::OP_REMOVE
            } else {
                work::OP_UPDATE
            };
            self.ops.push(work::Op {
                entity: work::ENTITY_CARD.to_string(),
                kind: kind.to_string(),
                entity_id: card.id.clone(),
                card_id: card.id.clone(),
                card_slug: card.slug.clone(),
                label: card.title.clone(),
                fields,
                before,
                ..work::Op::default()
            });
            if kind == work::OP_REMOVE {
                self.removed_cards.insert(card.id.clone());
                for task in &card.tasks {
                    self.release_claim(&task.id);
                }
            }
        }
        self.children(node, &card);
    }

    fn children(&mut self, node: &CardNode, card: &work::Card) {
        self.tasks(node, card);
        self.questions(node, card);
        self.invariants(node, card);
        self.handoff(node, card);
        self.coordination(node, card);
        self.confirmations(node, card);
    }

    /// Plans `confirm:` lines: each names one acceptance criterion (an active
    /// `!` invariant) or one binding Rule of the project, and stores the
    /// confirmation for the card.
    fn confirmations(&mut self, node: &CardNode, card: &work::Card) {
        let mut seen: HashSet<(String, String)> = HashSet::new();
        for line in &node.confirmations {
            if card.id.is_empty() || card.status == work::CARD_REMOVED {
                self.errorf(
                    line.line,
                    format!(
                        "cannot confirm for {}: the card is not stored yet",
                        go_quote(&card.slug)
                    ),
                );
                continue;
            }
            let (target_id, label) = if line.kind == CONFIRM_RULE {
                let key = line.target.to_lowercase();
                let found = self.rules.iter().find(|rule| {
                    rule.binding()
                        && (rule.slug.to_lowercase() == key || rule.id.to_lowercase() == key)
                });
                let Some(rule) = found else {
                    self.errorf(
                        line.line,
                        format!(
                            "unknown Rule {}: confirm one that binds, with its slug or id",
                            go_quote(&line.target)
                        ),
                    );
                    continue;
                };
                (rule.id.clone(), format!("rule {}", rule.handle()))
            } else {
                let key = work::normalize_text(&line.target);
                let found = card.invariants.iter().find(|invariant| {
                    invariant.state == work::INVARIANT_ACTIVE
                        && work::normalize_text(&invariant.text) == key
                });
                let Some(invariant) = found else {
                    self.errorf(
                        line.line,
                        format!(
                            "unknown acceptance criterion {}: confirm one of the card's ! lines",
                            go_quote(&line.target)
                        ),
                    );
                    continue;
                };
                (invariant.id.clone(), format!("! {}", invariant.text))
            };
            if !seen.insert((line.kind.clone(), target_id.clone())) {
                continue;
            }
            let confirmed_by = if self.holder.is_empty() {
                "person".to_string()
            } else {
                self.holder.clone()
            };
            let mut fields = work::Fields::new();
            fields.insert("kind".to_string(), json!(line.kind));
            fields.insert("target_id".to_string(), json!(target_id));
            fields.insert("confirmed_by".to_string(), json!(confirmed_by));
            self.ops.push(work::Op {
                entity: work::ENTITY_CONFIRMATION.to_string(),
                kind: work::OP_CREATE.to_string(),
                entity_id: format!("{}:{}:{}", card.id, line.kind, target_id),
                card_id: card.id.clone(),
                card_slug: card.slug.clone(),
                label,
                fields,
                ..work::Op::default()
            });
        }
    }

    /// Plans claim, done, and release lines against stored tasks.
    fn coordination(&mut self, node: &CardNode, card: &work::Card) {
        for line in &node.coordination {
            if self.holder.is_empty() {
                self.errorf(
                    line.line,
                    format!(
                        "{} needs an agent: pass --agent or set POLTERWARE_AGENT_LABEL",
                        line.action
                    ),
                );
                continue;
            }
            let Some(task) = card.tasks.iter().find(|task| task.number == line.number) else {
                self.errorf(
                    line.line,
                    format!(
                        "unknown task T{} in {}: apply the task before you {} it",
                        line.number,
                        go_quote(&card.slug),
                        line.action
                    ),
                );
                continue;
            };
            if card.id.is_empty()
                || card.status == work::CARD_REMOVED
                || self.removed_cards.contains(&card.id)
            {
                self.errorf(
                    line.line,
                    format!(
                        "cannot {} T{}: card {} is removed",
                        line.action,
                        line.number,
                        go_quote(&card.slug)
                    ),
                );
                continue;
            }
            let mut state = task.state.clone();
            if let Some(planned) = self.task_state.get(&task.id) {
                state = planned.clone();
            }
            let held = self.claims.get(&task.id).cloned();
            let claimed = held.is_some();
            match line.action.as_str() {
                COORD_CLAIM => self.claim(line, card, task, &state, held.as_ref()),
                COORD_DONE => {
                    if state == work::TASK_DROPPED {
                        self.errorf(
                            line.line,
                            format!("cannot mark T{} done: it is dropped", line.number),
                        );
                        continue;
                    }
                    if claimed
                        && let Some(held) = &held
                        && held.claim.holder != self.holder
                    {
                        self.errorf(
                            line.line,
                            format!(
                                "cannot mark {} T{} done: {}",
                                card.slug,
                                line.number,
                                describe_claim(held)
                            ),
                        );
                        continue;
                    }
                    if state != work::TASK_DONE {
                        self.task_state
                            .insert(task.id.clone(), work::TASK_DONE.to_string());
                        self.ops.push(work::Op {
                            entity: work::ENTITY_TASK.to_string(),
                            kind: work::OP_UPDATE.to_string(),
                            entity_id: task.id.clone(),
                            card_id: card.id.clone(),
                            card_slug: card.slug.clone(),
                            label: format!("T{}", task.number),
                            fields: one_field("state", s(work::TASK_DONE)),
                            before: one_field("state", s(&state)),
                            ..work::Op::default()
                        });
                    }
                    self.release_claim(&task.id);
                }
                COORD_RELEASE => {
                    if !claimed {
                        continue;
                    }
                    if let Some(held) = &held
                        && held.claim.holder != self.holder
                    {
                        self.errorf(
                            line.line,
                            format!(
                                "cannot release {} T{}: {}",
                                card.slug,
                                line.number,
                                describe_claim(held)
                            ),
                        );
                        continue;
                    }
                    self.release_claim(&task.id);
                }
                _ => {}
            }
        }
    }

    fn claim(
        &mut self,
        line: &CoordNode,
        card: &work::Card,
        task: &work::Task,
        state: &str,
        held: Option<&HeldClaim>,
    ) {
        if state != work::TASK_OPEN {
            self.errorf(
                line.line,
                format!(
                    "cannot claim {} T{}: it is {}",
                    card.slug, line.number, state
                ),
            );
            return;
        }
        if let Some(held) = held {
            if held.claim.holder != self.holder {
                self.errorf(
                    line.line,
                    format!(
                        "cannot claim {} T{}: {}",
                        card.slug,
                        line.number,
                        describe_claim(held)
                    ),
                );
            }
            return;
        }
        let scope = work::qualify_scope(&task.repo, &task.scope);
        if !scope.is_empty() {
            let mut others: Vec<(String, HeldClaim)> = self
                .claims
                .iter()
                .map(|(id, held)| (id.clone(), held.clone()))
                .collect();
            others.sort_by(|a, b| a.0.cmp(&b.0));
            for (other_id, other) in others {
                if other_id == task.id || other.claim.holder == self.holder {
                    continue;
                }
                if work::scopes_overlap(&scope, &other.claim.scope) {
                    self.errorf(
                        line.line,
                        format!(
                            "cannot claim {} T{}: scope {} overlaps {} T{} held by {} ({})",
                            card.slug,
                            line.number,
                            scope,
                            other.card_slug,
                            other.number,
                            other.claim.holder,
                            other.claim.scope
                        ),
                    );
                    return;
                }
            }
        }
        let claim = work::Claim {
            id: ids::new_ulid(),
            project_id: self.snapshot.project_id.clone(),
            task_id: task.id.clone(),
            holder: self.holder.clone(),
            scope: scope.clone(),
            ..work::Claim::default()
        };
        self.claims.insert(
            task.id.clone(),
            HeldClaim {
                claim: claim.clone(),
                card_slug: card.slug.clone(),
                card_id: card.id.clone(),
                number: task.number,
            },
        );
        let mut fields = work::Fields::new();
        fields.insert("task_id".to_string(), s(&task.id));
        fields.insert("holder".to_string(), s(&claim.holder));
        fields.insert("scope".to_string(), s(&claim.scope));
        self.ops.push(work::Op {
            entity: work::ENTITY_CLAIM.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: claim.id,
            card_id: card.id.clone(),
            card_slug: card.slug.clone(),
            label: format!("T{}", task.number),
            fields,
            ..work::Op::default()
        });
    }

    /// Plans the removal of `task_id`'s claim, if it has one.
    pub fn release_claim(&mut self, task_id: &str) {
        let Some(held) = self.claims.remove(task_id) else {
            return;
        };
        let mut fields = work::Fields::new();
        fields.insert("holder".to_string(), s(&held.claim.holder));
        fields.insert("scope".to_string(), s(&held.claim.scope));
        self.ops.push(work::Op {
            entity: work::ENTITY_CLAIM.to_string(),
            kind: work::OP_REMOVE.to_string(),
            entity_id: held.claim.id,
            card_id: held.card_id,
            card_slug: held.card_slug,
            label: format!("T{}", held.number),
            fields,
            ..work::Op::default()
        });
    }

    fn handoff(&mut self, node: &CardNode, card: &work::Card) {
        let Some(h) = &node.handoff else {
            return;
        };
        if h.branch.trim().is_empty() {
            self.errorf(h.line, "handoff needs branch: add \"branch: <name>\"");
            return;
        }
        if let Some(existing) = card.latest_handoff(&h.branch)
            && same_handoff(existing, h)
        {
            return;
        }
        let mut fields = work::Fields::new();
        fields.insert("branch".to_string(), s(&h.branch));
        fields.insert("agent".to_string(), s(&h.agent));
        fields.insert("goal".to_string(), s(&h.goal));
        fields.insert("done".to_string(), s(&h.done));
        fields.insert("in_progress".to_string(), s(&h.in_progress));
        fields.insert("next".to_string(), s(&h.next));
        fields.insert("touched".to_string(), json!(h.touched));
        self.ops.push(work::Op {
            entity: work::ENTITY_HANDOFF.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: ids::new_ulid(),
            card_id: card.id.clone(),
            card_slug: card.slug.clone(),
            label: h.branch.clone(),
            fields,
            ..work::Op::default()
        });
    }

    fn tasks(&mut self, node: &CardNode, card: &work::Card) {
        let mut max_number = 0;
        let mut max_position = -1;
        let mut by_number: HashMap<i32, work::Task> = HashMap::new();
        for task in &card.tasks {
            by_number.insert(task.number, task.clone());
            max_number = max_number.max(task.number);
            max_position = max_position.max(task.position);
        }
        for task in &node.tasks {
            max_number = max_number.max(task.number);
        }
        let mut matched: HashSet<String> = HashSet::new();
        for t in &node.tasks {
            let mut existing: Option<work::Task> = None;
            if t.number > 0 {
                existing = by_number.get(&t.number).cloned();
                if existing.is_none() && !card.tasks.is_empty() {
                    self.errorf(
                        t.line,
                        format!(
                            "unknown task T{} in {}: drop the number to create a new task",
                            t.number,
                            go_quote(&card.slug)
                        ),
                    );
                    continue;
                }
            } else {
                let key = work::normalize_text(&t.title);
                for task in &card.tasks {
                    if !matched.contains(&task.id) && work::normalize_text(&task.title) == key {
                        existing = Some(task.clone());
                        break;
                    }
                }
            }
            let Some(existing) = existing else {
                let mut number = t.number;
                if number == 0 {
                    max_number += 1;
                    number = max_number;
                }
                max_position += 1;
                let mut fields = work::Fields::new();
                fields.insert("number".to_string(), json!(number));
                fields.insert("title".to_string(), json!(t.title));
                fields.insert("state".to_string(), json!(t.state));
                fields.insert("repo".to_string(), json!(t.repo));
                fields.insert("scope".to_string(), json!(t.scope));
                fields.insert("position".to_string(), json!(max_position));
                self.ops.push(work::Op {
                    entity: work::ENTITY_TASK.to_string(),
                    kind: work::OP_CREATE.to_string(),
                    entity_id: ids::new_ulid(),
                    card_id: card.id.clone(),
                    card_slug: card.slug.clone(),
                    label: format!("T{number}"),
                    fields,
                    ..work::Op::default()
                });
                continue;
            };
            matched.insert(existing.id.clone());
            let mut fields = work::Fields::new();
            let mut before = work::Fields::new();
            if t.title != existing.title {
                set_field(
                    &mut fields,
                    &mut before,
                    "title",
                    json!(existing.title),
                    json!(t.title),
                );
            }
            if t.state != existing.state {
                set_field(
                    &mut fields,
                    &mut before,
                    "state",
                    json!(existing.state),
                    json!(t.state),
                );
            }
            if !t.scope.is_empty() && t.scope != existing.scope {
                set_field(
                    &mut fields,
                    &mut before,
                    "scope",
                    json!(existing.scope),
                    json!(t.scope),
                );
            }
            if !t.repo.is_empty() && t.repo != existing.repo {
                set_field(
                    &mut fields,
                    &mut before,
                    "repo",
                    json!(existing.repo),
                    json!(t.repo),
                );
            }
            if fields.is_empty() {
                continue;
            }
            let kind = if fields.get("state").and_then(Value::as_str) == Some(work::TASK_DROPPED) {
                work::OP_REMOVE
            } else {
                work::OP_UPDATE
            };
            self.ops.push(work::Op {
                entity: work::ENTITY_TASK.to_string(),
                kind: kind.to_string(),
                entity_id: existing.id.clone(),
                card_id: card.id.clone(),
                card_slug: card.slug.clone(),
                label: format!("T{}", existing.number),
                fields: fields.clone(),
                before,
                ..work::Op::default()
            });
            if let Some(Value::String(state)) = fields.get("state") {
                self.task_state.insert(existing.id.clone(), state.clone());
                // A task leaves its claim when the checkbox closes it.
                if state != work::TASK_OPEN {
                    self.release_claim(&existing.id);
                }
            }
        }
    }

    fn questions(&mut self, node: &CardNode, card: &work::Card) {
        let mut max_position = -1;
        for q in &card.questions {
            max_position = max_position.max(q.position);
        }
        let mut questions = card.questions.clone();
        for q in &node.questions {
            let key = work::normalize_text(&q.text);
            let existing = questions
                .iter()
                .find(|current| work::normalize_text(&current.text) == key)
                .cloned();
            let Some(existing) = existing else {
                if q.removed {
                    continue;
                }
                max_position += 1;
                let id = ids::new_ulid();
                let mut state = work::QUESTION_OPEN.to_string();
                if !q.answer.is_empty() {
                    state = work::QUESTION_ANSWERED.to_string();
                    self.answer_hint(q, &q.options);
                }
                questions.push(work::Question {
                    id: id.clone(),
                    card_id: card.id.clone(),
                    text: q.text.clone(),
                    blocking: q.blocking,
                    options: q.options.clone(),
                    state: state.clone(),
                    answer: q.answer.clone(),
                    position: max_position,
                });
                let mut fields = work::Fields::new();
                fields.insert("text".to_string(), json!(q.text));
                fields.insert("blocking".to_string(), json!(q.blocking));
                fields.insert("options".to_string(), json!(q.options));
                fields.insert("state".to_string(), json!(state));
                fields.insert("position".to_string(), json!(max_position));
                if !q.answer.is_empty() {
                    fields.insert("answer".to_string(), json!(q.answer));
                }
                self.ops.push(work::Op {
                    entity: work::ENTITY_QUESTION.to_string(),
                    kind: work::OP_CREATE.to_string(),
                    entity_id: id,
                    card_id: card.id.clone(),
                    card_slug: card.slug.clone(),
                    label: q.text.clone(),
                    fields,
                    ..work::Op::default()
                });
                continue;
            };
            let mut fields = work::Fields::new();
            let mut before = work::Fields::new();
            if q.removed {
                if existing.state != work::QUESTION_REMOVED {
                    set_field(
                        &mut fields,
                        &mut before,
                        "state",
                        json!(existing.state),
                        json!(work::QUESTION_REMOVED),
                    );
                }
            } else {
                if !q.answer.is_empty() {
                    // A line without an answer never reopens a decided question.
                    if existing.state != work::QUESTION_ANSWERED {
                        set_field(
                            &mut fields,
                            &mut before,
                            "state",
                            json!(existing.state),
                            json!(work::QUESTION_ANSWERED),
                        );
                    }
                    if q.answer != existing.answer {
                        set_field(
                            &mut fields,
                            &mut before,
                            "answer",
                            json!(existing.answer),
                            json!(q.answer),
                        );
                    }
                    if !fields.is_empty() {
                        let options = if q.options.is_empty() {
                            existing.options.clone()
                        } else {
                            q.options.clone()
                        };
                        self.answer_hint(q, &options);
                    }
                } else if existing.state == work::QUESTION_REMOVED {
                    set_field(
                        &mut fields,
                        &mut before,
                        "state",
                        json!(existing.state),
                        json!(work::QUESTION_OPEN),
                    );
                }
                if q.text != existing.text {
                    set_field(
                        &mut fields,
                        &mut before,
                        "text",
                        json!(existing.text),
                        json!(q.text),
                    );
                }
                if q.blocking != existing.blocking {
                    set_field(
                        &mut fields,
                        &mut before,
                        "blocking",
                        json!(existing.blocking),
                        json!(q.blocking),
                    );
                }
                if !q.options.is_empty() && q.options != existing.options {
                    set_field(
                        &mut fields,
                        &mut before,
                        "options",
                        json!(existing.options),
                        json!(q.options),
                    );
                }
            }
            if fields.is_empty() {
                continue;
            }
            let kind = if q.removed {
                work::OP_REMOVE
            } else {
                work::OP_UPDATE
            };
            self.ops.push(work::Op {
                entity: work::ENTITY_QUESTION.to_string(),
                kind: kind.to_string(),
                entity_id: existing.id.clone(),
                card_id: card.id.clone(),
                card_slug: card.slug.clone(),
                label: existing.text.clone(),
                fields,
                before,
                ..work::Op::default()
            });
        }
    }

    /// Flags an answer that is none of the question's options. Free answers are
    /// allowed; the hint catches a typo.
    fn answer_hint(&mut self, q: &QuestionNode, options: &[String]) {
        if options.is_empty() {
            return;
        }
        let answer = work::normalize_text(&q.answer);
        if options
            .iter()
            .any(|option| work::normalize_text(option) == answer)
        {
            return;
        }
        self.hintf(
            q.line,
            format!(
                "answer {} is not one of the options ({})",
                go_quote(&q.answer),
                options.join(" | ")
            ),
        );
    }

    fn invariants(&mut self, node: &CardNode, card: &work::Card) {
        let mut max_position = -1;
        for inv in &card.invariants {
            max_position = max_position.max(inv.position);
        }
        let mut invariants = card.invariants.clone();
        for inv in &node.invariants {
            let key = work::normalize_text(&inv.text);
            let existing = invariants
                .iter()
                .find(|current| work::normalize_text(&current.text) == key)
                .cloned();
            let Some(existing) = existing else {
                if inv.removed {
                    continue;
                }
                max_position += 1;
                let id = ids::new_ulid();
                invariants.push(work::Invariant {
                    id: id.clone(),
                    card_id: card.id.clone(),
                    text: inv.text.clone(),
                    state: work::INVARIANT_ACTIVE.to_string(),
                    position: max_position,
                });
                let mut fields = work::Fields::new();
                fields.insert("text".to_string(), json!(inv.text));
                fields.insert("state".to_string(), json!(work::INVARIANT_ACTIVE));
                fields.insert("position".to_string(), json!(max_position));
                self.ops.push(work::Op {
                    entity: work::ENTITY_INVARIANT.to_string(),
                    kind: work::OP_CREATE.to_string(),
                    entity_id: id,
                    card_id: card.id.clone(),
                    card_slug: card.slug.clone(),
                    label: inv.text.clone(),
                    fields,
                    ..work::Op::default()
                });
                continue;
            };
            let mut fields = work::Fields::new();
            let mut before = work::Fields::new();
            let want = if inv.removed {
                work::INVARIANT_REMOVED
            } else {
                work::INVARIANT_ACTIVE
            };
            if existing.state != want {
                set_field(
                    &mut fields,
                    &mut before,
                    "state",
                    json!(existing.state),
                    json!(want),
                );
            }
            if !inv.removed && inv.text != existing.text {
                set_field(
                    &mut fields,
                    &mut before,
                    "text",
                    json!(existing.text),
                    json!(inv.text),
                );
            }
            if fields.is_empty() {
                continue;
            }
            let kind = if inv.removed {
                work::OP_REMOVE
            } else {
                work::OP_UPDATE
            };
            self.ops.push(work::Op {
                entity: work::ENTITY_INVARIANT.to_string(),
                kind: kind.to_string(),
                entity_id: existing.id.clone(),
                card_id: card.id.clone(),
                card_slug: card.slug.clone(),
                label: existing.text.clone(),
                fields,
                before,
                ..work::Op::default()
            });
        }
    }
}

/// Adds `key` to the changed fields and its previous value to the plan output.
fn set_field(
    fields: &mut work::Fields,
    before: &mut work::Fields,
    key: &str,
    old: Value,
    value: Value,
) {
    fields.insert(key.to_string(), value);
    before.insert(key.to_string(), old);
}

/// A column map with one entry, for a single-field update.
fn one_field(key: &str, value: Value) -> work::Fields {
    let mut fields = work::Fields::new();
    fields.insert(key.to_string(), value);
    fields
}

/// A string as a plan field value.
fn s(value: &str) -> Value {
    Value::String(value.to_string())
}

/// A deployment list as a plan field value. An empty list stays `[]`.
fn deployments_json(deploys: &[structure::Deployment]) -> Value {
    if deploys.is_empty() {
        return json!([]);
    }
    serde_json::to_value(deploys).unwrap_or_else(|_| json!([]))
}

/// Describes a held claim for a refusal: `claimed by codex (api:src/*)`.
fn describe_claim(held: &HeldClaim) -> String {
    let scope = if held.claim.scope.is_empty() {
        "no scope"
    } else {
        held.claim.scope.as_str()
    };
    format!("claimed by {} ({})", held.claim.holder, scope)
}

/// Reports whether the stored handoff already says what the text says.
fn same_handoff(existing: &work::Handoff, node: &HandoffNode) -> bool {
    existing.branch == node.branch
        && existing.agent == node.agent
        && existing.goal == node.goal
        && existing.done == node.done
        && existing.in_progress == node.in_progress
        && existing.next == node.next
        && existing.touched == node.touched
}

/// A node's `repo:` or `path:` line, else its heading.
fn checkout_meta_line(node: &StructureNode) -> i32 {
    for entry in &node.meta {
        if entry.key == "repo" || entry.key == "path" {
            return entry.line;
        }
    }
    node.line
}

/// The line of the card's `key:` entry, or the card heading.
fn card_meta_line(node: &CardNode, key: &str) -> i32 {
    for entry in &node.meta {
        if entry.key == key {
            return entry.line;
        }
    }
    node.line
}

/// Renders ops terraform-style, one line each.
pub fn format_plan(ops: &[work::Op]) -> String {
    if ops.is_empty() {
        return "no changes\n".to_string();
    }
    let mut out = String::new();
    for op in ops {
        let sign = match op.kind.as_str() {
            work::OP_CREATE => "+",
            work::OP_REMOVE => "-",
            _ => "~",
        };
        out.push_str(&format!("{sign} {} {}", op.entity, op.card_slug));
        match op.entity.as_str() {
            work::ENTITY_CARD => {
                if op.kind == work::OP_CREATE {
                    out.push_str(&format!(
                        " {} [{}]",
                        go_quote(&op.label),
                        display(op.fields.get("stage"))
                    ));
                }
            }
            work::ENTITY_NODE => {
                if op.fields.contains_key("text") {
                    out.push_str(&format!(" {}", go_quote(&op.label)));
                } else if op.kind == work::OP_CREATE {
                    out.push_str(&format!(
                        " {} [{}]",
                        go_quote(&op.label),
                        display(op.fields.get("level"))
                    ));
                }
            }
            work::ENTITY_TASK => {
                out.push_str(&format!(" {}", op.label));
                if op.kind == work::OP_CREATE {
                    out.push_str(&format!(" {}", go_quote(&display(op.fields.get("title")))));
                }
            }
            work::ENTITY_CLAIM => {
                out.push_str(&format!(
                    " {} {}",
                    op.label,
                    display(op.fields.get("holder"))
                ));
                if let Some(scope) = op.fields.get("scope").and_then(Value::as_str)
                    && !scope.is_empty()
                {
                    out.push_str(&format!(" ({scope})"));
                }
                out.push('\n');
                continue;
            }
            work::ENTITY_CONFIRMATION => {
                out.push_str(&format!(" {}", op.label));
                out.push('\n');
                continue;
            }
            _ => {
                out.push_str(&format!(" {}", go_quote(&op.label)));
                if op.kind == work::OP_CREATE
                    && op.fields.get("blocking").and_then(Value::as_bool) == Some(true)
                {
                    out.push_str(" [blocking]");
                }
            }
        }
        if op.kind != work::OP_CREATE {
            let mut keys: Vec<&String> = op.fields.keys().collect();
            keys.sort();
            for key in keys {
                out.push_str(&format!(
                    " {}: {} → {}",
                    key,
                    short_value(op.before.get(key)),
                    short_value(op.fields.get(key))
                ));
            }
        }
        out.push('\n');
    }
    out
}

/// Formats a field value the way Go's `%v` does for the values a plan holds.
fn display(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "<nil>".to_string(),
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
    }
}

/// One plan output value: a list in brackets, long or spaced text quoted.
fn short_value(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "<nil>".to_string();
    };
    let text = match value {
        Value::Null => "<nil>".to_string(),
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Array(_) => deploy_list_short(value).unwrap_or_else(|| {
            let items: Vec<String> = value
                .as_array()
                .map(|items| items.iter().map(item_text).collect())
                .unwrap_or_default();
            format!("[{}]", items.join(", "))
        }),
        other => other.to_string(),
    };
    if text.is_empty() {
        return "\"\"".to_string();
    }
    let runes: Vec<char> = text.chars().collect();
    if runes.len() > 40 {
        return go_quote(&format!("{}...", runes[..37].iter().collect::<String>()));
    }
    if text.contains([' ', '\n']) {
        return go_quote(&text);
    }
    text
}

/// One element of a list field, as Go's `fmt.Sprint` writes it.
fn item_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// A deployment list as `[env=provider:resource@id, …]`, or `None` when the
/// array holds something else.
fn deploy_list_short(value: &Value) -> Option<String> {
    let items = value.as_array()?;
    if items.is_empty() {
        return None;
    }
    let mut parts = Vec::with_capacity(items.len());
    for item in items {
        let fields = item.as_object()?;
        let environment = match fields.get("environment") {
            Some(Value::String(name)) if !name.is_empty() => name.clone(),
            _ => fields.get("environment_id")?.as_str()?.to_string(),
        };
        let provider = fields.get("provider")?.as_str()?.to_string();
        let resource = fields.get("resource")?.as_str()?.to_string();
        let mut part = format!("{environment}={provider}:{resource}");
        if let Some(target) = fields.get("target_id").and_then(Value::as_str)
            && !target.is_empty()
        {
            part.push('@');
            part.push_str(target);
        }
        parts.push(part);
    }
    Some(format!("[{}]", parts.join(", ")))
}

/// Returns `5 ops (1 card, 4 tasks)`.
pub fn summarize(ops: &[work::Op]) -> String {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for op in ops {
        *counts.entry(op.entity.as_str()).or_default() += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for entity in [
        work::ENTITY_CARD,
        work::ENTITY_TASK,
        work::ENTITY_QUESTION,
        work::ENTITY_INVARIANT,
        work::ENTITY_HANDOFF,
        work::ENTITY_CLAIM,
        work::ENTITY_CONFIRMATION,
        work::ENTITY_NODE,
    ] {
        if let Some(&n) = counts.get(entity)
            && n > 0
        {
            let noun = if n > 1 {
                format!("{entity}s")
            } else {
                entity.to_string()
            };
            parts.push(format!("{n} {noun}"));
        }
    }
    let noun = if ops.len() == 1 { "op" } else { "ops" };
    format!("{} {} ({})", ops.len(), noun, parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn must_decode(text: &str) -> Document {
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode {text:?}: {diags:?}");
        doc
    }

    fn has_diag(diags: &[Diagnostic], line: i32, fragment: &str) -> bool {
        diags
            .iter()
            .any(|d| d.line == line && d.message.contains(fragment))
    }

    fn field_str(fields: &work::Fields, key: &str) -> String {
        match fields.get(key) {
            Some(Value::String(text)) => text.clone(),
            Some(Value::Null) | None => String::new(),
            Some(other) => other.to_string(),
        }
    }

    fn field_i32(fields: &work::Fields, key: &str) -> i32 {
        fields
            .get(key)
            .and_then(Value::as_i64)
            .map(|number| number as i32)
            .unwrap_or_default()
    }

    fn field_bool(fields: &work::Fields, key: &str) -> bool {
        fields.get(key).and_then(Value::as_bool).unwrap_or_default()
    }

    fn field_list(fields: &work::Fields, key: &str) -> Vec<String> {
        fields
            .get(key)
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Simulates the store applying create ops.
    fn snapshot_from_ops(ops: &[work::Op]) -> work::Snapshot {
        let mut snapshot = work::Snapshot::default();
        let mut index: HashMap<String, usize> = HashMap::new();
        for op in ops {
            let f = &op.fields;
            match op.entity.as_str() {
                work::ENTITY_CARD => {
                    index.insert(op.card_id.clone(), snapshot.cards.len());
                    snapshot.cards.push(work::Card {
                        id: op.card_id.clone(),
                        slug: field_str(f, "slug"),
                        title: field_str(f, "title"),
                        intent: field_str(f, "intent"),
                        stage: field_str(f, "stage"),
                        status: field_str(f, "status"),
                        owner: field_str(f, "owner"),
                        under: field_str(f, "under"),
                        decisions: field_list(f, "decisions"),
                        position: field_i32(f, "position"),
                        ..work::Card::default()
                    });
                }
                work::ENTITY_TASK => {
                    let card = &mut snapshot.cards[index[&op.card_id]];
                    card.tasks.push(work::Task {
                        id: op.entity_id.clone(),
                        card_id: op.card_id.clone(),
                        number: field_i32(f, "number"),
                        title: field_str(f, "title"),
                        state: field_str(f, "state"),
                        scope: field_str(f, "scope"),
                        position: field_i32(f, "position"),
                        ..work::Task::default()
                    });
                }
                work::ENTITY_QUESTION => {
                    let card = &mut snapshot.cards[index[&op.card_id]];
                    card.questions.push(work::Question {
                        id: op.entity_id.clone(),
                        card_id: op.card_id.clone(),
                        text: field_str(f, "text"),
                        blocking: field_bool(f, "blocking"),
                        options: field_list(f, "options"),
                        state: field_str(f, "state"),
                        position: field_i32(f, "position"),
                        ..work::Question::default()
                    });
                }
                work::ENTITY_INVARIANT => {
                    let card = &mut snapshot.cards[index[&op.card_id]];
                    card.invariants.push(work::Invariant {
                        id: op.entity_id.clone(),
                        card_id: op.card_id.clone(),
                        text: field_str(f, "text"),
                        state: field_str(f, "state"),
                        position: field_i32(f, "position"),
                    });
                }
                work::ENTITY_HANDOFF => {
                    let card = &mut snapshot.cards[index[&op.card_id]];
                    card.handoffs.push(work::Handoff {
                        id: op.entity_id.clone(),
                        card_id: op.card_id.clone(),
                        branch: field_str(f, "branch"),
                        agent: field_str(f, "agent"),
                        goal: field_str(f, "goal"),
                        done: field_str(f, "done"),
                        in_progress: field_str(f, "in_progress"),
                        next: field_str(f, "next"),
                        touched: field_list(f, "touched"),
                        ..work::Handoff::default()
                    });
                }
                _ => {}
            }
        }
        snapshot
    }

    const SAMPLE: &str = "---\npolter: 1\nsummary: Plan git import\nrationale: Slice 5 needs cards\n---\n\n# Roadmap\n\n## git-metadata-import: Git metadata import\nstage: plan · owner: erick · decisions: Decision-0007\n\nImport HEAD, branch and recent commits as evidence.\n\n- [ ] T1 Commit evidence upsert `backend/domains/canon/*`\n- [X] Stamp revision on observed facts\n? Include merge commits? [blocking] (yes | no | first-parent)\n! No network calls during scan\n";

    fn claim_snapshot() -> work::Snapshot {
        work::Snapshot {
            project_id: "p1".to_string(),
            cards: vec![
                work::Card {
                    id: "c1".to_string(),
                    slug: "git-import".to_string(),
                    title: "Git import".to_string(),
                    stage: work::STAGE_BUILD.to_string(),
                    status: work::CARD_ACTIVE.to_string(),
                    tasks: vec![
                        work::Task {
                            id: "t1".to_string(),
                            card_id: "c1".to_string(),
                            number: 1,
                            title: "Evidence".to_string(),
                            state: work::TASK_OPEN.to_string(),
                            scope: "backend/import/*".to_string(),
                            ..work::Task::default()
                        },
                        work::Task {
                            id: "t2".to_string(),
                            card_id: "c1".to_string(),
                            number: 2,
                            title: "Stamp".to_string(),
                            state: work::TASK_OPEN.to_string(),
                            scope: "backend/domains/canon/*".to_string(),
                            ..work::Task::default()
                        },
                        work::Task {
                            id: "t3".to_string(),
                            card_id: "c1".to_string(),
                            number: 3,
                            title: "Docs".to_string(),
                            state: work::TASK_OPEN.to_string(),
                            ..work::Task::default()
                        },
                        work::Task {
                            id: "t4".to_string(),
                            card_id: "c1".to_string(),
                            number: 4,
                            title: "Old".to_string(),
                            state: work::TASK_DONE.to_string(),
                            scope: "docs/*".to_string(),
                            ..work::Task::default()
                        },
                    ],
                    ..work::Card::default()
                },
                work::Card {
                    id: "c2".to_string(),
                    slug: "canon-core".to_string(),
                    title: "Canon core".to_string(),
                    stage: work::STAGE_BUILD.to_string(),
                    status: work::CARD_ACTIVE.to_string(),
                    tasks: vec![work::Task {
                        id: "u1".to_string(),
                        card_id: "c2".to_string(),
                        number: 1,
                        title: "Core".to_string(),
                        state: work::TASK_OPEN.to_string(),
                        scope: "backend/*".to_string(),
                        claim: Some(work::Claim {
                            id: "k1".to_string(),
                            project_id: "p1".to_string(),
                            task_id: "u1".to_string(),
                            holder: "claude".to_string(),
                            scope: "backend/*".to_string(),
                            ..work::Claim::default()
                        }),
                        ..work::Task::default()
                    }],
                    ..work::Card::default()
                },
            ],
            ..work::Snapshot::default()
        }
    }

    fn coord_doc(body: &str) -> Document {
        must_decode(&format!("---\npolter: 1\n---\n{body}"))
    }

    fn answer_snapshot() -> work::Snapshot {
        work::Snapshot {
            project_id: "p1".to_string(),
            cards: vec![work::Card {
                id: "c1".to_string(),
                slug: "git-import".to_string(),
                title: "Git import".to_string(),
                stage: work::STAGE_PLAN.to_string(),
                status: work::CARD_ACTIVE.to_string(),
                questions: vec![
                    work::Question {
                        id: "q1".to_string(),
                        card_id: "c1".to_string(),
                        text: "Include merge commits?".to_string(),
                        blocking: true,
                        options: vec![
                            "yes".to_string(),
                            "no".to_string(),
                            "first-parent".to_string(),
                        ],
                        state: work::QUESTION_OPEN.to_string(),
                        ..work::Question::default()
                    },
                    work::Question {
                        id: "q2".to_string(),
                        card_id: "c1".to_string(),
                        text: "How many commits back?".to_string(),
                        state: work::QUESTION_ANSWERED.to_string(),
                        answer: "20".to_string(),
                        ..work::Question::default()
                    },
                ],
                ..work::Card::default()
            }],
            ..work::Snapshot::default()
        }
    }

    #[test]
    fn plan_is_idempotent() {
        let doc = must_decode(SAMPLE);
        let (ops, diags) = plan(&doc, &work::Snapshot::default());
        assert!(diags.is_empty(), "diags={diags:?}");
        assert_eq!(ops.len(), 5, "ops = {}", format_plan(&ops));
        let snapshot = snapshot_from_ops(&ops);
        let (again, diags) = plan(&doc, &snapshot);
        assert!(diags.is_empty(), "diags={diags:?}");
        assert!(again.is_empty(), "re-apply: {}", format_plan(&again));
        // Encoding the stored card and planning it back is also a no-op.
        let (encoded, _) = decode(&encode_card(&snapshot.cards[0]));
        let (back, _) = plan(&encoded, &snapshot);
        assert!(back.is_empty(), "encode round trip: {}", format_plan(&back));
    }

    #[test]
    fn plan_partial_and_removal() {
        let doc = must_decode(SAMPLE);
        let (ops, _) = plan(&doc, &work::Snapshot::default());
        let snapshot = snapshot_from_ops(&ops);

        let update = "---\npolter: 1\n---\n## git-metadata-import\nstage: build\n- [x] T1 Commit evidence upsert\n- [ ] New follow-up\n! No network calls during scan [removed]\n";
        let parsed = must_decode(update);
        let (ops, diags) = plan(&parsed, &snapshot);
        assert!(diags.is_empty(), "plan diags = {diags:?}");
        let text = format_plan(&ops);
        for fragment in [
            "~ card git-metadata-import stage: plan → build",
            "~ task git-metadata-import T1 state: open → done",
            "+ task git-metadata-import T3 \"New follow-up\"",
            "- invariant git-metadata-import",
        ] {
            assert!(
                text.contains(fragment),
                "plan missing {fragment:?}:\n{text}"
            );
        }
        assert!(
            !text.contains("intent") && !text.contains("question"),
            "partial text must not touch unmentioned fields:\n{text}"
        );
    }

    #[test]
    fn plan_unknown_references() {
        let doc = must_decode(SAMPLE);
        let (ops, _) = plan(&doc, &work::Snapshot::default());
        let snapshot = snapshot_from_ops(&ops);

        let (_, diags) = plan(&must_decode("---\npolter: 1\n---\n## missing\n"), &snapshot);
        assert!(has_diag(&diags, 4, "unknown card"), "diags = {diags:?}");
        let (_, diags) = plan(
            &must_decode("---\npolter: 1\n---\n## git-metadata-import\n- [ ] T9 Ghost\n"),
            &snapshot,
        );
        assert!(has_diag(&diags, 5, "unknown task T9"), "diags = {diags:?}");
    }

    #[test]
    fn handoff_decode_plan_round_trip() {
        let text = "---\npolter: 1\nsummary: Handoff for git import\nrationale: Session handoff\n---\n\n## git-metadata-import: Git metadata import\nstage: build\n\nhandoff:\nbranch: feat/git-import\nagent: codex\ngoal: import HEAD as evidence\ndone: commit upsert\nin_progress: stamp revision\nnext: wire import\ntouched: backend/import.go, docs/spec.md\n\n- [ ] T1 Wire import\n? Include merge commits? [blocking]\n";
        let doc = must_decode(text);
        let handoff = doc.cards[0]
            .handoff
            .as_ref()
            .expect("the card has a handoff");
        assert_eq!(handoff.branch, "feat/git-import");
        assert_eq!(handoff.agent, "codex");
        assert_eq!(handoff.in_progress, "stamp revision");
        assert_eq!(handoff.touched.len(), 2, "touched = {:?}", handoff.touched);
        assert_eq!(handoff.touched[1], "docs/spec.md");
        let (ops, diags) = plan(&doc, &work::Snapshot::default());
        assert!(diags.is_empty(), "plan diags = {diags:?}");
        let handoffs = ops
            .iter()
            .filter(|op| op.entity == work::ENTITY_HANDOFF && op.kind == work::OP_CREATE)
            .count();
        assert_eq!(handoffs, 1, "ops = {}", format_plan(&ops));
        let snapshot = snapshot_from_ops(&ops);
        let (again, diags) = plan(&doc, &snapshot);
        assert!(diags.is_empty(), "diags={diags:?}");
        assert!(again.is_empty(), "re-apply: {}", format_plan(&again));
        let (encoded, diags) = decode(&encode_card(&snapshot.cards[0]));
        assert!(!has_errors(&diags), "encode diags = {diags:?}");
        let (back, _) = plan(&encoded, &snapshot);
        assert!(back.is_empty(), "encode round trip: {}", format_plan(&back));
    }

    #[test]
    fn confirm_lines_resolve_a_criterion_and_a_rule() {
        let mut snapshot = work::Snapshot {
            project_id: "p1".to_string(),
            cards: vec![work::Card {
                id: "c1".to_string(),
                slug: "git-import".to_string(),
                title: "Git import".to_string(),
                status: work::CARD_ACTIVE.to_string(),
                invariants: vec![work::Invariant {
                    id: "inv-1".to_string(),
                    card_id: "c1".to_string(),
                    text: "No network calls".to_string(),
                    state: work::INVARIANT_ACTIVE.to_string(),
                    ..work::Invariant::default()
                }],
                ..work::Card::default()
            }],
            ..work::Snapshot::default()
        };
        snapshot.rules = vec![work::Rule {
            id: "r1".to_string(),
            slug: "local-only".to_string(),
            statement: "Never call the network".to_string(),
            status: work::RULE_ACTIVE.to_string(),
            ..work::Rule::default()
        }];

        let (ops, diags) = plan_as(
            &coord_doc("## git-import\nconfirm: ! No network calls\nconfirm: rule local-only\n"),
            &snapshot,
            "codex",
        );
        assert!(!has_errors(&diags), "confirm: {diags:?}");
        assert_eq!(ops.len(), 2, "ops = {}", format_plan(&ops));
        assert!(
            ops.iter().all(|op| op.entity == work::ENTITY_CONFIRMATION
                && op.kind == work::OP_CREATE
                && op.card_id == "c1"),
            "ops = {ops:?}"
        );
        assert_eq!(
            field_str(&ops[0].fields, "kind"),
            work::CONFIRMATION_INVARIANT
        );
        assert_eq!(field_str(&ops[0].fields, "target_id"), "inv-1");
        assert_eq!(field_str(&ops[0].fields, "confirmed_by"), "codex");
        assert_eq!(field_str(&ops[1].fields, "kind"), work::CONFIRMATION_RULE);
        assert_eq!(field_str(&ops[1].fields, "target_id"), "r1");
        let text = format_plan(&ops);
        assert!(
            text.contains("+ confirmation git-import ! No network calls\n")
                && text.contains("+ confirmation git-import rule local-only\n"),
            "plan = {text}"
        );
        assert!(
            summarize(&ops).starts_with("2 ops (2 confirmations"),
            "{}",
            summarize(&ops)
        );

        // A person writes without an agent label; the confirmation names a
        // person instead.
        let (ops, diags) = plan_as(
            &coord_doc("## git-import\nconfirm: ! No network calls\n"),
            &snapshot,
            "",
        );
        assert!(!has_errors(&diags), "{diags:?}");
        assert_eq!(field_str(&ops[0].fields, "confirmed_by"), "person");

        // Unknown criterion, unknown Rule, and a malformed line all refuse.
        let (_, diags) = plan_as(
            &coord_doc("## git-import\nconfirm: ! Call the network\n"),
            &snapshot,
            "codex",
        );
        assert!(
            has_errors(&diags) && diags[0].message.contains("unknown acceptance criterion"),
            "{diags:?}"
        );
        let (_, diags) = plan_as(
            &coord_doc("## git-import\nconfirm: rule missing\n"),
            &snapshot,
            "codex",
        );
        assert!(
            has_errors(&diags) && diags[0].message.contains("unknown Rule"),
            "{diags:?}"
        );
        let (_, diags) = decode("---\npolter: 1\n---\n## git-import\nconfirm: everything\n");
        assert!(
            has_errors(&diags) && diags[0].message.contains("confirm needs"),
            "{diags:?}"
        );
        let (_, diags) = decode("---\npolter: 1\n---\nconfirm: rule local-only\n");
        assert!(
            has_errors(&diags) && diags[0].message.contains("confirm outside a card"),
            "{diags:?}"
        );
    }

    #[test]
    fn claim_plan() {
        let snapshot = claim_snapshot();

        let (_, diags) = plan_as(&coord_doc("## git-import\nclaim: T3\n"), &snapshot, "");
        assert!(
            has_errors(&diags) && diags[0].message.contains("needs an agent"),
            "claim without agent: {diags:?}"
        );

        let (ops, diags) = plan_as(&coord_doc("## git-import\nclaim: T3\n"), &snapshot, "codex");
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && ops[0].entity == work::ENTITY_CLAIM
                && ops[0].kind == work::OP_CREATE
                && field_str(&ops[0].fields, "holder") == "codex",
            "claim T3: ops={ops:?} diags={diags:?}"
        );

        // backend/import/* overlaps claude's backend/* on another card.
        let (_, diags) = plan_as(&coord_doc("## git-import\nclaim: T1\n"), &snapshot, "codex");
        assert!(
            has_errors(&diags)
                && diags[0]
                    .message
                    .contains("canon-core T1 held by claude (backend/*)"),
            "overlap: {diags:?}"
        );
        // The same holder may hold overlapping scopes.
        let (_, diags) = plan_as(
            &coord_doc("## git-import\nclaim: T1\n"),
            &snapshot,
            "claude",
        );
        assert!(!has_errors(&diags), "same holder overlap: {diags:?}");

        // Re-claim by the holder is a no-op; by another agent an error.
        let (ops, diags) = plan_as(
            &coord_doc("## canon-core\nclaim: T1\n"),
            &snapshot,
            "claude",
        );
        assert!(
            !has_errors(&diags) && ops.is_empty(),
            "re-claim: ops={ops:?} diags={diags:?}"
        );
        let (_, diags) = plan_as(&coord_doc("## canon-core\nclaim: T1\n"), &snapshot, "codex");
        assert!(
            has_errors(&diags) && diags[0].message.contains("claimed by claude"),
            "claim held task: {diags:?}"
        );

        // Done or unknown tasks cannot be claimed.
        let (_, diags) = plan_as(&coord_doc("## git-import\nclaim: T4\n"), &snapshot, "codex");
        assert!(
            has_errors(&diags) && diags[0].message.contains("it is done"),
            "claim done task: {diags:?}"
        );
        let (_, diags) = plan_as(&coord_doc("## git-import\nclaim: T9\n"), &snapshot, "codex");
        assert!(has_errors(&diags), "claim unknown task: {diags:?}");

        // Done by the holder marks the task done and deletes the claim.
        let (ops, diags) = plan_as(&coord_doc("## canon-core\ndone: T1\n"), &snapshot, "claude");
        assert!(
            !has_errors(&diags)
                && ops.len() == 2
                && field_str(&ops[0].fields, "state") == work::TASK_DONE
                && ops[1].entity == work::ENTITY_CLAIM
                && ops[1].kind == work::OP_REMOVE,
            "done: ops={ops:?} diags={diags:?}"
        );
        let (_, diags) = plan_as(&coord_doc("## canon-core\ndone: T1\n"), &snapshot, "codex");
        assert!(
            has_errors(&diags),
            "done on another agent's claim should fail"
        );
        // Done on a free task that is already done is a no-op.
        let (ops, diags) = plan_as(&coord_doc("## git-import\ndone: T4\n"), &snapshot, "codex");
        assert!(
            !has_errors(&diags) && ops.is_empty(),
            "done no-op: ops={ops:?} diags={diags:?}"
        );

        // Release: holder only; free task is a no-op.
        let (ops, diags) = plan_as(
            &coord_doc("## canon-core\nrelease: T1\n"),
            &snapshot,
            "claude",
        );
        assert!(
            !has_errors(&diags) && ops.len() == 1 && ops[0].kind == work::OP_REMOVE,
            "release: ops={ops:?} diags={diags:?}"
        );
        let (_, diags) = plan_as(
            &coord_doc("## canon-core\nrelease: T1\n"),
            &snapshot,
            "codex",
        );
        assert!(
            has_errors(&diags),
            "release another agent's claim should fail"
        );
        let (ops, diags) = plan_as(
            &coord_doc("## git-import\nrelease: T3\n"),
            &snapshot,
            "codex",
        );
        assert!(
            !has_errors(&diags) && ops.is_empty(),
            "release free: ops={ops:?} diags={diags:?}"
        );

        // A nested scope overlaps a wider claim on another card.
        let (_, diags) = plan_as(&coord_doc("## git-import\nclaim: T2\n"), &snapshot, "codex");
        assert!(
            has_errors(&diags),
            "backend/domains/canon/* overlaps backend/*: {diags:?}"
        );

        // A checkbox that closes a claimed task releases it.
        let (ops, diags) = plan_as(&coord_doc("## canon-core\n- [x] T1 Core\n"), &snapshot, "");
        assert!(
            !has_errors(&diags)
                && ops.len() == 2
                && ops[1].entity == work::ENTITY_CLAIM
                && ops[1].kind == work::OP_REMOVE,
            "checkbox done: ops={ops:?} diags={diags:?}"
        );
    }

    #[test]
    fn claim_status_lines() {
        let mut snapshot = claim_snapshot();
        snapshot.cards[0].tasks[2].claim = Some(work::Claim {
            holder: "codex".to_string(),
            ..work::Claim::default()
        });
        let status = encode_status_claims("git-import", &snapshot);
        assert_eq!(
            status,
            "claim: T3 codex\nclaimed: canon-core T1 claude (backend/*)\n"
        );
    }

    #[test]
    fn answer_plan() {
        let snapshot = answer_snapshot();

        let doc = must_decode(
            "---\npolter: 1\n---\n## git-import\n? Include merge commits? [blocking] -> first-parent\n",
        );
        let (ops, diags) = plan(&doc, &snapshot);
        assert!(
            !has_errors(&diags) && diags.is_empty() && ops.len() == 1,
            "answer: ops={ops:?} diags={diags:?}"
        );
        assert!(
            ops[0].kind == work::OP_UPDATE
                && field_str(&ops[0].fields, "state") == work::QUESTION_ANSWERED
                && field_str(&ops[0].fields, "answer") == "first-parent",
            "answer op = {:?}",
            ops[0]
        );
        assert!(
            !ops[0].fields.contains_key("blocking"),
            "repeating [blocking] must keep the flag: {:?}",
            ops[0].fields
        );

        // An answer outside the options is a hint, not an error.
        let doc = must_decode(
            "---\npolter: 1\n---\n## git-import\n? Include merge commits? [blocking] -> maybe\n",
        );
        let (ops, diags) = plan(&doc, &snapshot);
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && diags.len() == 1
                && diags[0].severity == SEVERITY_HINT
                && diags[0].message.contains("not one of the options"),
            "free answer: ops={ops:?} diags={diags:?}"
        );

        // The same answer again is a no-op; a bare line never reopens it.
        for line in ["? How many commits back? -> 20", "? How many commits back?"] {
            let doc = must_decode(&format!("---\npolter: 1\n---\n## git-import\n{line}\n"));
            let (ops, diags) = plan(&doc, &snapshot);
            assert!(
                !has_errors(&diags) && ops.is_empty(),
                "{line:?}: ops={ops:?} diags={diags:?}"
            );
        }

        // A different answer updates it.
        let doc =
            must_decode("---\npolter: 1\n---\n## git-import\n? How many commits back? -> 50\n");
        let (ops, _) = plan(&doc, &snapshot);
        assert!(
            ops.len() == 1
                && field_str(&ops[0].fields, "answer") == "50"
                && field_str(&ops[0].before, "answer") == "20",
            "re-answer: {ops:?}"
        );
        assert!(
            !ops[0].fields.contains_key("state"),
            "re-answer keeps the state: {:?}",
            ops[0].fields
        );

        // A new question written already answered is created answered.
        let doc = must_decode("---\npolter: 1\n---\n## git-import\n? Keep tags? -> no\n");
        let (ops, _) = plan(&doc, &snapshot);
        assert!(
            ops.len() == 1
                && ops[0].kind == work::OP_CREATE
                && field_str(&ops[0].fields, "state") == work::QUESTION_ANSWERED
                && field_str(&ops[0].fields, "answer") == "no",
            "new answered question: {ops:?}"
        );
    }

    #[test]
    fn answer_round_trip() {
        let snapshot = answer_snapshot();
        let text = encode_card(&snapshot.cards[0]);
        assert!(
            text.contains("? How many commits back? -> 20\n"),
            "encode should keep answered questions:\n{text}"
        );
        let (ops, diags) = plan(&must_decode(&text), &snapshot);
        assert!(
            !has_errors(&diags) && ops.is_empty(),
            "re-applying a read must be a no-op: ops={ops:?} diags={diags:?}"
        );
    }

    #[test]
    fn plan_refuses_a_checkout_on_a_domain() {
        let snapshot = work::Snapshot {
            project_id: "p1".to_string(),
            repos: vec!["api".to_string()],
            nodes: vec![structure::Node {
                id: "svc".to_string(),
                path: "svc".to_string(),
                level: structure::LEVEL_COMPONENT.to_string(),
                title: "Svc".to_string(),
                status: structure::NODE_ACTIVE.to_string(),
                repo: "api".to_string(),
                folder: "svc".to_string(),
                ..structure::Node::default()
            }],
            ..work::Snapshot::default()
        };
        for (text, want) in [
            (
                "# api: API\nlevel: domain · repo: api\n",
                "only components have a repo or path",
            ),
            (
                "# svc: Svc\nlevel: domain\n",
                "write repo: and path: to clear them",
            ),
        ] {
            let doc = must_decode(&format!("---\npolter: 1\n---\n{text}"));
            let (_, diags) = plan(&doc, &snapshot);
            assert!(
                has_errors(&diags) && diags[0].message.contains(want),
                "{text:?}: diags = {diags:?}"
            );
        }
        let doc = must_decode("---\npolter: 1\n---\n# svc: Svc\nlevel: domain · repo: · path:\n");
        let (_, diags) = plan(&doc, &snapshot);
        assert!(
            !has_errors(&diags),
            "a cleared checkout lets it become a domain: {diags:?}"
        );
    }

    #[test]
    fn plan_node_repo_and_folder() {
        let mut snapshot = work::Snapshot {
            project_id: "p1".to_string(),
            repos: vec!["api".to_string()],
            ..work::Snapshot::default()
        };
        let doc = must_decode(
            "---\npolter: 1\n---\n# api: API\nlevel: component · repo: api · path: services/api\n",
        );
        let (ops, diags) = plan(&doc, &snapshot);
        assert!(
            !has_errors(&diags) && ops.len() == 1,
            "create: ops={ops:?} diags={diags:?}"
        );
        let fields = &ops[0].fields;
        assert_eq!(field_str(fields, "path"), "api", "fields = {fields:?}");
        assert_eq!(field_str(fields, "repo"), "api", "fields = {fields:?}");
        assert_eq!(
            field_str(fields, "folder"),
            "services/api",
            "fields = {fields:?}"
        );

        snapshot.nodes = vec![structure::Node {
            id: "n1".to_string(),
            path: "api".to_string(),
            slug: "api".to_string(),
            title: "API".to_string(),
            level: structure::LEVEL_COMPONENT.to_string(),
            status: structure::NODE_ACTIVE.to_string(),
            repo: "api".to_string(),
            folder: "services/api".to_string(),
            ..structure::Node::default()
        }];
        // Absent keys leave the stored values; an empty repo: moves to the root.
        let (ops, diags) = plan(&must_decode("---\npolter: 1\n---\n# api: API\n"), &snapshot);
        assert!(
            !has_errors(&diags) && ops.is_empty(),
            "no-op: ops={ops:?} diags={diags:?}"
        );
        let (ops, diags) = plan(
            &must_decode("---\npolter: 1\n---\n# api: API\nrepo:\n"),
            &snapshot,
        );
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && field_str(&ops[0].fields, "repo").is_empty()
                && field_str(&ops[0].before, "repo") == "api",
            "clear repo: ops={ops:?} diags={diags:?}"
        );
        let (_, diags) = plan(
            &must_decode("---\npolter: 1\n---\n# api: API\nrepo: web\n"),
            &snapshot,
        );
        assert!(has_errors(&diags), "unknown repo should fail the plan");
    }

    #[test]
    fn claim_scopes_across_repos() {
        let snapshot = work::Snapshot {
            project_id: "p1".to_string(),
            repos: vec!["api".to_string(), "web".to_string()],
            cards: vec![
                work::Card {
                    id: "c1".to_string(),
                    slug: "login".to_string(),
                    title: "Login".to_string(),
                    stage: work::STAGE_BUILD.to_string(),
                    status: work::CARD_ACTIVE.to_string(),
                    tasks: vec![
                        work::Task {
                            id: "t1".to_string(),
                            card_id: "c1".to_string(),
                            number: 1,
                            title: "Callback".to_string(),
                            state: work::TASK_OPEN.to_string(),
                            repo: "api".to_string(),
                            scope: "src/auth/*".to_string(),
                            ..work::Task::default()
                        },
                        work::Task {
                            id: "t2".to_string(),
                            card_id: "c1".to_string(),
                            number: 2,
                            title: "Page".to_string(),
                            state: work::TASK_OPEN.to_string(),
                            repo: "web".to_string(),
                            scope: "src/auth/*".to_string(),
                            ..work::Task::default()
                        },
                        work::Task {
                            id: "t3".to_string(),
                            card_id: "c1".to_string(),
                            number: 3,
                            title: "Root".to_string(),
                            state: work::TASK_OPEN.to_string(),
                            scope: "src/auth/*".to_string(),
                            ..work::Task::default()
                        },
                    ],
                    ..work::Card::default()
                },
                work::Card {
                    id: "c2".to_string(),
                    slug: "session".to_string(),
                    title: "Session".to_string(),
                    stage: work::STAGE_BUILD.to_string(),
                    status: work::CARD_ACTIVE.to_string(),
                    tasks: vec![work::Task {
                        id: "u1".to_string(),
                        card_id: "c2".to_string(),
                        number: 1,
                        title: "Tokens".to_string(),
                        state: work::TASK_OPEN.to_string(),
                        repo: "api".to_string(),
                        scope: "src/**".to_string(),
                        claim: Some(work::Claim {
                            id: "k1".to_string(),
                            project_id: "p1".to_string(),
                            task_id: "u1".to_string(),
                            holder: "claude".to_string(),
                            scope: "api:src/**".to_string(),
                            ..work::Claim::default()
                        }),
                        ..work::Task::default()
                    }],
                    ..work::Card::default()
                },
            ],
            ..work::Snapshot::default()
        };

        // Same repository, overlapping globs: refused.
        let (_, diags) = plan_as(&coord_doc("## login\nclaim: T1\n"), &snapshot, "codex");
        assert!(
            has_errors(&diags)
                && diags[0].message.contains(
                    "scope api:src/auth/* overlaps session T1 held by claude (api:src/**)"
                ),
            "same repo: {diags:?}"
        );
        // Same glob in another checkout, or at the project root: allowed.
        let (ops, diags) = plan_as(&coord_doc("## login\nclaim: T2\n"), &snapshot, "codex");
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && field_str(&ops[0].fields, "scope") == "web:src/auth/*",
            "other repo: ops={ops:?} diags={diags:?}"
        );
        let (ops, diags) = plan_as(&coord_doc("## login\nclaim: T3\n"), &snapshot, "codex");
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && field_str(&ops[0].fields, "scope") == "src/auth/*",
            "root: ops={ops:?} diags={diags:?}"
        );
    }

    #[test]
    fn plan_move_node() {
        let snapshot = work::Snapshot {
            project_id: "p1".to_string(),
            nodes: vec![
                structure::Node {
                    id: "desktop".to_string(),
                    path: "desktop".to_string(),
                    level: structure::LEVEL_DOMAIN.to_string(),
                    title: "Desktop".to_string(),
                    status: structure::NODE_ACTIVE.to_string(),
                    ..structure::Node::default()
                },
                structure::Node {
                    id: "mobile".to_string(),
                    path: "mobile".to_string(),
                    level: structure::LEVEL_DOMAIN.to_string(),
                    title: "Mobile".to_string(),
                    status: structure::NODE_ACTIVE.to_string(),
                    ..structure::Node::default()
                },
                structure::Node {
                    id: "core".to_string(),
                    parent_id: "desktop".to_string(),
                    path: "desktop/core".to_string(),
                    level: structure::LEVEL_COMPONENT.to_string(),
                    title: "Core".to_string(),
                    status: structure::NODE_ACTIVE.to_string(),
                    ..structure::Node::default()
                },
            ],
            cards: vec![work::Card {
                id: "card-1".to_string(),
                slug: "import".to_string(),
                title: "Import".to_string(),
                under: "desktop/core".to_string(),
                status: work::CARD_ACTIVE.to_string(),
                ..work::Card::default()
            }],
            ..work::Snapshot::default()
        };
        let doc = must_decode(
            "---\npolter: 1\n---\n# desktop/core: Core\nlevel: component\nunder: mobile\n",
        );
        let (ops, diags) = plan(&doc, &snapshot);
        assert!(!has_errors(&diags), "plan: {diags:?}");
        let mut moved: HashMap<String, String> = HashMap::new();
        for op in &ops {
            if op.kind != work::OP_UPDATE {
                continue;
            }
            if op.fields.contains_key("path") {
                moved.insert(op.entity_id.clone(), field_str(&op.fields, "path"));
            }
            if op.fields.contains_key("under") {
                moved.insert(op.entity_id.clone(), field_str(&op.fields, "under"));
            }
            if op.fields.contains_key("parent_id")
                && op.entity_id == "core"
                && field_str(&op.fields, "parent_id") != "mobile"
            {
                panic!("parent = {}", field_str(&op.fields, "parent_id"));
            }
        }
        assert_eq!(
            moved.get("core").map(String::as_str),
            Some("mobile/core"),
            "moved = {moved:?} ops = {ops:?}"
        );
        assert_eq!(
            moved.get("card-1").map(String::as_str),
            Some("mobile/core"),
            "moved = {moved:?} ops = {ops:?}"
        );
    }

    #[test]
    fn plan_moves_component_to_the_project_root() {
        let snapshot = work::Snapshot {
            project_id: "p1".to_string(),
            nodes: vec![
                structure::Node {
                    id: "desktop".to_string(),
                    path: "desktop".to_string(),
                    level: structure::LEVEL_DOMAIN.to_string(),
                    title: "Desktop".to_string(),
                    status: structure::NODE_ACTIVE.to_string(),
                    ..structure::Node::default()
                },
                structure::Node {
                    id: "core".to_string(),
                    parent_id: "desktop".to_string(),
                    path: "desktop/core".to_string(),
                    level: structure::LEVEL_COMPONENT.to_string(),
                    title: "Core".to_string(),
                    status: structure::NODE_ACTIVE.to_string(),
                    ..structure::Node::default()
                },
            ],
            ..work::Snapshot::default()
        };
        let doc =
            must_decode("---\npolter: 1\n---\n# desktop/core: Core\nlevel: component\nunder:\n");
        let (ops, diags) = plan(&doc, &snapshot);
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && field_str(&ops[0].fields, "path") == "core"
                && field_str(&ops[0].fields, "parent_id").is_empty(),
            "ops = {ops:?} diags = {diags:?}"
        );
    }

    #[test]
    fn plan_refuses_system_and_nested_domains() {
        let snapshot = work::Snapshot {
            project_id: "p1".to_string(),
            nodes: vec![structure::Node {
                id: "core".to_string(),
                path: "core".to_string(),
                level: structure::LEVEL_DOMAIN.to_string(),
                title: "Core".to_string(),
                status: structure::NODE_ACTIVE.to_string(),
                ..structure::Node::default()
            }],
            ..work::Snapshot::default()
        };
        for (text, want) in [
            ("# app: App\nlevel: system\n", "level"),
            (
                "# core/sub: Sub\nlevel: domain\nunder: core\n",
                "sits under the project",
            ),
        ] {
            let (doc, mut diags) = decode(&format!("---\npolter: 1\n---\n{text}"));
            if !has_errors(&diags) {
                diags = plan(&doc, &snapshot).1;
            }
            assert!(
                has_errors(&diags) && diags[0].message.contains(want),
                "{text:?}: diags = {diags:?}"
            );
        }
    }

    #[test]
    fn plan_sample_output_matches_the_go_oracle() {
        let doc = must_decode(SAMPLE);
        let (ops, _) = plan(&doc, &work::Snapshot::default());
        assert_eq!(
            format_plan(&ops),
            "+ card git-metadata-import \"Git metadata import\" [plan]\n\
             + task git-metadata-import T1 \"Commit evidence upsert\"\n\
             + task git-metadata-import T2 \"Stamp revision on observed facts\"\n\
             + question git-metadata-import \"Include merge commits?\" [blocking]\n\
             + invariant git-metadata-import \"No network calls during scan\"\n"
        );
        assert_eq!(
            summarize(&ops),
            "5 ops (1 card, 2 tasks, 1 question, 1 invariant)"
        );
    }

    #[test]
    fn plan_update_output_matches_the_go_oracle() {
        let snapshot = work::Snapshot {
            project_id: "p1".to_string(),
            repos: vec!["api".to_string()],
            environments: vec![
                work::NamedEnvironment {
                    id: "env-local".to_string(),
                    name: "local".to_string(),
                },
                work::NamedEnvironment {
                    id: "env-prod".to_string(),
                    name: "production".to_string(),
                },
            ],
            nodes: vec![structure::Node {
                id: "n1".to_string(),
                path: "api".to_string(),
                slug: "api".to_string(),
                title: "API".to_string(),
                level: structure::LEVEL_COMPONENT.to_string(),
                status: structure::NODE_ACTIVE.to_string(),
                repo: "api".to_string(),
                folder: "services/api".to_string(),
                kind: "service".to_string(),
                tech: "Go".to_string(),
                deploys: vec![structure::Deployment {
                    id: "d1".to_string(),
                    environment_id: "env-local".to_string(),
                    environment: "local".to_string(),
                    provider: "docker".to_string(),
                    resource: "api".to_string(),
                    target_id: "abc".to_string(),
                    ..structure::Deployment::default()
                }],
                ..structure::Node::default()
            }],
            cards: vec![work::Card {
                id: "c1".to_string(),
                slug: "git-import".to_string(),
                title: "Git import".to_string(),
                stage: work::STAGE_PLAN.to_string(),
                status: work::CARD_ACTIVE.to_string(),
                questions: vec![work::Question {
                    id: "q1".to_string(),
                    card_id: "c1".to_string(),
                    text: "Include merge commits?".to_string(),
                    blocking: true,
                    options: vec!["yes".to_string(), "no".to_string()],
                    state: work::QUESTION_OPEN.to_string(),
                    ..work::Question::default()
                }],
                tasks: vec![work::Task {
                    id: "t1".to_string(),
                    card_id: "c1".to_string(),
                    number: 1,
                    title: "Evidence".to_string(),
                    state: work::TASK_OPEN.to_string(),
                    repo: "api".to_string(),
                    scope: "src/*".to_string(),
                    ..work::Task::default()
                }],
                ..work::Card::default()
            }],
            ..work::Snapshot::default()
        };
        for (text, want) in [
            (
                "---\npolter: 1\n---\n# api: API\ndeploys: production=vercel:web\n",
                "~ node api deploys: [local=docker:api@abc] → [production=vercel:web]\n",
            ),
            (
                "---\npolter: 1\n---\n# api: API\ndeploys:\n",
                "~ node api deploys: [local=docker:api@abc] → []\n",
            ),
            (
                "---\npolter: 1\n---\n# api: API\nkind: api · tech: Rust\n",
                "~ node api kind: service → api tech: Go → Rust\n",
            ),
            (
                "---\npolter: 1\n---\n## git-import\n? Include merge commits? (yes | no | first-parent)\n",
                "~ question git-import \"Include merge commits?\" blocking: true → false options: \"[yes, no]\" → \"[yes, no, first-parent]\"\n",
            ),
            (
                "---\npolter: 1\n---\n## git-import\n- [x] T1 Evidence\n",
                "~ task git-import T1 state: open → done\n",
            ),
            (
                "---\npolter: 1\n---\n## git-import\nstatus: removed\n",
                "- card git-import status: active → removed\n",
            ),
            (
                "---\npolter: 1\n---\n# api: API\nstatus: removed\n",
                "- node api status: active → removed\n",
            ),
        ] {
            let (ops, diags) = plan(&must_decode(text), &snapshot);
            assert!(!has_errors(&diags), "{text:?}: {diags:?}");
            assert_eq!(format_plan(&ops), want, "{text:?}");
        }
    }

    #[test]
    fn plan_output_and_summary() {
        let ops = vec![
            work::Op {
                entity: work::ENTITY_CARD.to_string(),
                kind: work::OP_CREATE.to_string(),
                card_slug: "git-import".to_string(),
                label: "Git import".to_string(),
                fields: one_field("stage", s("plan")),
                ..work::Op::default()
            },
            work::Op {
                entity: work::ENTITY_TASK.to_string(),
                kind: work::OP_CREATE.to_string(),
                card_slug: "git-import".to_string(),
                label: "T1".to_string(),
                fields: one_field("title", s("Commit evidence upsert")),
                ..work::Op::default()
            },
            work::Op {
                entity: work::ENTITY_CLAIM.to_string(),
                kind: work::OP_CREATE.to_string(),
                card_slug: "git-import".to_string(),
                label: "T2".to_string(),
                fields: {
                    let mut fields = one_field("holder", s("codex"));
                    fields.insert("scope".to_string(), s("api:src/*"));
                    fields
                },
                ..work::Op::default()
            },
            work::Op {
                entity: work::ENTITY_CARD.to_string(),
                kind: work::OP_UPDATE.to_string(),
                card_slug: "git-import".to_string(),
                label: "Git import".to_string(),
                fields: one_field("stage", s("build")),
                before: one_field("stage", s("plan")),
                ..work::Op::default()
            },
        ];
        assert_eq!(
            format_plan(&ops),
            "+ card git-import \"Git import\" [plan]\n\
             + task git-import T1 \"Commit evidence upsert\"\n\
             + claim git-import T2 codex (api:src/*)\n\
             ~ card git-import stage: plan → build\n"
        );
        assert_eq!(summarize(&ops), "4 ops (2 cards, 1 task, 1 claim)");
        assert_eq!(format_plan(&[]), "no changes\n");
        assert_eq!(summarize(&[]), "0 ops ()");
        assert_eq!(summarize(&ops[1..2]), "1 op (1 task)");
    }
}
