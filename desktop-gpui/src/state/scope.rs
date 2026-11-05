//! The scope filter's state (`scopeTokens`, `applyScopeTokens`, and `setActiveScope` in
//! `App.tsx` and `workspace/load.ts`): one selection of domains, components, and environments
//! that every scoped page's filters share, cleared when another project opens, and the catalog
//! they pick from (the project's Structure nodes and environments, and its milestones, which
//! the shared Composer offers).
//!
//! A domain or component token reads under that node and every node below it: the Inbox, the
//! changes Activity lists, Knowledge's records and Rules, and Milestones' cards are read with
//! those `scope_nodes` again whenever the selection changes. An environment token narrows
//! nothing the pages read (only Structure would); Activity and Docs show it.

use gpui::{AppContext as _, Context};

use super::{AppState, Load, Section};
use crate::client::{Core, CoreError};
use crate::contracts::{
    Environment, ListEnvironmentsRequest, ListMilestonesRequest, ListStructureRequest, Milestone,
    StructureNode,
};

/// A scope level (`ScopeFilterKey`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScopeKey {
    Domain,
    Component,
    Env,
}

impl ScopeKey {
    pub const ALL: [ScopeKey; 3] = [ScopeKey::Domain, ScopeKey::Component, ScopeKey::Env];

    pub fn id(self) -> &'static str {
        match self {
            ScopeKey::Domain => "domain",
            ScopeKey::Component => "component",
            ScopeKey::Env => "env",
        }
    }

    /// `SCOPE_FILTER_MENU_LABELS`.
    pub fn label(self) -> &'static str {
        match self {
            ScopeKey::Domain => "Domain",
            ScopeKey::Component => "Component",
            ScopeKey::Env => "Environment",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|key| key.id() == id)
    }
}

/// One picked node or environment (`ScopeFilterToken`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeToken {
    pub key: ScopeKey,
    pub id: String,
}

/// What the scope picks from, read once per project.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScopeCatalog {
    pub nodes: Vec<StructureNode>,
    pub environments: Vec<Environment>,
    pub milestones: Vec<Milestone>,
}

/// `dedupeScopeTokens`: the first of each key and id, in order.
pub fn dedupe(tokens: Vec<ScopeToken>) -> Vec<ScopeToken> {
    let mut out: Vec<ScopeToken> = Vec::new();
    for token in tokens {
        if !token.id.is_empty() && !out.contains(&token) {
            out.push(token);
        }
    }
    out
}

/// `scopeNodeIds`: each picked domain or component the catalog knows, then the nodes under
/// it, each once.
pub fn node_ids(tokens: &[ScopeToken], nodes: &[StructureNode]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    fn walk(id: &str, nodes: &[StructureNode], out: &mut Vec<String>) {
        if out.iter().any(|seen| seen == id) || !nodes.iter().any(|node| node.id == id) {
            return;
        }
        out.push(id.to_string());
        for child in nodes.iter().filter(|node| node.parent_id == id) {
            walk(&child.id, nodes, out);
        }
    }
    for token in tokens.iter().filter(|token| token.key != ScopeKey::Env) {
        walk(&token.id, nodes, &mut out);
    }
    out
}

impl AppState {
    /// The shared selection, in the order it was picked.
    pub fn scope(&self) -> &[ScopeToken] {
        &self.scope
    }

    /// Bumped by each change of the selection, so a page's filters follow it.
    pub fn scope_generation(&self) -> u64 {
        self.scope_generation
    }

    pub fn catalog(&self) -> &Load<ScopeCatalog> {
        &self.catalog
    }

    /// Bumped by each catalog read, so a page's scope fields follow it.
    pub fn catalog_generation(&self) -> u64 {
        self.catalog_generation
    }

    /// The node ids the scoped reads send (`scopeArgs`); none reads the whole project.
    pub fn scope_node_ids(&self) -> Vec<String> {
        match &self.catalog {
            Load::Ready(catalog) => node_ids(&self.scope, &catalog.nodes),
            _ => Vec::new(),
        }
    }

    /// The picked domains and components, which a new decision or doc starts with as its
    /// Context (`defaultContexts`).
    pub fn scope_contexts(&self) -> Vec<String> {
        self.scope
            .iter()
            .filter(|token| token.key != ScopeKey::Env)
            .map(|token| token.id.clone())
            .collect()
    }

    /// `applyScopeTokens`: takes `tokens` as the selection and reads the scoped pages again;
    /// a page not open reads again when it next opens.
    pub fn set_scope(&mut self, tokens: Vec<ScopeToken>, cx: &mut Context<Self>) {
        let tokens = dedupe(tokens);
        if tokens == self.scope {
            return;
        }
        let nodes_before = self.scope_node_ids();
        self.scope = tokens;
        self.scope_generation += 1;
        cx.notify();
        if self.scope_node_ids() == nodes_before {
            // Only an environment moved: nothing the pages read changes.
            return;
        }
        self.scope_reads += 1;
        self.load_inbox(cx);
        let section = self.section();
        if section == Some(Section::Activity) {
            self.load_activity(cx);
        } else {
            self.activity = Load::Idle;
        }
        if section == Some(Section::Knowledge) {
            self.load_knowledge(cx);
        } else {
            self.knowledge = Load::Idle;
        }
        if section == Some(Section::Milestones) {
            self.load_work(cx);
        } else {
            self.work = Load::Idle;
        }
    }

    /// Forgets the selection, as opening another project does.
    pub(super) fn clear_scope(&mut self) {
        if !self.scope.is_empty() {
            self.scope.clear();
            self.scope_generation += 1;
        }
        self.catalog = Load::Idle;
        self.catalog_generation += 1;
    }

    /// Reads the open project's Structure nodes, environments, and milestones.
    pub fn load_catalog(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        if !matches!(self.catalog, Load::Ready(_)) {
            self.catalog = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move { read_catalog(&core, &project) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.project_id() != Some(wanted.as_str()) {
                    return;
                }
                let catalog = match result {
                    Ok(catalog) => Load::Ready(catalog),
                    // The scope then offers nothing to pick; the pages read the whole project.
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                if catalog != state.catalog {
                    state.catalog = catalog;
                    state.catalog_generation += 1;
                    cx.notify();
                }
            });
        })
        .detach();
    }
}

fn read_catalog(core: &Core, project: &str) -> Result<ScopeCatalog, CoreError> {
    let nodes = core
        .work()
        .list_structure(&ListStructureRequest {
            project_id: project.to_string(),
            ..ListStructureRequest::default()
        })?
        .nodes;
    let environments = core
        .project()
        .list_environments(&ListEnvironmentsRequest {
            project_id: project.to_string(),
        })?
        .environments;
    let milestones = core
        .work()
        .list_milestones(&ListMilestonesRequest {
            project_id: project.to_string(),
        })?
        .milestones;
    Ok(ScopeCatalog {
        nodes,
        environments,
        milestones,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, parent: &str, level: &str) -> StructureNode {
        StructureNode {
            id: id.into(),
            parent_id: parent.into(),
            level: level.into(),
            ..StructureNode::default()
        }
    }

    fn token(key: ScopeKey, id: &str) -> ScopeToken {
        ScopeToken { key, id: id.into() }
    }

    #[test]
    fn a_domain_reads_under_itself_and_its_components() {
        let nodes = vec![
            node("billing", "", "domain"),
            node("invoices", "billing", "component"),
            node("payouts", "billing", "component"),
            node("identity", "", "domain"),
        ];
        let tokens = vec![
            token(ScopeKey::Domain, "billing"),
            token(ScopeKey::Component, "invoices"),
            token(ScopeKey::Env, "prod"),
            token(ScopeKey::Domain, "gone"),
        ];
        assert_eq!(
            node_ids(&tokens, &nodes),
            vec!["billing", "invoices", "payouts"]
        );
        assert!(node_ids(&[token(ScopeKey::Env, "prod")], &nodes).is_empty());
    }

    #[test]
    fn a_repeated_pick_counts_once() {
        let tokens = dedupe(vec![
            token(ScopeKey::Domain, "billing"),
            token(ScopeKey::Domain, "billing"),
            token(ScopeKey::Component, "billing"),
            token(ScopeKey::Domain, ""),
        ]);
        assert_eq!(
            tokens,
            vec![
                token(ScopeKey::Domain, "billing"),
                token(ScopeKey::Component, "billing")
            ]
        );
    }
}
