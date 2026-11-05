//! This computer's preferences, in the files the Electron Desktop keeps in its user data
//! directory (`main/*-preferences.ts`), so both apps read and write the same choices: the
//! local core's idle timeout and start on open (`core-preferences.json`), what quitting does
//! to it (`quit-preferences.json`), reduced transparency (`appearance-preferences.json`), and
//! the remapped shortcuts (`shortcut-preferences.json`). A missing or unreadable file, or one
//! of another version, reads as the defaults.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

/// Every file's `version`.
const VERSION: u64 = 1;
const CORE_FILE: &str = "core-preferences.json";
const QUIT_FILE: &str = "quit-preferences.json";
const APPEARANCE_FILE: &str = "appearance-preferences.json";
const SHORTCUT_FILE: &str = "shortcut-preferences.json";

/// The Electron Desktop's user data: `POLTERWARE_USER_DATA`, else
/// `~/Library/Application Support/polterware`.
pub fn user_data_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("POLTERWARE_USER_DATA").filter(|path| !path.is_empty()) {
        return PathBuf::from(path);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("Library/Application Support/polterware")
}

/// How long a core this app starts waits without clients before it exits
/// (`CORE_IDLE_OPTIONS`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Idle {
    #[default]
    Never,
    TenMinutes,
    ThirtyMinutes,
    OneHour,
}

impl Idle {
    pub const ALL: [Idle; 4] = [
        Idle::Never,
        Idle::TenMinutes,
        Idle::ThirtyMinutes,
        Idle::OneHour,
    ];

    pub fn wire(self) -> &'static str {
        match self {
            Idle::Never => "never",
            Idle::TenMinutes => "10m",
            Idle::ThirtyMinutes => "30m",
            Idle::OneHour => "1h",
        }
    }

    fn from_wire(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|idle| idle.wire() == value)
    }

    /// `IDLE_OPTIONS`' labels.
    pub fn label(self) -> &'static str {
        match self {
            Idle::Never => "Never (keep running)",
            Idle::TenMinutes => "After 10 minutes",
            Idle::ThirtyMinutes => "After 30 minutes",
            Idle::OneHour => "After 1 hour",
        }
    }

    /// `polterware serve --idle`'s value (`idleToFlag`).
    pub fn flag(self) -> &'static str {
        match self {
            Idle::Never => "0",
            idle => idle.wire(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorePrefs {
    pub idle: Idle,
    /// Start (or attach to) the core when the app opens.
    pub auto_start: bool,
}

impl Default for CorePrefs {
    fn default() -> Self {
        Self {
            idle: Idle::Never,
            auto_start: true,
        }
    }
}

/// What quitting does to the core: ask, or keep it running, or stop it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QuitMode {
    #[default]
    Ask,
    Keep,
    Stop,
}

impl QuitMode {
    pub const ALL: [QuitMode; 3] = [QuitMode::Ask, QuitMode::Keep, QuitMode::Stop];

    /// `QUIT_OPTIONS`' labels.
    pub fn label(self) -> &'static str {
        match self {
            QuitMode::Ask => "Ask me",
            QuitMode::Keep => "Keep the core running",
            QuitMode::Stop => "Stop the core",
        }
    }
}

/// Reads them from `dir`.
pub fn core_prefs(dir: &Path) -> CorePrefs {
    let Some(record) = read(dir, CORE_FILE) else {
        return CorePrefs::default();
    };
    CorePrefs {
        idle: record
            .get("idle")
            .and_then(Value::as_str)
            .and_then(Idle::from_wire)
            .unwrap_or_default(),
        auto_start: record.get("autoStart") != Some(&Value::Bool(false)),
    }
}

pub fn save_core_prefs(dir: &Path, prefs: CorePrefs) -> std::io::Result<()> {
    write(
        dir,
        CORE_FILE,
        json!({ "version": VERSION, "idle": prefs.idle.wire(), "autoStart": prefs.auto_start }),
    )
}

/// `quitMode`: asking wins over the stored action.
pub fn quit_mode(dir: &Path) -> QuitMode {
    let Some(record) = read(dir, QUIT_FILE) else {
        return QuitMode::Ask;
    };
    if record.get("confirmOnQuit") != Some(&Value::Bool(false)) {
        return QuitMode::Ask;
    }
    match record.get("quitAction").and_then(Value::as_str) {
        Some("stop") => QuitMode::Stop,
        _ => QuitMode::Keep,
    }
}

/// `saveMode`: Ask keeps the action already stored.
pub fn save_quit_mode(dir: &Path, mode: QuitMode) -> std::io::Result<()> {
    let action = match (mode, read(dir, QUIT_FILE)) {
        (QuitMode::Stop, _) => "stop",
        (QuitMode::Keep, _) => "keep",
        (QuitMode::Ask, Some(record)) if record.get("quitAction") == Some(&json!("stop")) => "stop",
        (QuitMode::Ask, _) => "keep",
    };
    write(
        dir,
        QUIT_FILE,
        json!({
            "version": VERSION,
            "confirmOnQuit": mode == QuitMode::Ask,
            "quitAction": action,
        }),
    )
}

pub fn reduce_transparency(dir: &Path) -> bool {
    read(dir, APPEARANCE_FILE)
        .and_then(|record| record.get("reduceTransparency").and_then(Value::as_bool))
        .unwrap_or(false)
}

pub fn save_reduce_transparency(dir: &Path, reduce: bool) -> std::io::Result<()> {
    write(
        dir,
        APPEARANCE_FILE,
        json!({ "version": VERSION, "reduceTransparency": reduce }),
    )
}

/// The remapped chords by command id, each as `serializeChord` writes it.
pub fn shortcut_overrides(dir: &Path) -> BTreeMap<String, String> {
    let Some(Value::Object(overrides)) =
        read(dir, SHORTCUT_FILE).and_then(|mut record| record.remove("overrides"))
    else {
        return BTreeMap::new();
    };
    overrides
        .into_iter()
        .filter_map(|(id, chord)| {
            let chord = chord.as_str()?.trim().to_lowercase();
            (!chord.is_empty() && !id.trim().is_empty()).then_some((id, chord))
        })
        .collect()
}

pub fn save_shortcut_overrides(
    dir: &Path,
    overrides: &BTreeMap<String, String>,
) -> std::io::Result<()> {
    write(
        dir,
        SHORTCUT_FILE,
        json!({ "version": VERSION, "overrides": overrides }),
    )
}

/// The file's object, when it is this version's.
fn read(dir: &Path, name: &str) -> Option<Map<String, Value>> {
    let body = fs::read_to_string(dir.join(name)).ok()?;
    let Value::Object(record) = serde_json::from_str(&body).ok()? else {
        return None;
    };
    (record.get("version").and_then(Value::as_u64) == Some(VERSION)).then_some(record)
}

/// Written as the Electron Desktop writes them: two-space JSON.
fn write(dir: &Path, name: &str, value: Value) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let body = serde_json::to_string_pretty(&value).map_err(std::io::Error::other)?;
    fs::write(dir.join(name), body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("polterware-prefs-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn missing_files_read_as_the_defaults() {
        let dir = scratch("defaults");
        assert_eq!(core_prefs(&dir), CorePrefs::default());
        assert_eq!(quit_mode(&dir), QuitMode::Ask);
        assert!(!reduce_transparency(&dir));
        assert!(shortcut_overrides(&dir).is_empty());
    }

    #[test]
    fn choices_round_trip_in_the_electron_shape() {
        let dir = scratch("round-trip");
        save_core_prefs(
            &dir,
            CorePrefs {
                idle: Idle::ThirtyMinutes,
                auto_start: false,
            },
        )
        .unwrap();
        let stored: Value =
            serde_json::from_str(&fs::read_to_string(dir.join(CORE_FILE)).unwrap()).unwrap();
        assert_eq!(
            stored,
            json!({ "version": 1, "idle": "30m", "autoStart": false })
        );
        assert_eq!(core_prefs(&dir).idle, Idle::ThirtyMinutes);

        save_quit_mode(&dir, QuitMode::Stop).unwrap();
        assert_eq!(quit_mode(&dir), QuitMode::Stop);
        // Asking again keeps the stored action for when the question is turned off.
        save_quit_mode(&dir, QuitMode::Ask).unwrap();
        assert_eq!(quit_mode(&dir), QuitMode::Ask);
        let stored: Value =
            serde_json::from_str(&fs::read_to_string(dir.join(QUIT_FILE)).unwrap()).unwrap();
        assert_eq!(stored["quitAction"], json!("stop"));

        let overrides = BTreeMap::from([("shell.create".to_string(), "shift+c".to_string())]);
        save_shortcut_overrides(&dir, &overrides).unwrap();
        assert_eq!(shortcut_overrides(&dir), overrides);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn another_version_reads_as_the_defaults() {
        let dir = scratch("version");
        write(
            &dir,
            APPEARANCE_FILE,
            json!({ "version": 2, "reduceTransparency": true }),
        )
        .unwrap();
        assert!(!reduce_transparency(&dir));
        let _ = fs::remove_dir_all(&dir);
    }
}
