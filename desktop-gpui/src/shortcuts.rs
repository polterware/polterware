//! The shortcuts a person can remap (`shortcuts/catalog.ts`) and their chords
//! (`shortcuts/chord.ts`): a key with Command, Option, and Shift, stored as `serializeChord`
//! writes it (`mod+shift+k`), so the remaps in `shortcut-preferences.json` mean the same in
//! both Desktops.

use std::collections::BTreeMap;

use gpui::Keystroke;

/// One key with its modifiers; `mod` is Command (Control off macOS).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    pub key: &'static str,
    pub mod_: bool,
    pub shift: bool,
    pub alt: bool,
}

/// A recorded chord, whose key is any text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedChord {
    pub key: String,
    pub mod_: bool,
    pub shift: bool,
    pub alt: bool,
}

const fn key(key: &'static str) -> Chord {
    Chord {
        key,
        mod_: false,
        shift: false,
        alt: false,
    }
}

const fn cmd(key: &'static str) -> Chord {
    Chord {
        key,
        mod_: true,
        shift: false,
        alt: false,
    }
}

const fn cmd_shift(key: &'static str) -> Chord {
    Chord {
        key,
        mod_: true,
        shift: true,
        alt: false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Shell,
    Inbox,
    Work,
    Structure,
}

impl Group {
    /// `SHORTCUT_GROUPS`, in the order Settings lists them.
    pub const ALL: [Group; 4] = [Group::Shell, Group::Inbox, Group::Work, Group::Structure];

    /// `SHORTCUT_GROUP_LABELS`.
    pub fn label(self) -> &'static str {
        match self {
            Group::Shell => "Shell",
            Group::Inbox => "Inbox",
            Group::Work => "Milestones",
            Group::Structure => "Structure",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub id: &'static str,
    pub label: &'static str,
    pub group: Group,
    pub default: Chord,
    pub aliases: &'static [Chord],
}

/// `SHORTCUT_CATALOG`.
pub const CATALOG: [Command; 13] = [
    Command {
        id: "shell.toggle_commander",
        label: "Toggle Commander",
        group: Group::Shell,
        default: cmd("k"),
        aliases: &[],
    },
    Command {
        id: "shell.toggle_commander_global",
        label: "Toggle Commander (global)",
        group: Group::Shell,
        default: cmd_shift("k"),
        aliases: &[],
    },
    Command {
        id: "shell.toggle_sidebar",
        label: "Toggle sidebar",
        group: Group::Shell,
        default: key("["),
        aliases: &[],
    },
    Command {
        id: "shell.toggle_detail",
        label: "Close details",
        group: Group::Shell,
        default: cmd("j"),
        aliases: &[],
    },
    Command {
        id: "shell.show_shortcuts",
        label: "Keyboard shortcuts",
        group: Group::Shell,
        default: cmd("/"),
        aliases: &[Chord {
            key: "?",
            mod_: false,
            shift: true,
            alt: false,
        }],
    },
    Command {
        id: "shell.open_canvas",
        label: "Open Canvas",
        group: Group::Shell,
        default: cmd_shift("c"),
        aliases: &[],
    },
    Command {
        id: "shell.open_filter",
        label: "Add filter",
        group: Group::Shell,
        default: key("f"),
        aliases: &[],
    },
    Command {
        id: "shell.create",
        label: "Create",
        group: Group::Shell,
        default: key("c"),
        aliases: &[],
    },
    Command {
        id: "inbox.defer",
        label: "Defer the item",
        group: Group::Inbox,
        default: key("h"),
        aliases: &[],
    },
    Command {
        id: "inbox.dismiss",
        label: "Dismiss the item",
        group: Group::Inbox,
        default: key("d"),
        aliases: &[],
    },
    Command {
        id: "work.toggle_layout",
        label: "Switch between list and board",
        group: Group::Work,
        default: cmd("b"),
        aliases: &[],
    },
    Command {
        id: "work.set_milestone",
        label: "Move to a milestone",
        group: Group::Work,
        default: key("m"),
        aliases: &[],
    },
    Command {
        id: "structure.add_node",
        label: "Add structure node",
        group: Group::Structure,
        default: key("n"),
        aliases: &[],
    },
];

/// The navigation sequences in `shortcuts/catalog.ts`, formatted as `formatSequence`.
pub fn sequence_hint(id: &str) -> Option<String> {
    let key = match id {
        "go.inbox" => "I",
        "go.activity" => "A",
        "go.milestones" => "M",
        "go.canvas" => "C",
        "go.structure" => "S",
        "go.truth" => "T",
        "go.projects" => "H",
        _ => return None,
    };
    Some(format!("G then {key}"))
}

pub fn command(id: &str) -> Option<&'static Command> {
    CATALOG.iter().find(|command| command.id == id)
}

pub fn in_group(group: Group) -> impl Iterator<Item = &'static Command> {
    CATALOG.iter().filter(move |command| command.group == group)
}

impl From<Chord> for OwnedChord {
    fn from(chord: Chord) -> Self {
        Self {
            key: chord.key.to_string(),
            mod_: chord.mod_,
            shift: chord.shift,
            alt: chord.alt,
        }
    }
}

/// Keys that are not one character (`parseChord`'s `allowedSpecial`).
const SPECIAL: [&str; 12] = [
    "space",
    "enter",
    "delete",
    "backspace",
    "tab",
    "arrowup",
    "arrowdown",
    "arrowleft",
    "arrowright",
    "=",
    "+",
    "-",
];

impl OwnedChord {
    /// `serializeChord`: `mod+alt+shift+key`.
    pub fn serialize(&self) -> String {
        let mut parts = Vec::new();
        if self.mod_ {
            parts.push("mod");
        }
        if self.alt {
            parts.push("alt");
        }
        if self.shift {
            parts.push("shift");
        }
        parts.push(&self.key);
        parts.join("+")
    }

    /// `parseChord`: modifiers under any of their names, then one key.
    pub fn parse(raw: &str) -> Option<Self> {
        let lower = raw.trim().to_lowercase();
        let mut chord = OwnedChord {
            key: String::new(),
            mod_: false,
            shift: false,
            alt: false,
        };
        for token in lower
            .split('+')
            .map(str::trim)
            .filter(|token| !token.is_empty())
        {
            match token {
                "mod" | "cmd" | "command" | "meta" | "ctrl" | "control" | "commandorcontrol" => {
                    chord.mod_ = true
                }
                "shift" => chord.shift = true,
                "alt" | "option" => chord.alt = true,
                _ => {
                    if !chord.key.is_empty() {
                        return None;
                    }
                    chord.key = match token {
                        "return" => "enter",
                        "del" => "delete",
                        "esc" => "escape",
                        other => other,
                    }
                    .to_string();
                }
            }
        }
        let one_char = chord.key.chars().count() == 1;
        (!chord.key.is_empty()
            && chord.key != "escape"
            && (one_char || SPECIAL.contains(&chord.key.as_str())))
        .then_some(chord)
    }

    /// `chordFromEvent`: the chord a key press makes, none for a modifier alone or Escape.
    pub fn from_keystroke(keystroke: &Keystroke) -> Option<Self> {
        let modifiers = keystroke.modifiers;
        let key = match keystroke.key.as_str() {
            "" | "escape" | "shift" | "control" | "alt" | "platform" | "function" | "fn" => {
                return None;
            }
            "up" => "arrowup".to_string(),
            "down" => "arrowdown".to_string(),
            "left" => "arrowleft".to_string(),
            "right" => "arrowright".to_string(),
            // Shift and / is the "?" key (`defaultAliases` of Keyboard shortcuts).
            "/" if modifiers.shift => "?".to_string(),
            "space" | "enter" | "backspace" | "delete" | "tab" => keystroke.key.clone(),
            other if other.chars().count() == 1 => other.to_lowercase(),
            _ => return None,
        };
        Some(Self {
            key,
            mod_: modifiers.platform || modifiers.control,
            shift: modifiers.shift,
            alt: modifiers.alt,
        })
    }

    /// The chord as a GPUI keystroke for a key binding (`cmd-shift-k`).
    pub fn keystroke(&self) -> String {
        let key = match self.key.as_str() {
            "arrowup" => "up",
            "arrowdown" => "down",
            "arrowleft" => "left",
            "arrowright" => "right",
            "?" => "/",
            other => other,
        };
        let mut parts = Vec::new();
        if self.mod_ {
            parts.push(if cfg!(target_os = "macos") {
                "cmd"
            } else {
                "ctrl"
            });
        }
        if self.alt {
            parts.push("alt");
        }
        if self.shift {
            parts.push("shift");
        }
        parts.push(key);
        parts.join("-")
    }

    /// `chordKeys` on macOS: the modifiers' glyphs, then the key.
    pub fn keys(&self) -> Vec<String> {
        let mut parts = Vec::new();
        if self.mod_ {
            parts.push("\u{2318}".to_string());
        }
        if self.alt {
            parts.push("\u{2325}".to_string());
        }
        // "?" already names the shifted key.
        if self.shift && self.key != "?" {
            parts.push("\u{21E7}".to_string());
        }
        parts.push(key_label(&self.key));
        parts
    }

    /// `formatChord` on macOS.
    pub fn format(&self) -> String {
        self.keys().concat()
    }
}

fn key_label(key: &str) -> String {
    match key {
        "space" => "Space".into(),
        "enter" => "Enter".into(),
        "delete" => "Delete".into(),
        "backspace" => "Backspace".into(),
        "tab" => "Tab".into(),
        "arrowup" => "\u{2191}".into(),
        "arrowdown" => "\u{2193}".into(),
        "arrowleft" => "\u{2190}".into(),
        "arrowright" => "\u{2192}".into(),
        "-" => "\u{2212}".into(),
        other => other.to_uppercase(),
    }
}

/// The remaps, by command id, of the commands this catalog has.
pub type Overrides = BTreeMap<String, OwnedChord>;

/// `decodeStoredShortcutOverrides`: the stored remaps this catalog knows and can read.
pub fn decode(stored: &BTreeMap<String, String>) -> Overrides {
    stored
        .iter()
        .filter(|(id, _)| command(id).is_some())
        .filter_map(|(id, raw)| Some((id.clone(), OwnedChord::parse(raw)?)))
        .collect()
}

pub fn encode(overrides: &Overrides) -> BTreeMap<String, String> {
    overrides
        .iter()
        .map(|(id, chord)| (id.clone(), chord.serialize()))
        .collect()
}

/// `resolveChord`: the remap, else the default.
pub fn resolve(command: &Command, overrides: &Overrides) -> OwnedChord {
    overrides
        .get(command.id)
        .cloned()
        .unwrap_or_else(|| command.default.into())
}

/// `effectiveChords`: the remap alone, else the default and its aliases.
pub fn effective(command: &Command, overrides: &Overrides) -> Vec<OwnedChord> {
    match overrides.get(command.id) {
        Some(chord) => vec![chord.clone()],
        None => std::iter::once(command.default)
            .chain(command.aliases.iter().copied())
            .map(OwnedChord::from)
            .collect(),
    }
}

/// `findConflict`: another command of the same group already answering `chord`.
pub fn conflict(id: &str, chord: &OwnedChord, overrides: &Overrides) -> Option<&'static Command> {
    let own = command(id)?;
    CATALOG.iter().find(|other| {
        other.id != id && other.group == own.group && effective(other, overrides).contains(chord)
    })
}

/// `setOverride`: refused on a conflict; the default chord clears the remap.
pub fn set_override(
    overrides: &mut Overrides,
    id: &str,
    chord: OwnedChord,
) -> Result<(), &'static Command> {
    if let Some(other) = conflict(id, &chord, overrides) {
        return Err(other);
    }
    let Some(own) = command(id) else {
        return Ok(());
    };
    if OwnedChord::from(own.default) == chord {
        overrides.remove(id);
    } else {
        overrides.insert(id.to_string(), chord);
    }
    Ok(())
}

/// Whether `keystroke` is one of the command's chords.
pub fn matches(id: &str, keystroke: &Keystroke, overrides: &Overrides) -> bool {
    let (Some(command), Some(chord)) = (command(id), OwnedChord::from_keystroke(keystroke)) else {
        return false;
    };
    effective(command, overrides).contains(&chord)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chords_serialize_and_parse_as_the_web_does() {
        let chord = OwnedChord::parse("Command+Shift+K").unwrap();
        assert_eq!(chord.serialize(), "mod+shift+k");
        assert_eq!(chord.format(), "\u{2318}\u{21E7}K");
        assert_eq!(chord.keystroke(), "cmd-shift-k");
        assert_eq!(OwnedChord::parse("mod+esc"), None);
        assert_eq!(OwnedChord::parse("mod+k+j"), None);
        assert_eq!(OwnedChord::parse("f12"), None);
        assert_eq!(OwnedChord::parse("return").unwrap().key, "enter");
    }

    #[test]
    fn a_key_press_makes_its_chord() {
        let press = Keystroke::parse("cmd-shift-k").unwrap();
        assert_eq!(
            OwnedChord::from_keystroke(&press).unwrap().serialize(),
            "mod+shift+k"
        );
        let question = Keystroke::parse("shift-/").unwrap();
        assert_eq!(OwnedChord::from_keystroke(&question).unwrap().key, "?");
        assert_eq!(
            OwnedChord::from_keystroke(&Keystroke::parse("escape").unwrap()),
            None
        );
        assert!(matches(
            "shell.show_shortcuts",
            &question,
            &Overrides::new()
        ));
    }

    #[test]
    fn a_remap_is_refused_on_a_conflict_and_cleared_by_the_default() {
        let mut overrides = Overrides::new();
        // C already creates, in the same group.
        let taken = OwnedChord::parse("c").unwrap();
        assert_eq!(
            set_override(&mut overrides, "shell.open_filter", taken)
                .unwrap_err()
                .id,
            "shell.create"
        );
        // Another group's chord is free: D dismisses only in the Inbox.
        let other_group = OwnedChord::parse("d").unwrap();
        assert!(set_override(&mut overrides, "shell.open_filter", other_group).is_ok());
        assert_eq!(overrides["shell.open_filter"].serialize(), "d");
        assert!(
            set_override(
                &mut overrides,
                "shell.open_filter",
                OwnedChord::parse("f").unwrap()
            )
            .is_ok()
        );
        assert!(overrides.is_empty());
    }

    #[test]
    fn stored_remaps_of_unknown_commands_are_dropped() {
        let stored = BTreeMap::from([
            ("shell.create".to_string(), "shift+c".to_string()),
            ("gone.command".to_string(), "x".to_string()),
            ("shell.open_filter".to_string(), "mod+k+j".to_string()),
        ]);
        let overrides = decode(&stored);
        assert_eq!(overrides.len(), 1);
        assert_eq!(encode(&overrides)["shell.create"], "shift+c");
    }
}
