//! Starting an in-app Commander run through the Dispatch core.

use gpui::{AppContext as _, ClipboardItem, Context};
use polterware_ui::toast::ToastTone;

use super::AppState;
use crate::contracts::{
    HostRepository, ListHostCardsRequest, ListMilestonesRequest, ListWorkCardsRequest, Milestone,
    QueryPolterRequest, WorkCard,
};
use crate::dispatch::{
    self,
    contracts::{
        ConfigRequest, DispatchAgent, DispatchModel, DispatchTier, ListAgentsRequest,
        ListModelsRequest, ListTiersRequest, StartRunRequest,
    },
};

#[derive(Clone, Debug, Default)]
pub struct CommanderSources {
    pub tiers: Vec<DispatchTier>,
    pub agents: Vec<DispatchAgent>,
    pub models: Vec<DispatchModel>,
    pub auto: bool,
    pub cards: Vec<WorkCard>,
    pub milestones: Vec<Milestone>,
    pub repositories: Vec<HostRepository>,
}

#[derive(Clone, Debug, Default)]
pub struct CommanderSelection {
    pub tier: String,
    pub agent: String,
    pub model: String,
    pub card_slug: String,
    pub repo: String,
}

impl AppState {
    pub fn commander_source_context(&self) -> Option<(crate::client::Core, String)> {
        Some((self.ready_core()?, self.project_id()?.to_string()))
    }

    pub fn copy_commander_brief(&mut self, slug: String, cx: &mut Context<Self>) {
        let (Some(core), Some(project_id)) = (self.ready_core(), self.project_id()) else {
            self.toast("Could not copy the agent brief.", true);
            return;
        };
        let project_id = project_id.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.work().query_polter(&QueryPolterRequest {
                        project_id,
                        query: format!("brief {slug}"),
                        ..QueryPolterRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(response) if !response.text.trim().is_empty() => {
                        cx.write_to_clipboard(ClipboardItem::new_string(response.text));
                        state.push_toast("Agent brief copied.", ToastTone::Success);
                    }
                    Ok(_) => state.toast("The core returned no brief for this card.", true),
                    Err(error) => state.toast(error.to_string(), true),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn start_commander_run(
        &mut self,
        prompt: String,
        selection: CommanderSelection,
        cx: &mut Context<Self>,
    ) {
        let Some(project_id) = self.project_id().map(str::to_owned) else {
            return;
        };
        if prompt.trim().is_empty() {
            return;
        }
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let dispatch = dispatch::connect()
                        .ok_or_else(|| "Install Dispatch to run agents.".to_string())?;
                    dispatch
                        .dispatch()
                        .start_run(&StartRunRequest {
                            project_id,
                            prompt,
                            card_slug: selection.card_slug,
                            tier: selection.tier,
                            agent: selection.agent,
                            model: selection.model,
                            repo: selection.repo,
                            ..StartRunRequest::default()
                        })
                        .map_err(|error| error.to_string())
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(response) => {
                        state.push_toast("Agent run started.", ToastTone::Success);
                        if let Some(run) = response.run.filter(|run| !run.id.is_empty()) {
                            dispatch::open_in_dispatch(&dispatch::run_url(&run.id), cx);
                        }
                    }
                    Err(error) => state.toast(error, true),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

/// Reads the same project and Dispatch sources as Electron's run route.
pub fn load_commander_sources(core: crate::client::Core, project_id: String) -> CommanderSources {
    let cards = core
        .work()
        .list_work_cards(&ListWorkCardsRequest {
            project_id: project_id.clone(),
            ..Default::default()
        })
        .map(|r| r.cards)
        .unwrap_or_default();
    let milestones = core
        .work()
        .list_milestones(&ListMilestonesRequest {
            project_id: project_id.clone(),
        })
        .map(|r| r.milestones)
        .unwrap_or_default();
    let repositories = core
        .dispatch_host()
        .list_host_cards(&ListHostCardsRequest { project_id })
        .map(|r| r.repositories)
        .unwrap_or_default();
    let mut sources = CommanderSources {
        cards,
        milestones,
        repositories,
        ..Default::default()
    };
    if let Some(dispatch) = dispatch::connect() {
        let service = dispatch.dispatch();
        sources.tiers = service
            .list_tiers(&ListTiersRequest {})
            .map(|r| r.tiers)
            .unwrap_or_default();
        sources.agents = service
            .list_agents(&ListAgentsRequest {})
            .map(|r| r.agents)
            .unwrap_or_default();
        sources.models = service
            .list_models(&ListModelsRequest {})
            .map(|r| r.models)
            .unwrap_or_default();
        sources.auto = service
            .config(&ConfigRequest {
                action: "get".into(),
                key: "classifier".into(),
                ..Default::default()
            })
            .ok()
            .and_then(|r| serde_json::from_str::<String>(&r.json).ok())
            .is_some_and(|s| !s.is_empty());
    }
    sources
}

/// Electron offers Auto only when a classifier is configured.
pub fn default_tier(classifier: Option<&str>) -> &'static str {
    if classifier.is_some_and(|value| !value.is_empty()) {
        "auto"
    } else {
        "normal"
    }
}

#[cfg(test)]
mod tests {
    use super::default_tier;

    #[test]
    fn default_route_depends_on_the_classifier() {
        assert_eq!(default_tier(Some("")), "normal");
        assert_eq!(default_tier(None), "normal");
        assert_eq!(default_tier(Some("claude")), "auto");
    }
}
