use serde::Deserialize;
use std::time::Duration;

/// Runtime-only rebind: takes effect immediately and resets on the next
/// Hyprland reload/login. `hyprland.lua` can't safely be rewritten
/// line-by-line by a program (hand-authored Lua, arbitrary formatting), so
/// this is the only rebind path; callers must label it clearly as temporary
/// in the UI.
///
/// Hyprland is configured with `hyprland.lua`, where `hyprctl keyword` is
/// refused ("keyword can't work with non-legacy parsers. Use eval.") while
/// still exiting 0. The bind goes through `hyprctl eval` with `hl.bind`
/// instead, the same call BOS's `scripts/input/keybinds.lua` makes, and the
/// reply must be `ok`.
///
/// `bind_value` is the `<mods>,<key>,<dispatcher>,<args>` string this app
/// stores and builds, e.g. `"SUPER,U,exec,breadpad"` (or `",XF86Foo,exec,x"`
/// with no modifier). Only the `exec` dispatcher is supported, which is all
/// binds.json entries can be turned into here.
pub fn rebind_temp(bind_value: &str) -> Result<(), String> {
    let expr = bind_eval_expr(bind_value)?;
    let out = bread_utils::proc::run("hyprctl", &["eval", &expr], Duration::from_secs(3));
    let reply = out.stdout.trim();
    if out.success && reply == "ok" {
        Ok(())
    } else if reply.is_empty() {
        Err(format!("hyprctl eval produced no reply ({})", out.stderr.trim()))
    } else {
        Err(format!("hyprctl eval replied {reply:?}"))
    }
}

/// Quote `s` as a single-quoted Lua string literal.
fn lua_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// Turn `<mods>,<key>,exec,<command>` into
/// `hl.bind('<MOD + MOD + key>', hl.dsp.exec_cmd('<command>'))`.
/// Modifiers are separated by `_` or whitespace in the `<mods>` field and by
/// ` + ` in `hl.bind`'s key string.
fn bind_eval_expr(bind_value: &str) -> Result<String, String> {
    let mut parts = bind_value.splitn(4, ',');
    let mods = parts.next().unwrap_or("").trim();
    let key = parts.next().unwrap_or("").trim();
    let dispatcher = parts.next().unwrap_or("").trim();
    let args = parts.next().unwrap_or("").trim();
    if key.is_empty() {
        return Err(format!("bind {bind_value:?} has no key"));
    }
    if dispatcher != "exec" || args.is_empty() {
        return Err(format!("bind {bind_value:?}: only `exec,<command>` binds can be applied"));
    }
    let mut combo: Vec<&str> = mods.split(|c: char| c == '_' || c.is_whitespace()).filter(|m| !m.is_empty()).collect();
    combo.push(key);
    Ok(format!("hl.bind({}, hl.dsp.exec_cmd({}))", lua_str(&combo.join(" + ")), lua_str(args)))
}

#[derive(Debug, Deserialize)]
pub struct Client {
    pub class: String,
    pub at: (i32, i32),
    pub size: (i32, i32),
    /// Numeric monitor id: join against `monitors()` by `Monitor::id` for a name.
    pub monitor: i32,
}

#[derive(Debug, Deserialize)]
pub struct Monitor {
    pub id: i32,
    pub name: String,
    pub focused: bool,
}

#[derive(Debug, Clone)]
pub struct LayerClient {
    pub namespace: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub monitor: String,
}

fn query_json(cmd: &str) -> Option<serde_json::Value> {
    // Timeout-guarded: an unresponsive hyprctl (Hyprland wedged/reloading)
    // used to be able to block clients()/monitors()/layers() indefinitely.
    bread_utils::proc::run_json("hyprctl", &["-j", cmd], std::time::Duration::from_secs(3))
}

pub fn clients() -> Vec<Client> {
    query_json("clients").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default()
}

pub fn monitors() -> Vec<Monitor> {
    query_json("monitors").and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default()
}

/// `hyprctl -j layers` nests surfaces as `{monitor_name: {levels: {level: [surface]}}}`
/// with no fixed schema the `hyprland` crate or any sibling repo has a typed struct
/// for, so this walks the `serde_json::Value` by hand instead of deriving `Deserialize`.
pub fn layers() -> Vec<LayerClient> {
    let Some(serde_json::Value::Object(by_monitor)) = query_json("layers") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (monitor_name, monitor_val) in by_monitor {
        let Some(levels) = monitor_val.get("levels").and_then(|v| v.as_object()) else {
            continue;
        };
        for entries in levels.values().filter_map(|v| v.as_array()) {
            for entry in entries {
                let fields = (
                    entry.get("namespace").and_then(|v| v.as_str()),
                    entry.get("x").and_then(|v| v.as_i64()),
                    entry.get("y").and_then(|v| v.as_i64()),
                    entry.get("w").and_then(|v| v.as_i64()),
                    entry.get("h").and_then(|v| v.as_i64()),
                );
                if let (Some(namespace), Some(x), Some(y), Some(w), Some(h)) = fields {
                    out.push(LayerClient {
                        namespace: namespace.to_string(),
                        x: x as i32,
                        y: y as i32,
                        w: w as i32,
                        h: h as i32,
                        monitor: monitor_name.clone(),
                    });
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::bind_eval_expr;

    #[test]
    fn exec_bind_with_one_modifier() {
        assert_eq!(
            bind_eval_expr("SUPER,U,exec,breadpad").unwrap(),
            "hl.bind('SUPER + U', hl.dsp.exec_cmd('breadpad'))"
        );
    }

    #[test]
    fn modifiers_split_on_underscore() {
        assert_eq!(
            bind_eval_expr("SUPER_SHIFT,U,exec,breadpad").unwrap(),
            "hl.bind('SUPER + SHIFT + U', hl.dsp.exec_cmd('breadpad'))"
        );
    }

    #[test]
    fn empty_mods_field_binds_the_bare_key() {
        assert_eq!(
            bind_eval_expr(",XF86AudioMute,exec,wpctl set-mute @DEFAULT_SINK@ toggle").unwrap(),
            "hl.bind('XF86AudioMute', hl.dsp.exec_cmd('wpctl set-mute @DEFAULT_SINK@ toggle'))"
        );
    }

    #[test]
    fn command_keeps_commas_quotes_and_backslashes() {
        assert_eq!(
            bind_eval_expr(r"SUPER,U,exec,sh -c 'a, b ; c' \x").unwrap(),
            r"hl.bind('SUPER + U', hl.dsp.exec_cmd('sh -c \'a, b ; c\' \\x'))"
        );
    }

    #[test]
    fn non_exec_and_malformed_binds_are_rejected() {
        assert!(bind_eval_expr("SUPER,Q,killactive,").is_err());
        assert!(bind_eval_expr("SUPER,U,exec").is_err());
        assert!(bind_eval_expr("SUPER").is_err());
        assert!(bind_eval_expr("SUPER,,exec,x").is_err());
    }
}
