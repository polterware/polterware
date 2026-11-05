//! The scope filter in a page's filters (`createPageFilters`' `scope`, `scopeFilterFields`,
//! `scopeTokensToClauses`, `clausesToScopeTokens`): the scope's levels lead the page's
//! FilterPopover as fields of their own section (Domain with the `Boxes` mark, Component with
//! `Blockchain01Icon`, and Environment with `Cloud` where the page allows it), each one pill
//! of picked values with no `is not`. Their pills are the shared selection: picking one on a
//! page sets it for every scoped page, and a page shows what another one picked when it next
//! draws. The page's own clauses match its rows; the scope's narrow what the pages read.

use gpui::{App, Entity};
use polterware_ui::Icon;
use polterware_ui::filter::{FilterOption, FilterSpec};
use polterware_ui::filter_popover::{FilterField, FilterPopover};

use crate::state::{AppState, Load, ScopeCatalog, ScopeKey, ScopeToken};

/// What a scope field's id starts with (`SCOPE_FIELD_PREFIX`).
const PREFIX: &str = "scope-";

fn field_id(key: ScopeKey) -> String {
    format!("{PREFIX}{}", key.id())
}

pub fn is_scope_field(id: &str) -> bool {
    id.starts_with(PREFIX)
}

/// `ScopeIcon`.
fn icon(key: ScopeKey) -> Icon {
    match key {
        ScopeKey::Domain => Icon::Boxes,
        ScopeKey::Component => Icon::Blockchain01Icon,
        ScopeKey::Env => Icon::Cloud,
    }
}

fn keys(allow_env: bool) -> impl Iterator<Item = ScopeKey> {
    ScopeKey::ALL
        .into_iter()
        .filter(move |key| allow_env || *key != ScopeKey::Env)
}

/// `scopeFilterFields`: the domains, the components, and, where the page allows it, the
/// project's environments.
pub fn scope_fields(catalog: &ScopeCatalog, allow_env: bool) -> Vec<FilterField> {
    keys(allow_env)
        .map(|key| {
            let options = match key {
                ScopeKey::Env => catalog
                    .environments
                    .iter()
                    .map(|environment| {
                        FilterOption::new(environment.id.clone(), environment.name.clone())
                    })
                    .collect(),
                level => catalog
                    .nodes
                    .iter()
                    .filter(|node| node.level == level.id())
                    .map(|node| FilterOption::new(node.id.clone(), node.title.clone()))
                    .collect(),
            };
            let mut spec = FilterSpec::multi(field_id(key), key.label(), options);
            spec.negatable = false;
            spec.repeatable = false;
            FilterField::new(spec, Some(icon(key))).section("scope")
        })
        .collect()
}

/// Sets the page's own fields, after the scope's the popover already shows.
pub fn set_page_fields(popover: &mut FilterPopover, own: Vec<FilterField>) {
    let scope: Vec<FilterField> = popover
        .fields()
        .iter()
        .filter(|field| is_scope_field(&field.spec.id))
        .cloned()
        .collect();
    popover.set_fields(scope.into_iter().chain(own).collect());
}

/// `clearPage`: drops the page's own clauses and keeps the shared scope's.
pub fn clear_page(popover: &Entity<FilterPopover>, cx: &mut App) {
    popover.update(cx, |popover, cx| {
        let mut own: Vec<String> = popover
            .filters()
            .clauses
            .iter()
            .filter(|clause| !is_scope_field(&clause.field))
            .map(|clause| clause.field.clone())
            .collect();
        own.dedup();
        for field in own {
            popover.set_filter(&field, &[], cx);
        }
    });
}

/// A page's link to the shared selection.
pub struct ScopeFilter {
    allow_env: bool,
    catalog_seen: Option<u64>,
    scope_seen: Option<u64>,
}

impl ScopeFilter {
    /// `allow_env`: the page offers Environment too (`scopeFilterAllowsEnv`).
    pub fn new(allow_env: bool) -> Self {
        Self {
            allow_env,
            catalog_seen: None,
            scope_seen: None,
        }
    }

    /// Before the page draws, after it set its own fields: puts the scope's fields first when
    /// they are missing or the catalog was read again, and its pills on the shared selection
    /// when that moved.
    pub fn sync(
        &mut self,
        popover: &Entity<FilterPopover>,
        state: &Entity<AppState>,
        cx: &mut App,
    ) {
        let app = state.read(cx);
        let (catalog_generation, scope_generation) =
            (app.catalog_generation(), app.scope_generation());
        let catalog = match app.catalog() {
            Load::Ready(catalog) => Some(catalog),
            _ => None,
        };
        let shown = popover
            .read(cx)
            .fields()
            .iter()
            .any(|field| is_scope_field(&field.spec.id));
        let fields =
            (self.catalog_seen != Some(catalog_generation) || (catalog.is_some() && !shown)).then(
                || catalog.map_or_else(Vec::new, |catalog| scope_fields(catalog, self.allow_env)),
            );
        let tokens = (self.scope_seen != Some(scope_generation)).then(|| app.scope().to_vec());
        self.catalog_seen = Some(catalog_generation);
        self.scope_seen = Some(scope_generation);
        if fields.is_none() && tokens.is_none() {
            return;
        }
        let allow_env = self.allow_env;
        popover.update(cx, |popover, cx| {
            if let Some(fields) = fields {
                let own: Vec<FilterField> = popover
                    .fields()
                    .iter()
                    .filter(|field| !is_scope_field(&field.spec.id))
                    .cloned()
                    .collect();
                popover.set_fields(fields.into_iter().chain(own).collect());
            }
            let Some(tokens) = tokens else {
                return;
            };
            for key in keys(allow_env) {
                let field = field_id(key);
                let picked: Vec<String> = tokens
                    .iter()
                    .filter(|token| token.key == key)
                    .map(|token| token.id.clone())
                    .collect();
                let pills: Vec<String> = popover
                    .filters()
                    .clauses
                    .iter()
                    .filter(|clause| clause.field == field)
                    .flat_map(|clause| clause.values.clone())
                    .collect();
                if pills != picked {
                    popover.set_filter(&field, &picked, cx);
                }
            }
        });
    }

    /// After the page's clauses changed: the shared selection takes its scope pills, keeping
    /// the environments a page without them cannot show.
    pub fn push(&self, popover: &Entity<FilterPopover>, state: &Entity<AppState>, cx: &mut App) {
        let app = state.read(cx);
        // A page that has not drawn the latest selection yet holds stale pills.
        if self.scope_seen != Some(app.scope_generation()) {
            return;
        }
        let mut tokens: Vec<ScopeToken> = popover
            .read(cx)
            .filters()
            .clauses
            .iter()
            .filter_map(|clause| {
                let key = ScopeKey::parse(clause.field.strip_prefix(PREFIX)?)?;
                Some(clause.values.iter().map(move |id| ScopeToken {
                    key,
                    id: id.clone(),
                }))
            })
            .flatten()
            .collect();
        if !self.allow_env {
            tokens.extend(
                app.scope()
                    .iter()
                    .filter(|token| token.key == ScopeKey::Env)
                    .cloned(),
            );
        }
        let mut next = tokens.clone();
        next.sort();
        next.dedup();
        let mut current = app.scope().to_vec();
        current.sort();
        if next != current {
            state.update(cx, |state, cx| state.set_scope(tokens, cx));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{Environment, StructureNode};

    #[test]
    fn the_scope_leads_with_its_levels_and_environments_only_where_allowed() {
        let node = |id: &str, level: &str, title: &str| StructureNode {
            id: id.into(),
            level: level.into(),
            title: title.into(),
            ..StructureNode::default()
        };
        let catalog = ScopeCatalog {
            nodes: vec![
                node("billing", "domain", "Billing"),
                node("invoices", "component", "Invoices"),
            ],
            environments: vec![Environment {
                id: "prod".into(),
                name: "Production".into(),
                ..Environment::default()
            }],
            milestones: Vec::new(),
        };
        let fields = scope_fields(&catalog, false);
        let ids: Vec<&str> = fields.iter().map(|field| field.spec.id.as_str()).collect();
        assert_eq!(ids, ["scope-domain", "scope-component"]);
        assert!(fields.iter().all(|field| {
            field.section.as_ref().map(|section| section.as_ref()) == Some("scope")
                && !field.spec.negatable
                && !field.spec.repeatable
        }));
        assert_eq!(fields[0].spec.options[0].label, "Billing");
        assert_eq!(fields[1].spec.options[0].id, "invoices");
        let with_env = scope_fields(&catalog, true);
        assert_eq!(with_env[2].spec.id, "scope-env");
        assert_eq!(with_env[2].spec.options[0].label, "Production");
    }
}
