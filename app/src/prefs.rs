//! This computer's preferences, in the files the Electron Desktop keeps in its user data
//! directory (`main/*-preferences.ts`), so both apps read and write the same choices: the
//! local core's idle timeout and start on open (`core-preferences.json`), what quitting does
//! to it (`quit-preferences.json`), appearance (`appearance-preferences.json`: reduced
//! transparency and app icon), and the remapped shortcuts (`shortcut-preferences.json`). A
//! missing or unreadable file, or one of another version, reads as the defaults.

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

const DEFAULT_ICON_PNG: &[u8] = include_bytes!("../resources/polterware-icon.png");
const ICE_ICON_PNG: &[u8] = include_bytes!("../resources/polterware-ice-icon.png");
const FIRE_ICON_PNG: &[u8] = include_bytes!("../resources/polterware-fire-icon.png");

/// Which app icon variant is shown in the Dock and on the bundle (`appIcon` in
/// `appearance-preferences.json`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppIcon {
    #[default]
    Default,
    Ice,
    Fire,
}

impl AppIcon {
    pub const ALL: [AppIcon; 3] = [AppIcon::Default, AppIcon::Ice, AppIcon::Fire];

    pub fn wire(self) -> &'static str {
        match self {
            AppIcon::Default => "default",
            AppIcon::Ice => "ice",
            AppIcon::Fire => "fire",
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|icon| icon.wire() == value)
    }

    pub fn label(self) -> &'static str {
        match self {
            AppIcon::Default => "Default",
            AppIcon::Ice => "Ice",
            AppIcon::Fire => "Fire",
        }
    }

    pub fn png(self) -> &'static [u8] {
        match self {
            AppIcon::Default => DEFAULT_ICON_PNG,
            AppIcon::Ice => ICE_ICON_PNG,
            AppIcon::Fire => FIRE_ICON_PNG,
        }
    }

    /// The override PNG passed to `app_icon::apply`: `None` clears custom overrides on a bundle
    /// and falls back to `Default.png()` when unbundled.
    pub fn override_png(self) -> Option<&'static [u8]> {
        match self {
            AppIcon::Default => None,
            AppIcon::Ice | AppIcon::Fire => Some(self.png()),
        }
    }
}

/// Appearance choices stored in `appearance-preferences.json`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppearancePrefs {
    pub reduce_transparency: bool,
    pub app_icon: AppIcon,
}

pub fn appearance_prefs(dir: &Path) -> AppearancePrefs {
    let Some(record) = read(dir, APPEARANCE_FILE) else {
        return AppearancePrefs::default();
    };
    AppearancePrefs {
        reduce_transparency: record
            .get("reduceTransparency")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        app_icon: record
            .get("appIcon")
            .and_then(Value::as_str)
            .and_then(AppIcon::from_wire)
            .unwrap_or_default(),
    }
}

pub fn save_appearance_prefs(dir: &Path, prefs: AppearancePrefs) -> std::io::Result<()> {
    write(
        dir,
        APPEARANCE_FILE,
        json!({
            "version": VERSION,
            "reduceTransparency": prefs.reduce_transparency,
            "appIcon": prefs.app_icon.wire(),
        }),
    )
}

pub fn reduce_transparency(dir: &Path) -> bool {
    appearance_prefs(dir).reduce_transparency
}

pub fn save_reduce_transparency(dir: &Path, reduce: bool) -> std::io::Result<()> {
    let mut prefs = appearance_prefs(dir);
    prefs.reduce_transparency = reduce;
    save_appearance_prefs(dir, prefs)
}

pub fn app_icon(dir: &Path) -> AppIcon {
    appearance_prefs(dir).app_icon
}

pub fn save_app_icon(dir: &Path, icon: AppIcon) -> std::io::Result<()> {
    let mut prefs = appearance_prefs(dir);
    prefs.app_icon = icon;
    save_appearance_prefs(dir, prefs)
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

/// Electron's `work.list.display` is machine-local, shared by every project and board.
/// GPUI has no renderer localStorage: keep the same value schema in its user data directory.
pub fn work_list_display(dir: &Path) -> crate::work::Display {
    let value = fs::read_to_string(dir.join("work-list-display.json"))
        .ok()
        .and_then(|body| serde_json::from_str::<Value>(&body).ok())
        .unwrap_or(Value::Null);
    crate::work::Display::from_json(&value["work.list.display"])
}
pub fn save_work_list_display(dir: &Path, display: &crate::work::Display) -> std::io::Result<()> {
    write(
        dir,
        "work-list-display.json",
        json!({ "work.list.display": display.json() }),
    )
}

/// Electron's machine-local `structure.tree` and `structure.expanded` remembered values.
/// GPUI has no renderer localStorage, so it preserves their schema in user data.
pub fn structure_tree(dir: &Path) -> (bool, Vec<String>) {
    let record = read(dir, "structure-preferences.json");
    let open = record
        .as_ref()
        .and_then(|r| r.get("structure.tree"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let expanded = record
        .as_ref()
        .and_then(|r| r.get("structure.expanded"))
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_else(|| vec!["structure:project".into()]);
    (open, expanded)
}

pub fn save_structure_tree(dir: &Path, open: bool, expanded: &[String]) -> std::io::Result<()> {
    write(
        dir,
        "structure-preferences.json",
        json!({"version": VERSION, "structure.tree": open, "structure.expanded": expanded}),
    )
}

const CANVAS_FILE: &str = "canvas-preferences.json";

/// Electron's remembered `canvas.<project>` (the canvas each project's Canvas opens on) and the
/// editor's panel widths (`polterware.canvas.library`, `polterware.canvas.inspector`), which
/// GPUI keeps in user data since it has no renderer localStorage.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasPrefs {
    pub remembered: BTreeMap<String, String>,
    pub library: Option<f32>,
    pub inspector: Option<f32>,
}

pub fn canvas_prefs(dir: &Path) -> CanvasPrefs {
    let Some(record) = read(dir, CANVAS_FILE) else {
        return CanvasPrefs::default();
    };
    let width = |key: &str| record.get(key).and_then(Value::as_f64).map(|w| w as f32);
    CanvasPrefs {
        remembered: record
            .get("remembered")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        library: width("polterware.canvas.library"),
        inspector: width("polterware.canvas.inspector"),
    }
}

pub fn save_canvas_prefs(dir: &Path, prefs: &CanvasPrefs) -> std::io::Result<()> {
    write(
        dir,
        CANVAS_FILE,
        json!({
            "version": VERSION,
            "remembered": prefs.remembered,
            "polterware.canvas.library": prefs.library,
            "polterware.canvas.inspector": prefs.inspector,
        }),
    )
}

/// Changes the stored canvas preferences in place.
pub fn update_canvas_prefs(dir: &Path, change: impl FnOnce(&mut CanvasPrefs)) {
    // The app's own tests open canvases and drag panels; they never write a person's prefs.
    if cfg!(test) && dir == user_data_dir() {
        return;
    }
    let mut prefs = canvas_prefs(dir);
    change(&mut prefs);
    let _ = save_canvas_prefs(dir, &prefs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_prefs_round_trip() {
        let dir = scratch("canvas-prefs");
        assert_eq!(canvas_prefs(&dir), CanvasPrefs::default());
        update_canvas_prefs(&dir, |prefs| {
            prefs.remembered.insert("p1".into(), "c1".into());
            prefs.library = Some(200.0);
        });
        let prefs = canvas_prefs(&dir);
        assert_eq!(prefs.remembered.get("p1").map(String::as_str), Some("c1"));
        assert_eq!(prefs.library, Some(200.0));
        assert_eq!(prefs.inspector, None);
    }

    #[test]
    fn work_display_round_trips_and_rejects_unknown_values_independently() {
        let dir = scratch("work-display");
        assert_eq!(work_list_display(&dir), crate::work::Display::default());
        let display = crate::work::Display::from_json(
            &json!({ "grouping": "milestone", "ordering": "title", "direction": "ascending", "properties": [] }),
        );
        save_work_list_display(&dir, &display).unwrap();
        assert_eq!(work_list_display(&dir), display);
        let normalized = crate::work::Display::from_json(
            &json!({ "grouping": "owner", "ordering": "title", "properties": ["updated", "slug", "gone", "slug"] }),
        );
        assert_eq!(normalized.grouping, crate::work::Grouping::Stage);
        assert_eq!(normalized.ordering, crate::work::Ordering::Title);
        assert!(normalized.descending);
        assert_eq!(
            normalized.properties,
            vec![crate::work::Property::Slug, crate::work::Property::Updated]
        );
        fs::write(dir.join("work-list-display.json"), "invalid").unwrap();
        assert_eq!(work_list_display(&dir), crate::work::Display::default());
        fs::remove_dir_all(dir).unwrap();
    }

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("polterware-prefs-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn structure_choices_round_trip_and_invalid_fields_fall_back_independently() {
        let dir = scratch("structure");
        assert_eq!(
            structure_tree(&dir),
            (true, vec!["structure:project".into()])
        );
        save_structure_tree(&dir, false, &[]).unwrap();
        assert_eq!(structure_tree(&dir), (false, vec![]));
        let expanded = vec!["structure:project".into(), "domain-1".into()];
        save_structure_tree(&dir, true, &expanded).unwrap();
        assert_eq!(structure_tree(&dir), (true, expanded));
        write(
            &dir,
            "structure-preferences.json",
            json!({"version": 1, "structure.tree": false, "structure.expanded": [7]}),
        )
        .unwrap();
        assert_eq!(
            structure_tree(&dir),
            (false, vec!["structure:project".into()])
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_files_read_as_the_defaults() {
        let dir = scratch("defaults");
        assert_eq!(core_prefs(&dir), CorePrefs::default());
        assert_eq!(quit_mode(&dir), QuitMode::Ask);
        assert_eq!(appearance_prefs(&dir), AppearancePrefs::default());
        assert!(!reduce_transparency(&dir));
        assert_eq!(app_icon(&dir), AppIcon::Default);
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
    fn app_icon_round_trips_and_falls_back_to_default() {
        let dir = scratch("app-icon");
        assert_eq!(app_icon(&dir), AppIcon::Default);

        for icon in AppIcon::ALL {
            save_app_icon(&dir, icon).unwrap();
            let stored: Value =
                serde_json::from_str(&fs::read_to_string(dir.join(APPEARANCE_FILE)).unwrap())
                    .unwrap();
            assert_eq!(stored["appIcon"], json!(icon.wire()));
            assert_eq!(app_icon(&dir), icon);
        }

        // Changing either field in appearance-preferences.json preserves the other.
        save_app_icon(&dir, AppIcon::Ice).unwrap();
        save_reduce_transparency(&dir, true).unwrap();
        assert_eq!(
            appearance_prefs(&dir),
            AppearancePrefs {
                reduce_transparency: true,
                app_icon: AppIcon::Ice,
            }
        );
        save_app_icon(&dir, AppIcon::Fire).unwrap();
        assert!(reduce_transparency(&dir));
        assert_eq!(app_icon(&dir), AppIcon::Fire);

        // Missing or unknown appIcon reads as default while preserving valid fields.
        write(
            &dir,
            APPEARANCE_FILE,
            json!({ "version": 1, "reduceTransparency": true }),
        )
        .unwrap();
        assert_eq!(app_icon(&dir), AppIcon::Default);
        assert!(reduce_transparency(&dir));

        write(
            &dir,
            APPEARANCE_FILE,
            json!({ "version": 1, "reduceTransparency": true, "appIcon": "neon" }),
        )
        .unwrap();
        assert_eq!(app_icon(&dir), AppIcon::Default);
        assert!(reduce_transparency(&dir));

        write(
            &dir,
            APPEARANCE_FILE,
            json!({ "version": 1, "appIcon": 42 }),
        )
        .unwrap();
        assert_eq!(app_icon(&dir), AppIcon::Default);

        fs::write(dir.join(APPEARANCE_FILE), "invalid").unwrap();
        assert_eq!(app_icon(&dir), AppIcon::Default);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn app_icon_keys_map_to_distinct_pngs() {
        let mapped: Vec<(&str, AppIcon, &[u8])> = ["default", "ice", "fire"]
            .into_iter()
            .map(|key| {
                let icon = AppIcon::from_wire(key).expect("known key");
                (key, icon, icon.png())
            })
            .collect();
        assert_eq!(mapped[0].1, AppIcon::Default);
        assert_eq!(mapped[1].1, AppIcon::Ice);
        assert_eq!(mapped[2].1, AppIcon::Fire);
        assert_eq!(AppIcon::from_wire("unknown"), None);

        for (_, icon, png) in &mapped {
            assert!(
                png.starts_with(b"\x89PNG\r\n\x1a\n"),
                "{icon:?} must be PNG"
            );
        }
        assert_ne!(mapped[0].2, mapped[1].2);
        assert_ne!(mapped[1].2, mapped[2].2);
        assert_ne!(mapped[0].2, mapped[2].2);

        assert_eq!(AppIcon::Default.override_png(), None);
        assert_eq!(AppIcon::Ice.override_png(), Some(AppIcon::Ice.png()));
        assert_eq!(AppIcon::Fire.override_png(), Some(AppIcon::Fire.png()));
    }

    #[test]
    fn another_version_reads_as_the_defaults() {
        let dir = scratch("version");
        write(
            &dir,
            APPEARANCE_FILE,
            json!({ "version": 2, "reduceTransparency": true, "appIcon": "ice" }),
        )
        .unwrap();
        assert!(!reduce_transparency(&dir));
        assert_eq!(app_icon(&dir), AppIcon::Default);
        let _ = fs::remove_dir_all(&dir);
    }
}
