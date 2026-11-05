//! Commander run-route choices, following Electron's `run-route.tsx` and `startRunCards`.
use crate::state::{CommanderSelection, CommanderSources};
use gpui::IntoElement;
use polterware_ui::bots::agent_avatar;
use polterware_ui::{Icon, Property, PropertyOption};

#[derive(Default)]
pub struct Route {
    pub sources: CommanderSources,
    pub tier: String,
    agent: String,
    pub route_touched: bool,
    milestone: String,
    pub card: String,
    repo: String,
}
impl Route {
    pub fn new() -> Self {
        Self {
            tier: "normal".into(),
            ..Default::default()
        }
    }
    pub fn cards(&self) -> Vec<(String, String, String, String)> {
        let mut milestones: Vec<_> = self.sources.milestones.iter().collect();
        milestones.sort_by_key(|m| match m.state.as_str() {
            "active" => 0,
            "planned" => 1,
            _ => 2,
        });
        let mut cards: Vec<_> = self
            .sources
            .cards
            .iter()
            .filter(|c| c.stage != "delivered")
            .collect();
        let index = |c: &crate::contracts::WorkCard| {
            milestones
                .iter()
                .position(|m| m.id == c.milestone_id)
                .unwrap_or(milestones.len())
        };
        cards.sort_by_key(|c| {
            (
                crate::work::Stage::parse(&c.stage).map_or(usize::MAX, |s| s.index()),
                index(c),
            )
        });
        cards
            .into_iter()
            .map(|c| {
                (
                    c.slug.clone(),
                    if c.title.is_empty() {
                        c.slug.clone()
                    } else {
                        c.title.clone()
                    },
                    milestones
                        .get(index(c))
                        .map_or("Backlog", |m| m.name.as_str())
                        .to_string(),
                    crate::work::Stage::parse(&c.stage)
                        .map_or(c.stage.as_str(), |s| s.label())
                        .to_string(),
                )
            })
            .collect()
    }
    pub fn clear_project(&mut self) {
        self.sources.cards.clear();
        self.sources.milestones.clear();
        self.sources.repositories.clear();
        self.card.clear();
        self.milestone.clear();
    }
    pub fn preset(&mut self, slug: &str) {
        self.card = slug.to_string();
        self.milestone = self
            .cards()
            .into_iter()
            .find(|c| c.0 == slug)
            .map_or(String::new(), |c| c.2);
    }
    pub fn selection(&self) -> CommanderSelection {
        let (agent, model) = self.agent.split_once('\0').unwrap_or((&self.agent, ""));
        CommanderSelection {
            tier: if !agent.is_empty() && self.tier == "auto" {
                String::new()
            } else {
                self.tier.clone()
            },
            agent: agent.into(),
            model: model.into(),
            card_slug: self.card.trim().into(),
            repo: self.repo.clone(),
        }
    }
    pub fn change(&mut self, id: &str, value: &str) {
        if id == "route" {
            self.route_touched = true;
        }
        match id {
            "route" if value.starts_with("tier:") => {
                self.tier = value[5..].into();
                self.agent.clear();
            }
            "route" => self.agent = value.into(),
            "milestone" => {
                self.milestone = value.into();
                if !self
                    .cards()
                    .iter()
                    .any(|c| c.0 == self.card && c.2 == value)
                {
                    self.card.clear();
                }
            }
            "card" if value.is_empty() => self.card.clear(),
            "card" => self.preset(value),
            "repository" => self.repo = value.into(),
            _ => {}
        }
    }
    fn route_options(&self) -> Vec<PropertyOption> {
        let mut route = Vec::new();
        if self.sources.auto {
            route.push(
                PropertyOption::new("tier:auto", "Auto")
                    .icon(Icon::Gauge)
                    .group("By tier")
                    .hint("The classifier judges the task, then the cheapest model likely to pass"),
            );
        }
        let mut names: Vec<String> = Vec::new();
        for t in &self.sources.tiers {
            if !names.contains(&t.name) {
                names.push(t.name.clone());
            }
        }
        if names.is_empty() {
            names = ["easy", "normal", "hard"]
                .into_iter()
                .map(String::from)
                .collect();
        }
        for name in names {
            let mut entries: Vec<_> = self
                .sources
                .tiers
                .iter()
                .filter(|t| t.name == name)
                .collect();
            entries.sort_by_key(|t| t.position);
            let hint = entries.first().map_or("The tier table picks".into(), |t| {
                let extra = entries.len() - 1;
                format!(
                    "{} / {}{}",
                    t.agent,
                    t.model,
                    if extra > 0 {
                        format!(
                            ", then {extra} fallback{}",
                            if extra == 1 { "" } else { "s" }
                        )
                    } else {
                        String::new()
                    }
                )
            });
            let mut chars = name.chars();
            let label = chars.next().map_or(String::new(), |c| {
                c.to_uppercase().collect::<String>() + chars.as_str()
            });
            route.push(
                PropertyOption::new(format!("tier:{name}"), label)
                    .icon(Icon::Gauge)
                    .group("By tier")
                    .hint(hint),
            );
        }
        for model in &self.sources.models {
            let agent = model.agent.clone();
            let mark = agent.clone();
            let hint = [&model.provider, &model.tier]
                .into_iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(" / ");
            route.push(
                PropertyOption::new(format!("{}\0{}", agent, model.model), model.model.clone())
                    .group(agent)
                    .hint(hint)
                    .mark(move |size, _, _| agent_avatar(&mark, "", size).into_any_element()),
            );
        }
        for agent in &self.sources.agents {
            if !self.sources.models.iter().any(|m| m.agent == agent.name) {
                let mark = agent.name.clone();
                route.push(
                    PropertyOption::new(agent.name.clone(), agent.name.clone())
                        .group(agent.name.clone())
                        .hint("Its default model")
                        .mark(move |size, _, _| agent_avatar(&mark, "", size).into_any_element()),
                );
            }
        }
        route
    }

    pub fn properties(&self) -> Vec<Property> {
        let route = self.route_options();
        let cards = self.cards();
        let mut milestones =
            vec![PropertyOption::new("", "Any milestone").icon(Icon::Briefcase06Icon)];
        let mut seen = Vec::new();
        for card in &cards {
            if !seen.contains(&card.2) {
                let count = cards.iter().filter(|c| c.2 == card.2).count();
                milestones.push(
                    PropertyOption::new(card.2.clone(), card.2.clone())
                        .icon(Icon::Briefcase06Icon)
                        .hint(format!(
                            "{count} open card{}",
                            if count == 1 { "" } else { "s" }
                        )),
                );
                seen.push(card.2.clone());
            }
        }
        let mut properties = vec![
            Property::single(
                "route",
                "Route",
                route,
                if self.agent.is_empty() {
                    format!("tier:{}", self.tier)
                } else {
                    self.agent.clone()
                },
            )
            .icon(Icon::Bot)
            .picker_width(340.0),
            Property::single("milestone", "Milestone", milestones, self.milestone.clone())
                .icon(Icon::Briefcase06Icon)
                .picker_width(280.0),
        ];
        if !self.milestone.is_empty() {
            let mut options = vec![PropertyOption::new("", "No card").icon(Icon::FolderKanban)];
            options.extend(
                cards
                    .into_iter()
                    .filter(|c| c.2 == self.milestone)
                    .map(|c| {
                        PropertyOption::new(c.0.clone(), c.1)
                            .hint(c.0)
                            .group(c.3)
                            .icon(Icon::FolderKanban)
                    }),
            );
            properties.push(
                Property::single("card", "Card", options, self.card.clone())
                    .icon(Icon::FolderKanban)
                    .picker_width(320.0),
            );
        }
        if self.sources.repositories.len() >= 2 {
            let mut options = vec![
                PropertyOption::new("", "Card's repository")
                    .hint("The card's component, or ask")
                    .icon(Icon::GitBranch),
            ];
            options.extend(self.sources.repositories.iter().map(|r| {
                let label = r.remote.strip_prefix("github.com/").unwrap_or(&r.name);
                let opt = PropertyOption::new(r.name.clone(), label.to_string()).icon(
                    if r.remote.is_empty() {
                        Icon::GitBranch
                    } else {
                        Icon::Github
                    },
                );
                if r.remote.is_empty() {
                    opt
                } else {
                    opt.hint(r.name.clone())
                }
            }));
            properties.push(
                Property::single("repository", "Repository", options, self.repo.clone())
                    .icon(Icon::GitBranch)
                    .picker_width(280.0),
            );
        }
        properties
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{HostRepository, Milestone, WorkCard};
    use crate::dispatch::contracts::{DispatchAgent, DispatchModel};

    #[test]
    fn cards_follow_stage_then_milestone_and_open_card_is_preset() {
        let mut route = Route::new();
        route.sources.milestones = vec![
            Milestone {
                id: "p".into(),
                name: "Planned".into(),
                state: "planned".into(),
                ..Default::default()
            },
            Milestone {
                id: "a".into(),
                name: "Active".into(),
                state: "active".into(),
                ..Default::default()
            },
        ];
        route.sources.cards = [
            ("planned", "build", "p"),
            ("active", "build", "a"),
            ("backlog", "shape", ""),
            ("done", "delivered", "a"),
        ]
        .into_iter()
        .map(|(slug, stage, milestone_id)| WorkCard {
            slug: slug.into(),
            stage: stage.into(),
            milestone_id: milestone_id.into(),
            ..Default::default()
        })
        .collect();
        assert_eq!(
            route
                .cards()
                .iter()
                .map(|c| c.0.as_str())
                .collect::<Vec<_>>(),
            ["backlog", "active", "planned"]
        );
        route.preset("active");
        assert_eq!(route.milestone, "Active");
        assert_eq!(route.selection().card_slug, "active");
        route.change("card", "");
        assert_eq!(route.milestone, "Active");
        route.preset("active");
        route.change("milestone", "Planned");
        assert!(route.card.is_empty());
        assert_eq!(
            route
                .cards()
                .iter()
                .filter(|c| c.2 == route.milestone)
                .map(|c| c.0.as_str())
                .collect::<Vec<_>>(),
            ["planned"]
        );
    }

    #[test]
    fn defaults_and_picks_produce_dispatch_selection() {
        let mut route = Route::new();
        assert_eq!(route.selection().tier, "normal");
        assert_eq!(
            route
                .route_options()
                .iter()
                .map(|o| o.value().as_ref())
                .collect::<Vec<_>>(),
            ["tier:easy", "tier:normal", "tier:hard"]
        );
        assert_eq!(
            route
                .properties()
                .iter()
                .map(|p| p.id().as_ref())
                .collect::<Vec<_>>(),
            ["route", "milestone"]
        );
        route.sources.auto = true;
        route.tier = "auto".into();
        route.sources.models.push(DispatchModel {
            agent: "codex".into(),
            model: "model".into(),
            ..Default::default()
        });
        route.sources.agents.push(DispatchAgent {
            name: "other".into(),
            ..Default::default()
        });
        route.sources.repositories = vec![HostRepository::default(), HostRepository::default()];
        assert_eq!(
            route
                .route_options()
                .iter()
                .map(|o| o.value().as_ref())
                .collect::<Vec<_>>(),
            [
                "tier:auto",
                "tier:easy",
                "tier:normal",
                "tier:hard",
                "codex\0model",
                "other"
            ]
        );
        assert_eq!(route.properties().last().unwrap().id(), "repository");
        route.change("route", "codex\0model");
        route.change("repository", "repo");
        let selection = route.selection();
        assert_eq!(
            (
                selection.tier.as_str(),
                selection.agent.as_str(),
                selection.model.as_str(),
                selection.repo.as_str()
            ),
            ("", "codex", "model", "repo")
        );
        route.change("route", "tier:hard");
        assert_eq!(route.selection().tier, "hard");
        assert!(route.selection().agent.is_empty());
    }
}
