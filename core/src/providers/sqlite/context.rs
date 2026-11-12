//! Port of `backend/providers/sqlite/context.go`.
//!
//! The Context Compiler's candidates: the entity asked for, its linked
//! knowledge and relationship hops, search hits, title matches, facts, and a
//! project sample when nothing is asked, narrowed by the scope and packed under
//! the token budget by `domains::context`.

use std::collections::HashMap;

use super::*;

use crate::domains::canon;
use crate::domains::context;
use crate::domains::knowledge;
use crate::domains::structure;

/// How many entities the project sample shows.
const CONTEXT_SAMPLE_ENTITY_LIMIT: usize = 8;
/// How many facts the project sample shows.
const CONTEXT_SAMPLE_FACT_LIMIT: usize = 8;

impl super::Store {
    pub fn compile_context(&self, req: &context::Request) -> Result<context::Package> {
        let mut candidates: Vec<context::Item> = Vec::with_capacity(32);

        if !req.entity_id.is_empty()
            && let Ok(entity) = self.get_canon_entity(&req.entity_id)
        {
            candidates.push(context::Item {
                kind: "entity".to_string(),
                id: entity.id.clone(),
                title: entity.title.clone(),
                excerpt: format!("{} · {}/{}", entity.kind, entity.ref_kind, entity.ref_id),
                truth_class: entity.truth_class.clone(),
                source: entity.source.clone(),
                path: canon::format_path(&entity),
                reasons: vec![context::REASON_EXACT_ENTITY.to_string()],
            });
            let links = self.list_knowledge_links(&req.project_id, "", &entity.id)?;
            for link in &links {
                let Ok(item) = self.get_active_knowledge(&link.knowledge_id) else {
                    continue;
                };
                candidates.push(context::Item {
                    kind: item.kind.clone(),
                    id: item.id.clone(),
                    title: item.title.clone(),
                    excerpt: context::excerpt(first_non_empty(&[&item.summary, &item.body]), 280),
                    truth_class: link.truth_class.clone(),
                    source: link.source.clone(),
                    path: format!("knowledge/{}/{}", item.kind, item.id),
                    reasons: vec![context::REASON_KNOWLEDGE_LINK.to_string()],
                });
            }
            let rels = self.list_canon_relationships(&req.project_id, &entity.id)?;
            for rel in &rels {
                let mut other_id = rel.object_entity_id.clone();
                if other_id == entity.id {
                    other_id = rel.subject_entity_id.clone();
                }
                let Ok(other) = self.get_canon_entity(&other_id) else {
                    continue;
                };
                candidates.push(context::Item {
                    kind: "entity".to_string(),
                    id: other.id.clone(),
                    title: other.title.clone(),
                    excerpt: format!("{} · {}", rel.predicate, other.kind),
                    truth_class: rel.truth_class.clone(),
                    source: rel.source.clone(),
                    path: canon::format_path(&other),
                    reasons: vec![context::REASON_RELATIONSHIP_HOP.to_string()],
                });
            }
            let rule_items =
                self.rule_context_items(&req.project_id, "", context::REASON_FACT_FOR_ENTITY, 0)?;
            candidates.extend(rule_items);
        }

        if !req.query.trim().is_empty() {
            let hits = self.search_knowledge(&req.project_id, &req.query, "", 12)?;
            for hit in &hits {
                candidates.push(context::Item {
                    kind: hit.item.kind.clone(),
                    id: hit.item.id.clone(),
                    title: hit.item.title.clone(),
                    excerpt: context::excerpt(
                        first_non_empty(&[&hit.snippet, &hit.item.summary, &hit.item.body]),
                        280,
                    ),
                    truth_class: hit.truth_class.clone(),
                    source: hit.source.clone(),
                    path: format!("knowledge/{}/{}", hit.item.kind, hit.item.id),
                    reasons: vec![context::REASON_FTS_HIT.to_string()],
                });
            }
            let entities = self.list_canon_entities(&req.project_id, "")?;
            let needle = req.query.trim().to_lowercase();
            for entity in &entities {
                if !entity.title.to_lowercase().contains(&needle)
                    && !entity.ref_id.to_lowercase().contains(&needle)
                {
                    continue;
                }
                candidates.push(context::Item {
                    kind: "entity".to_string(),
                    id: entity.id.clone(),
                    title: entity.title.clone(),
                    excerpt: format!("{} · {}", entity.kind, entity.ref_id),
                    truth_class: entity.truth_class.clone(),
                    source: entity.source.clone(),
                    path: canon::format_path(entity),
                    reasons: vec![context::REASON_TITLE_MATCH.to_string()],
                });
            }
            let rule_items = self.rule_context_items(
                &req.project_id,
                &req.query,
                context::REASON_FACT_QUERY_MATCH,
                0,
            )?;
            candidates.extend(rule_items);
        }

        if candidates.is_empty() && req.entity_id.is_empty() && req.query.trim().is_empty() {
            let entities = self.list_canon_entities(&req.project_id, "")?;
            for (i, entity) in entities.iter().enumerate() {
                if i >= CONTEXT_SAMPLE_ENTITY_LIMIT {
                    break;
                }
                candidates.push(context::Item {
                    kind: "entity".to_string(),
                    id: entity.id.clone(),
                    title: entity.title.clone(),
                    excerpt: format!("{} · {}", entity.kind, entity.ref_id),
                    truth_class: entity.truth_class.clone(),
                    source: entity.source.clone(),
                    path: canon::format_path(entity),
                    reasons: vec![context::REASON_PROJECT_SAMPLE.to_string()],
                });
            }
            let rule_items = self.rule_context_items(
                &req.project_id,
                "",
                context::REASON_PROJECT_SAMPLE,
                CONTEXT_SAMPLE_FACT_LIMIT,
            )?;
            candidates.extend(rule_items);
        }

        let candidates = self.scope_context_items(req, candidates)?;
        Ok(context::compile(req, &dedupe_context_items(candidates)))
    }

    /// Drops knowledge and fact candidates scoped outside the request's nodes
    /// (the union of each node). Entities and unscoped items stay.
    fn scope_context_items(
        &self,
        req: &context::Request,
        items: Vec<context::Item>,
    ) -> Result<Vec<context::Item>> {
        let ids = req.scope_ids();
        if ids.is_empty() {
            return Ok(items);
        }
        let resolved = self.load_scopes(
            &req.project_id,
            &structure::MultiScope {
                node_ids: ids,
                exact: req.scope_exact,
            },
        )?;
        let mut maps: HashMap<&str, HashMap<String, Vec<String>>> = HashMap::new();
        for kind in [SCOPE_KNOWLEDGE, SCOPE_RULE] {
            maps.insert(kind, self.scope_map(&req.project_id, kind)?);
        }
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let nodes: Vec<String> = {
                let kind = match item.kind.as_str() {
                    "rule" => Some(SCOPE_RULE),
                    knowledge::KIND_DOC => Some(SCOPE_KNOWLEDGE),
                    _ => None,
                };
                match kind.and_then(|kind| maps.get(kind)) {
                    Some(map) => map.get(&item.id).cloned().unwrap_or_default(),
                    None => Vec::new(),
                }
            };
            if resolved.matches_any(&nodes) {
                out.push(item);
            }
        }
        Ok(out)
    }

    fn rule_context_items(
        &self,
        project_id: &str,
        query: &str,
        reason: &str,
        limit: usize,
    ) -> Result<Vec<context::Item>> {
        let rules = self.load_rules(project_id)?;
        let needle = query.trim().to_lowercase();
        let mut out = Vec::with_capacity(rules.len());
        for rule in &rules {
            if !rule.binding() {
                continue;
            }
            if !needle.is_empty() && !rule.statement.to_lowercase().contains(&needle) {
                continue;
            }
            let item_reason = if reason.is_empty() {
                context::REASON_PROJECT_SAMPLE
            } else {
                reason
            };
            out.push(context::Item {
                kind: "rule".to_string(),
                id: rule.id.clone(),
                title: context::excerpt(&rule.statement, 120),
                excerpt: context::excerpt(&rule.statement, 280),
                truth_class: canon::TRUTH_DECLARED.to_string(),
                source: "rule".to_string(),
                path: format!("rule/{}", rule.handle()),
                reasons: vec![item_reason.to_string()],
            });
            if limit > 0 && out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    for value in values {
        if !value.trim().is_empty() {
            return value;
        }
    }
    ""
}

fn dedupe_context_items(items: Vec<context::Item>) -> Vec<context::Item> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out: Vec<context::Item> = Vec::with_capacity(items.len());
    for item in items {
        let key = format!("{}:{}", item.kind, item.id);
        if let Some(&idx) = seen.get(&key) {
            out[idx].reasons = context::merge_reasons(&out[idx].reasons, &item.reasons);
            continue;
        }
        seen.insert(key, out.len());
        out.push(item);
    }
    out
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use time::OffsetDateTime;

    use crate::domains::canon;
    use crate::domains::context;
    use crate::domains::knowledge;
    use crate::domains::projects;
    use crate::providers::sqlite::format_rfc3339;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn save_environment(store: &super::super::Store, id: &str, name: &str) {
        store
            .save_environment(projects::Environment {
                id: id.to_string(),
                project_id: "p".to_string(),
                name: name.to_string(),
                kind: "local".to_string(),
                ..projects::Environment::default()
            })
            .expect("environment");
    }

    fn item(id: &str, title: &str, body: &str, at: OffsetDateTime) -> knowledge::Item {
        knowledge::Item {
            id: id.to_string(),
            project_id: "p".to_string(),
            title: title.to_string(),
            body: body.to_string(),
            kind: knowledge::KIND_DOC.to_string(),
            doc_type: knowledge::DOC_TYPE_DOC.to_string(),
            created_at: at,
            updated_at: at,
            ..knowledge::Item::default()
        }
    }

    fn find<'a>(pkg: &'a context::Package, kind: &str, id: &str) -> Option<&'a context::Item> {
        pkg.items
            .iter()
            .find(|item| item.kind == kind && item.id == id)
    }

    #[test]
    fn compile_context_includes_rules_with_reasons() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        save_environment(store, "local", "Local");
        assert!(
            store.sync_project_model("p").expect("sync") > 0,
            "the project must project its environments"
        );
        seed_rule(
            store,
            "rule-1",
            "module uses Go modules for dependency management",
        );

        let sample = store
            .compile_context(&context::Request {
                project_id: "p".to_string(),
                token_budget: 2000,
                ..context::Request::default()
            })
            .expect("sample");
        assert_rule_item(&sample, "rule-1", context::REASON_PROJECT_SAMPLE);

        let matched = store
            .compile_context(&context::Request {
                project_id: "p".to_string(),
                query: "Go modules".to_string(),
                token_budget: 2000,
                ..context::Request::default()
            })
            .expect("matched");
        assert_rule_item(&matched, "rule-1", context::REASON_FACT_QUERY_MATCH);
    }

    fn seed_rule(store: &super::Store, id: &str, statement: &str) {
        let mut fields = serde_json::Map::new();
        fields.insert("statement".into(), serde_json::json!(statement));
        store
            .apply_work_ops(
                "p",
                "",
                "seed-rule",
                &[crate::domains::work::Op {
                    entity: crate::domains::work::ENTITY_RULE.to_string(),
                    kind: crate::domains::work::OP_CREATE.to_string(),
                    entity_id: id.to_string(),
                    card_slug: id.to_string(),
                    fields,
                    ..crate::domains::work::Op::default()
                }],
            )
            .expect("rule");
    }

    fn assert_rule_item(pkg: &context::Package, rule_id: &str, want_reason: &str) {
        let found = find(pkg, "rule", rule_id).unwrap_or_else(|| {
            panic!(
                "expected rule {rule_id} in package ({} items)",
                pkg.items.len()
            )
        });
        assert!(!found.reasons.is_empty(), "expected non-empty reasons");
        assert!(
            found.reasons.iter().any(|reason| reason == want_reason),
            "expected reason {want_reason:?}, got {:?}",
            found.reasons
        );
    }

    #[test]
    fn canon_knowledge_search_and_context() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        save_environment(store, "local", "Local");
        assert!(store.sync_project_model("p").expect("sync") > 0);
        store
            .save_knowledge(item(
                "doc-database",
                "Database",
                "The database runs Postgres",
                at,
            ))
            .expect("doc");
        let entity = store
            .list_canon_entities("p", canon::KIND_ENVIRONMENT)
            .expect("entities")
            .remove(0);
        let link = knowledge::new_entity_link(
            "p",
            "doc-database",
            &entity.id,
            canon::PRED_DOCUMENTS,
            "",
            "",
            "",
        )
        .expect("link");
        store.save_knowledge_link(&link).expect("save link");

        let hits = store
            .search_knowledge("p", "database", "", 10)
            .expect("hits");
        assert!(!hits.is_empty(), "expected fts hits");
        let links = store
            .list_knowledge_links("p", "doc-database", "")
            .expect("links");
        assert!(!links.is_empty(), "expected knowledge links");
        let pkg = store
            .compile_context(&context::Request {
                project_id: "p".to_string(),
                query: "database".to_string(),
                token_budget: 800,
                ..context::Request::default()
            })
            .expect("package");
        assert!(!pkg.items.is_empty(), "expected compiled context items");
        let doc = find(&pkg, knowledge::KIND_DOC, "doc-database").expect("doc item");
        assert!(
            doc.reasons
                .iter()
                .any(|reason| reason == context::REASON_FTS_HIT),
            "doc reasons = {:?}",
            doc.reasons
        );
    }

    #[test]
    fn compile_context_follows_an_entity() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        save_environment(store, "local", "Local");
        store.sync_project_model("p").expect("sync");
        let stamp = format_rfc3339(at);
        store
            .db
            .execute(
                "INSERT INTO structure_nodes(id,project_id,level,slug,path,title,created_at,updated_at) VALUES('n-api','p','component','api','app/api','Gateway API',?,?)",
                params![stamp, stamp],
            )
            .expect("node");
        let api = canon::new_entity(
            "p",
            canon::KIND_COMPONENT,
            canon::REF_ARCHITECTURE_NODE,
            "n-api",
            "Gateway API",
            canon::TRUTH_DECLARED,
            "structure",
            "system",
        )
        .expect("entity");
        let gateway = canon::new_entity(
            "p",
            canon::KIND_COMPONENT,
            canon::REF_ARCHITECTURE_NODE,
            "n-gw",
            "Gateway",
            canon::TRUTH_DECLARED,
            "structure",
            "system",
        )
        .expect("entity");
        store.upsert_canon_entity(&api).expect("api");
        store.upsert_canon_entity(&gateway).expect("gateway");
        store
            .save_knowledge(item("doc-api", "API notes", "Routes and handlers", at))
            .expect("doc");
        let link =
            knowledge::new_entity_link("p", "doc-api", &api.id, canon::PRED_DOCUMENTS, "", "", "")
                .expect("link");
        store.save_knowledge_link(&link).expect("link");
        let rel = canon::new_relationship(
            "p",
            &api.id,
            canon::PRED_DEPENDS_ON,
            &gateway.id,
            canon::TRUTH_DECLARED,
            "structure",
            "system",
        )
        .expect("relationship");
        store.upsert_canon_relationship(&rel).expect("rel");

        let pkg = store
            .compile_context(&context::Request {
                project_id: "p".to_string(),
                entity_id: api.id.clone(),
                token_budget: 4000,
                ..context::Request::default()
            })
            .expect("package");
        let exact = find(&pkg, "entity", &api.id).expect("exact entity");
        assert_eq!(exact.excerpt, "component · architecture_node/n-api");
        assert_eq!(exact.path, canon::format_path(&api));
        assert_eq!(
            exact.reasons,
            vec![context::REASON_EXACT_ENTITY.to_string()]
        );
        let linked = find(&pkg, knowledge::KIND_DOC, "doc-api").expect("linked doc");
        assert_eq!(
            linked.reasons,
            vec![context::REASON_KNOWLEDGE_LINK.to_string()]
        );
        assert_eq!(linked.path, "knowledge/doc/doc-api");
        let hop = find(&pkg, "entity", &gateway.id).expect("relationship hop");
        assert_eq!(
            hop.reasons,
            vec![context::REASON_RELATIONSHIP_HOP.to_string()]
        );
        assert_eq!(hop.excerpt, "DEPENDS_ON · component");
    }

    #[test]
    fn compile_context_reports_a_title_match() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        save_environment(store, "staging", "Gateway staging");
        store.sync_project_model("p").expect("sync");
        let pkg = store
            .compile_context(&context::Request {
                project_id: "p".to_string(),
                query: "gateway".to_string(),
                token_budget: 2000,
                ..context::Request::default()
            })
            .expect("package");
        let entity = pkg
            .items
            .iter()
            .find(|item| item.kind == "entity")
            .expect("title match");
        assert_eq!(
            entity.reasons,
            vec![context::REASON_TITLE_MATCH.to_string()]
        );
        assert_eq!(entity.excerpt, "environment · staging");
    }

    #[test]
    fn compile_context_narrows_to_the_scope() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        let stamp = format_rfc3339(at);
        for (id, parent, level, path) in [
            ("app", None, "domain", "app"),
            ("api", Some("app"), "component", "app/api"),
            ("web", None, "domain", "web"),
        ] {
            store
                .db
                .execute(
                    "INSERT INTO structure_nodes(id,project_id,parent_id,level,slug,path,title,created_at,updated_at) VALUES(?,'p',?,?,?,?,?,?,?)",
                    params![id, parent, level, id, path, id, stamp, stamp],
                )
                .expect("node");
        }
        store
            .save_knowledge(item("doc-api", "Database access", "Postgres", at))
            .expect("api doc");
        store
            .save_knowledge(item("doc-web", "Database guide", "Postgres", at))
            .expect("web doc");
        store
            .set_scope("p", SCOPE_KNOWLEDGE, "doc-api", &["api".to_string()])
            .expect("api scope");
        store
            .set_scope("p", SCOPE_KNOWLEDGE, "doc-web", &["web".to_string()])
            .expect("web scope");

        let pkg = store
            .compile_context(&context::Request {
                project_id: "p".to_string(),
                query: "database".to_string(),
                token_budget: 4000,
                scope_node_ids: vec!["app".to_string()],
                ..context::Request::default()
            })
            .expect("package");
        assert!(
            find(&pkg, knowledge::KIND_DOC, "doc-api").is_some(),
            "a doc under the scoped node stays: {:?}",
            pkg.items
        );
        assert!(
            find(&pkg, knowledge::KIND_DOC, "doc-web").is_none(),
            "a doc outside the scope goes: {:?}",
            pkg.items
        );

        // An unscoped item belongs to the whole project and stays.
        store
            .db
            .execute(
                "DELETE FROM entity_contexts WHERE project_id='p' AND entity_id='doc-api'",
                [],
            )
            .expect("unscope");
        let pkg = store
            .compile_context(&context::Request {
                project_id: "p".to_string(),
                query: "database".to_string(),
                token_budget: 4000,
                scope_node_id: "app".to_string(),
                ..context::Request::default()
            })
            .expect("package");
        assert!(
            find(&pkg, knowledge::KIND_DOC, "doc-api").is_some(),
            "the legacy scope field works the same"
        );
    }

    #[test]
    fn compile_context_dedupes_keeping_every_reason() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        save_environment(store, "local", "Local");
        store.sync_project_model("p").expect("sync");
        let entity = store
            .list_canon_entities("p", canon::KIND_ENVIRONMENT)
            .expect("entities")
            .remove(0);
        let pkg = store
            .compile_context(&context::Request {
                project_id: "p".to_string(),
                entity_id: entity.id.clone(),
                token_budget: 4000,
                ..context::Request::default()
            })
            .expect("package");
        assert_eq!(
            find(&pkg, "entity", &entity.id).expect("entity").reasons,
            vec![context::REASON_EXACT_ENTITY.to_string()]
        );
    }

    #[test]
    fn dedupe_merges_reasons_and_orders_stay() {
        let item = |kind: &str, id: &str, reason: &str| context::Item {
            kind: kind.to_string(),
            id: id.to_string(),
            reasons: vec![reason.to_string()],
            ..context::Item::default()
        };
        let out = dedupe_context_items(vec![
            item("doc", "d1", context::REASON_FTS_HIT),
            item("doc", "d2", context::REASON_FTS_HIT),
            item("doc", "d1", context::REASON_KNOWLEDGE_LINK),
        ]);
        assert_eq!(out.len(), 2, "out = {out:?}");
        assert_eq!(out[0].id, "d1");
        assert_eq!(
            out[0].reasons,
            vec![
                context::REASON_FTS_HIT.to_string(),
                context::REASON_KNOWLEDGE_LINK.to_string()
            ]
        );
    }

    #[test]
    fn first_non_empty_skips_blanks() {
        assert_eq!(first_non_empty(&["", "  ", "body"]), "body");
        assert_eq!(first_non_empty(&["", ""]), "");
    }
}
