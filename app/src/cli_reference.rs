//! The `polterware` CLI's manual: the usage, the commands by group with their options and an
//! example, and the keys of the interactive status view. `cli_reference.json` is its source,
//! first taken from the Electron Desktop's `cli-reference.ts`; edit it when the CLI changes.

use std::sync::OnceLock;

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Flag {
    pub name: String,
    pub value: Option<String>,
    pub required: bool,
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CliCommand {
    pub id: String,
    pub name: String,
    pub synopsis: String,
    pub summary: String,
    pub flags: Vec<Flag>,
    pub example: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Group {
    pub id: String,
    pub label: String,
    pub commands: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Key {
    pub keys: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Reference {
    pub usage: String,
    pub usage_flags: Vec<Flag>,
    pub groups: Vec<Group>,
    pub commands: Vec<CliCommand>,
    pub keys: Vec<Key>,
}

/// The usage section's prose.
pub const INTRO: &str = "The polterware CLI reads and writes the same local database as the Desktop, through the local core. Commands that list --json in their options print JSON for scripts and agents.";

pub fn reference() -> &'static Reference {
    static REFERENCE: OnceLock<Reference> = OnceLock::new();
    REFERENCE.get_or_init(|| {
        serde_json::from_str(include_str!("cli_reference.json"))
            .expect("cli_reference.json is the manual")
    })
}

impl Reference {
    /// A group's commands, in the group's order.
    pub fn grouped(&self, group: &Group) -> Vec<&CliCommand> {
        group
            .commands
            .iter()
            .flat_map(|id| {
                self.commands
                    .iter()
                    .filter(move |command| command.id == *id)
            })
            .collect()
    }

    /// `cliReferenceMarkdown`: the whole manual as Markdown, for agents, docs, or issues.
    pub fn markdown(&self) -> String {
        let mut lines: Vec<String> = vec![
            "# polterware CLI reference".into(),
            String::new(),
            INTRO.into(),
            String::new(),
            "## Usage".into(),
            String::new(),
            "```sh".into(),
            self.usage.clone(),
            "```".into(),
            String::new(),
        ];
        lines.extend(flag_table(&self.usage_flags, false));
        lines.extend([String::new(), "## Commands".into(), String::new()]);
        for group in &self.groups {
            let commands = self.grouped(group);
            lines.extend([format!("### {}", group.label), String::new()]);
            lines.extend([
                "| Command | Description |".to_string(),
                "| --- | --- |".to_string(),
            ]);
            lines.extend(
                commands
                    .iter()
                    .map(|command| format!("| `{}` | {} |", command.name, command.summary)),
            );
            lines.push(String::new());
            for command in commands {
                lines.extend([
                    format!("#### {}", command.name),
                    String::new(),
                    command.summary.clone(),
                    String::new(),
                    "```sh".into(),
                    command.synopsis.clone(),
                    "```".into(),
                ]);
                if !command.flags.is_empty() {
                    lines.extend([String::new(), "Options:".into(), String::new()]);
                    lines.extend(flag_table(&command.flags, true));
                }
                lines.extend([
                    String::new(),
                    "Example:".into(),
                    String::new(),
                    "```sh".into(),
                    command.example.clone(),
                    "```".into(),
                    String::new(),
                ]);
            }
        }
        lines.extend([
            "## Interactive keys".into(),
            String::new(),
            "| Key | Action |".into(),
            "| --- | --- |".into(),
        ]);
        lines.extend(
            self.keys
                .iter()
                .map(|key| format!("| `{}` | {} |", key.keys, key.description)),
        );
        format!("{}\n", lines.join("\n").trim_end())
    }
}

fn flag_table(flags: &[Flag], with_required: bool) -> Vec<String> {
    let mut lines = if with_required {
        vec![
            "| Flag | Value | Required | Description |".to_string(),
            "| --- | --- | --- | --- |".to_string(),
        ]
    } else {
        vec![
            "| Flag | Value | Description |".to_string(),
            "| --- | --- | --- |".to_string(),
        ]
    };
    for flag in flags {
        let mut cells = vec![
            format!("`{}`", flag.name),
            flag.value
                .as_ref()
                .map(|value| format!("`<{value}>`"))
                .unwrap_or_default(),
        ];
        if with_required {
            cells.push(if flag.required { "Yes" } else { "" }.into());
        }
        cells.push(flag.description.clone());
        lines.push(format!("| {} |", cells.join(" | ")));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_grouped_command_is_in_the_manual() {
        let reference = reference();
        let grouped: usize = reference
            .groups
            .iter()
            .map(|group| reference.grouped(group).len())
            .sum();
        assert_eq!(grouped, reference.commands.len());
        assert!(
            reference
                .markdown()
                .starts_with("# polterware CLI reference\n")
        );
    }
}
