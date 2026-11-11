//! The subset of Go's `flag` package the CLI uses.
//!
//! Go parses `-name`, `--name`, `-name=value`, and `-name value`, stops at the
//! first non-flag argument (or `--`), and prints `flag provided but not
//! defined` and the flag set's usage on an error. This module keeps that
//! behavior, with the same messages and the same `PrintDefaults` layout, so a
//! person sees what `polterware` in Go shows.

use std::time::Duration;

/// Why a parse stopped: `-h`/`--help` asked for usage (Go's `flag.ErrHelp`), or
/// a flag was rejected. The message is Go's own text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlagError {
    /// `-h` or `--help`: usage already printed.
    Help,
    /// A malformed or unknown flag: the error and usage are already printed.
    Invalid(String),
}

impl FlagError {
    /// The message Go's `flag` package wraps in its returned error.
    pub fn message(&self) -> String {
        match self {
            FlagError::Help => "flag: help requested".to_string(),
            FlagError::Invalid(message) => message.clone(),
        }
    }
}

enum Value {
    Flag(bool),
    Text(String),
    Wait(i64),
    Items(Vec<String>),
}

struct FlagDef {
    name: &'static str,
    /// The type shown after `-name` in the usage; empty for a bool flag.
    type_name: &'static str,
    /// The default as Go's `PrintDefaults` writes it, or `None` when Go omits
    /// a zero default.
    default: Option<String>,
    usage: &'static str,
    value: Value,
    split_commas: bool,
}

/// A flag set: the flags, the positional arguments of the last parse pass, and
/// the usage text printed on errors.
pub struct FlagSet {
    name: &'static str,
    defs: Vec<FlagDef>,
    positional: Vec<String>,
    custom_usage: Option<String>,
}

impl FlagSet {
    /// Builds a set whose usage header is `Usage of <name>:`.
    pub fn new(name: &'static str) -> FlagSet {
        FlagSet {
            name,
            defs: Vec::new(),
            positional: Vec::new(),
            custom_usage: None,
        }
    }

    /// Replaces the usage Go's `PrintDefaults` builds with explicit text, as
    /// the top-level flag set does.
    pub fn set_usage(&mut self, text: impl Into<String>) {
        self.custom_usage = Some(text.into());
    }

    pub fn bool_flag(&mut self, name: &'static str, default: bool, usage: &'static str) {
        self.defs.push(FlagDef {
            name,
            type_name: "",
            default: default.then(|| "true".to_string()),
            usage,
            value: Value::Flag(default),
            split_commas: false,
        });
    }

    pub fn string_flag(&mut self, name: &'static str, default: &str, usage: &'static str) {
        self.defs.push(FlagDef {
            name,
            type_name: "string",
            default: (!default.is_empty()).then(|| go_quote(default)),
            usage,
            value: Value::Text(default.to_string()),
            split_commas: false,
        });
    }

    pub fn duration_flag(&mut self, name: &'static str, default: i64, usage: &'static str) {
        self.defs.push(FlagDef {
            name,
            type_name: "duration",
            default: (default != 0)
                .then(|| format_go_duration(Duration::from_nanos(default.unsigned_abs()))),
            usage,
            value: Value::Wait(default),
            split_commas: false,
        });
    }

    /// Registers a repeatable string flag (Go's `flag.Var`). With
    /// `split_commas`, each occurrence is split on commas and trimmed, as the
    /// CLI's `--scope` flag does.
    pub fn list_flag(&mut self, name: &'static str, usage: &'static str, split_commas: bool) {
        self.defs.push(FlagDef {
            name,
            type_name: "value",
            default: None,
            usage,
            value: Value::Items(Vec::new()),
            split_commas,
        });
    }

    /// One Go `FlagSet.Parse` pass: leading flags only, leaving the rest as
    /// positional arguments.
    pub fn parse(&mut self, args: &[String]) -> Result<(), FlagError> {
        self.positional.clear();
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            if arg == "--" {
                self.positional = args[index + 1..].to_vec();
                return Ok(());
            }
            if !arg.starts_with('-') || arg.len() == 1 {
                self.positional = args[index..].to_vec();
                return Ok(());
            }
            let body = arg.strip_prefix("--").unwrap_or(&arg[1..]);
            let (name, inline) = match body.split_once('=') {
                Some((name, value)) => (name, Some(value)),
                None => (body, None),
            };
            if name.is_empty() || name == "help" || name == "h" {
                self.print_usage();
                return Err(FlagError::Help);
            }
            let def = self
                .defs
                .iter()
                .position(|def| def.name == name)
                .ok_or_else(|| self.invalid(format!("flag provided but not defined: -{name}")))?;
            let consumed = self.set(index, def, name, inline, args)?;
            index += consumed;
        }
        Ok(())
    }

    /// Go's `parseInterspersed`: positional arguments may sit between flags.
    pub fn parse_interspersed(&mut self, args: &[String]) -> Result<(), FlagError> {
        let mut positional = Vec::new();
        let mut rest: Vec<String> = args.to_vec();
        loop {
            self.parse(&rest)?;
            if self.positional.is_empty() {
                self.positional = positional;
                return Ok(());
            }
            positional.push(self.positional[0].clone());
            rest = self.positional[1..].to_vec();
        }
    }

    /// Sets one flag and answers how many arguments it consumed.
    fn set(
        &mut self,
        index: usize,
        def: usize,
        name: &str,
        inline: Option<&str>,
        args: &[String],
    ) -> Result<usize, FlagError> {
        let kind_is_bool = matches!(self.defs[def].value, Value::Flag(_));
        let (raw, consumed) = match (kind_is_bool, inline) {
            (true, None) => ("true".to_string(), 1),
            (true, Some(value)) => (value.to_string(), 1),
            (false, Some(value)) => (value.to_string(), 1),
            (false, None) => {
                let Some(value) = args.get(index + 1) else {
                    return Err(self.invalid(format!("flag needs an argument: -{name}")));
                };
                (value.clone(), 2)
            }
        };
        let split_commas = self.defs[def].split_commas;
        let result: Result<(), String> = match &mut self.defs[def].value {
            Value::Flag(value) => match parse_go_bool(&raw) {
                Some(parsed) => {
                    *value = parsed;
                    Ok(())
                }
                None => Err("parse error".to_string()),
            },
            Value::Text(value) => {
                *value = raw.clone();
                Ok(())
            }
            Value::Wait(value) => match parse_go_duration(&raw) {
                Ok(parsed) => {
                    *value = parsed;
                    Ok(())
                }
                Err(err) => Err(err),
            },
            Value::Items(items) => {
                if split_commas {
                    for part in raw.split(',') {
                        let part = part.trim();
                        if !part.is_empty() {
                            items.push(part.to_string());
                        }
                    }
                } else {
                    items.push(raw.clone());
                }
                Ok(())
            }
        };
        match result {
            Ok(()) => Ok(consumed),
            Err(err) => Err(self.invalid(format!(
                "invalid value {} for flag -{name}: {err}",
                go_quote(&raw)
            ))),
        }
    }

    /// Prints `message` and the usage, as Go prints both before answering.
    fn invalid(&self, message: String) -> FlagError {
        eprintln!("{message}");
        self.print_usage();
        FlagError::Invalid(message)
    }

    /// Writes the usage to stderr.
    pub fn print_usage(&self) {
        eprint!("{}", self.usage_text());
    }

    /// The usage Go writes: the custom text, or `Usage of <name>:` and the
    /// sorted flags with their defaults.
    pub fn usage_text(&self) -> String {
        if let Some(text) = &self.custom_usage {
            return text.clone();
        }
        let mut text = format!("Usage of {}:\n", self.name);
        let mut defs: Vec<&FlagDef> = self.defs.iter().collect();
        defs.sort_by_key(|def| def.name);
        for def in defs {
            text.push_str(&format!("  -{}", def.name));
            if !def.type_name.is_empty() {
                text.push(' ');
                text.push_str(def.type_name);
            }
            text.push_str("\n    \t");
            text.push_str(def.usage);
            if let Some(default) = &def.default {
                text.push_str(&format!(" (default {default})"));
            }
            text.push('\n');
        }
        text
    }

    pub fn bool_value(&self, name: &str) -> bool {
        match &self.def(name).value {
            Value::Flag(value) => *value,
            _ => panic!("{name} is not a bool flag"),
        }
    }

    pub fn string_value(&self, name: &str) -> &str {
        match &self.def(name).value {
            Value::Text(value) => value,
            _ => panic!("{name} is not a string flag"),
        }
    }

    /// The duration in nanoseconds, as Go keeps it (it may be negative).
    pub fn duration_value(&self, name: &str) -> i64 {
        match &self.def(name).value {
            Value::Wait(value) => *value,
            _ => panic!("{name} is not a duration flag"),
        }
    }

    pub fn list_value(&self, name: &str) -> &[String] {
        match &self.def(name).value {
            Value::Items(items) => items,
            _ => panic!("{name} is not a list flag"),
        }
    }

    /// The positional arguments the last pass left.
    pub fn rest(&self) -> &[String] {
        &self.positional
    }

    fn def(&self, name: &str) -> &FlagDef {
        self.defs
            .iter()
            .find(|def| def.name == name)
            .unwrap_or_else(|| panic!("{name} is not a flag of this set"))
    }
}

/// Go's `strconv.ParseBool`.
fn parse_go_bool(value: &str) -> Option<bool> {
    match value {
        "1" | "t" | "T" | "true" | "TRUE" | "True" => Some(true),
        "0" | "f" | "F" | "false" | "FALSE" | "False" => Some(false),
        _ => None,
    }
}

/// Go's `time.ParseDuration`, in nanoseconds so a sign survives.
pub fn parse_go_duration(value: &str) -> Result<i64, String> {
    let invalid = || format!("time: invalid duration {}", go_quote(value));
    let (negative, rest) = match value.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, value.strip_prefix('+').unwrap_or(value)),
    };
    if rest.is_empty() {
        return Err(invalid());
    }
    if rest == "0" {
        return Ok(0);
    }
    let mut total: u128 = 0;
    let mut chars = rest.char_indices().peekable();
    while let Some(&(_, first)) = chars.peek() {
        if !first.is_ascii_digit() && first != '.' {
            return Err(invalid());
        }
        let mut integer: u128 = 0;
        let mut digits = 0;
        while let Some(&(_, ch)) = chars.peek() {
            if !ch.is_ascii_digit() {
                break;
            }
            integer = integer
                .checked_mul(10)
                .and_then(|value| value.checked_add((ch as u8 - b'0') as u128))
                .ok_or_else(invalid)?;
            digits += 1;
            chars.next();
        }
        let mut fraction: u128 = 0;
        let mut scale: u128 = 1;
        if let Some(&(_, '.')) = chars.peek() {
            chars.next();
            while let Some(&(_, ch)) = chars.peek() {
                if !ch.is_ascii_digit() {
                    break;
                }
                fraction = fraction
                    .checked_mul(10)
                    .and_then(|value| value.checked_add((ch as u8 - b'0') as u128))
                    .ok_or_else(invalid)?;
                scale = scale.checked_mul(10).ok_or_else(invalid)?;
                digits += 1;
                chars.next();
            }
        }
        if digits == 0 {
            return Err(invalid());
        }
        let unit_start = chars.peek().map(|&(index, _)| index).unwrap_or(rest.len());
        let unit: String = rest[unit_start..]
            .chars()
            .take_while(|ch| !ch.is_ascii_digit() && *ch != '.')
            .collect();
        if unit.is_empty() {
            return Err(format!(
                "time: missing unit in duration {}",
                go_quote(value)
            ));
        }
        let multiplier: u128 = match unit.as_str() {
            "ns" => 1,
            "us" | "µs" | "μs" => 1_000,
            "ms" => 1_000_000,
            "s" => 1_000_000_000,
            "m" => 60_000_000_000,
            "h" => 3_600_000_000_000,
            _ => {
                return Err(format!(
                    "time: unknown unit {} in duration {}",
                    go_quote(&unit),
                    go_quote(value)
                ));
            }
        };
        for _ in 0..unit.chars().count() {
            chars.next();
        }
        let integer = integer.checked_mul(multiplier).ok_or_else(invalid)?;
        let scaled = fraction
            .checked_mul(multiplier)
            .map(|value| value / scale)
            .ok_or_else(invalid)?;
        total = total
            .checked_add(integer)
            .and_then(|value| value.checked_add(scaled))
            .ok_or_else(invalid)?;
        if chars.peek().is_none() {
            break;
        }
    }
    let total = i128::try_from(total).map_err(|_| invalid())?;
    let total = if negative { -total } else { total };
    i64::try_from(total).map_err(|_| invalid())
}

/// Go's `time.Duration.String`.
pub fn format_go_duration(duration: Duration) -> String {
    let total = duration.as_nanos();
    if total == 0 {
        return "0s".to_string();
    }
    if total < 1_000_000_000 {
        let (scale, unit) = if total < 1_000 {
            (1, "ns")
        } else if total < 1_000_000 {
            (1_000, "µs")
        } else {
            (1_000_000, "ms")
        };
        let whole = total / scale;
        let fraction = fraction_digits(total % scale, scale, 3);
        return format!("{whole}{fraction}{unit}");
    }
    let seconds = total / 1_000_000_000;
    let nanos = total % 1_000_000_000;
    let fraction = fraction_digits(nanos, 1_000_000_000, 9);
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours}h{minutes}m{seconds}{fraction}s")
    } else if minutes > 0 {
        format!("{minutes}m{seconds}{fraction}s")
    } else {
        format!("{seconds}{fraction}s")
    }
}

/// A decimal fraction of `remainder / scale`, with at most `precision` digits
/// and trailing zeros dropped, as `fmtFrac` writes it.
fn fraction_digits(remainder: u128, scale: u128, precision: u32) -> String {
    if remainder == 0 {
        return String::new();
    }
    let digits = precision as usize;
    let scaled = (remainder * 10u128.pow(precision)) / scale;
    let mut text = format!("{scaled:0width$}", width = digits);
    while text.ends_with('0') {
        text.pop();
    }
    if text.is_empty() {
        String::new()
    } else {
        format!(".{text}")
    }
}

/// Go's `%q` for a string: double quotes with Go's escapes.
pub fn go_quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x07' => out.push_str("\\a"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x0b' => out.push_str("\\v"),
            ch if (ch as u32) < 0x20 || ch as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", ch as u32));
            }
            ch if ch.is_control() => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn flags_take_both_dash_forms_and_interspersed_arguments() {
        let mut flags = FlagSet::new("t");
        flags.string_flag("project", "", "project id");
        flags.bool_flag("exact", false, "exact");
        let args = vec![
            "first".to_string(),
            "--project".to_string(),
            "p1".to_string(),
            "second".to_string(),
            "-exact".to_string(),
        ];
        flags.parse_interspersed(&args).expect("parse");
        assert_eq!(flags.rest(), ["first", "second"]);
        assert_eq!(flags.string_value("project"), "p1");
        assert!(flags.bool_value("exact"));
    }

    #[test]
    fn a_bool_flag_never_takes_the_next_argument() {
        let mut flags = FlagSet::new("t");
        flags.bool_flag("json", false, "json");
        let args = vec!["--json".to_string(), "file".to_string()];
        flags.parse(&args).expect("parse");
        assert!(flags.bool_value("json"));
        assert_eq!(flags.rest(), ["file"]);
    }

    #[test]
    fn inline_values_and_the_terminator_follow_go() {
        let mut flags = FlagSet::new("t");
        flags.string_flag("db", "local.db", "path");
        let args = vec![
            "--db=/tmp/x.db".to_string(),
            "--".to_string(),
            "--db".to_string(),
        ];
        flags.parse(&args).expect("parse");
        assert_eq!(flags.string_value("db"), "/tmp/x.db");
        assert_eq!(flags.rest(), ["--db"]);
    }

    #[test]
    fn missing_values_and_unknown_flags_carry_go_messages() {
        let mut flags = FlagSet::new("serve");
        flags.duration_flag("idle", 0, "exit after");
        let mut message = |args: &[&str]| {
            let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
            flags.parse(&args).expect_err("parse").message()
        };
        assert_eq!(message(&["--idle"]), "flag needs an argument: -idle");
        assert_eq!(
            message(&["--bogus"]),
            "flag provided but not defined: -bogus"
        );
    }

    #[test]
    fn usage_lists_flags_sorted_with_go_defaults() {
        let mut flags = FlagSet::new("serve");
        flags.duration_flag("idle", 0, "exit after");
        flags.string_flag("listen", "127.0.0.1:0", "listen address");
        flags.string_flag("token", "", "token");
        assert_eq!(
            flags.usage_text(),
            "Usage of serve:\n  -idle duration\n    \texit after\n  -listen string\n    \tlisten address (default \"127.0.0.1:0\")\n  -token string\n    \ttoken\n"
        );
    }

    #[test]
    fn durations_parse_and_format_as_go_does() {
        for (text, seconds) in [
            ("0", 0.0),
            ("1s", 1.0),
            ("1.5s", 1.5),
            ("2m", 120.0),
            ("1h30m", 5400.0),
            ("500ms", 0.5),
            ("1500us", 0.0015),
            ("2µs", 0.000002),
        ] {
            let parsed = parse_go_duration(text).unwrap_or_else(|err| panic!("{text}: {err}"));
            let got = parsed as f64 / 1e9;
            assert!((got - seconds).abs() < 1e-9, "{text}: {got}");
        }
        assert!(parse_go_duration("1").is_err());
        assert!(parse_go_duration("1d").is_err());
        assert!(parse_go_duration("").is_err());
        assert_eq!(format_go_duration(Duration::ZERO), "0s");
        assert_eq!(format_go_duration(Duration::from_secs(60)), "1m0s");
        assert_eq!(format_go_duration(Duration::from_secs(3661)), "1h1m1s");
        assert_eq!(format_go_duration(Duration::from_millis(1500)), "1.5s");
        assert_eq!(format_go_duration(Duration::from_micros(1500)), "1.5ms");
    }

    #[test]
    fn go_quote_escapes_like_fmt_q() {
        assert_eq!(go_quote("run"), "\"run\"");
        assert_eq!(go_quote("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
        assert_eq!(go_quote("\u{7}"), "\"\\a\"");
    }
}
