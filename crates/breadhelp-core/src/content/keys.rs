//! Key names, three ways.
//!
//! binds.json stores XKB keysym names (`comma`, `slash`, `RETURN`,
//! `XF86AudioMute`), guides write what a person types (`super + ,`), and the
//! help window shows keycaps (`Super` `,`). Everything goes through
//! [`canonical`] before two combos are compared, so a guide's
//! `[Show Keybind]super + ,[/Show Keybind]` finds the bind stored as
//! `comma`; [`display`] is the keycap text.

/// Modifier order used whenever a combo is rebuilt, so `shift + super + x`
/// and `super + shift + x` compare equal.
const MOD_ORDER: &[&str] = &["super", "ctrl", "alt", "shift"];

/// The canonical (lowercase keysym) name for one key or modifier token.
pub fn canonical(token: &str) -> String {
    let t = token.trim();
    let lower = t.to_lowercase();
    let mapped = match lower.as_str() {
        "," => "comma",
        "/" => "slash",
        "." => "period",
        "[" => "bracketleft",
        "]" => "bracketright",
        "-" => "minus",
        "=" => "equal",
        ";" => "semicolon",
        "'" => "apostrophe",
        "`" => "grave",
        "\\" => "backslash",
        "enter" => "return",
        "esc" => "escape",
        "del" => "delete",
        "control" => "ctrl",
        "win" | "mod4" | "logo" | "meta" => "super",
        "mod1" => "alt",
        _ => return lower,
    };
    mapped.to_string()
}

pub fn is_modifier(canonical_token: &str) -> bool {
    MOD_ORDER.contains(&canonical_token)
}

/// Canonical modifiers in [`MOD_ORDER`], deduplicated.
pub fn sort_mods(mods: &[String]) -> Vec<String> {
    let canon: Vec<String> = mods.iter().map(|m| canonical(m)).collect();
    MOD_ORDER.iter().filter(|m| canon.iter().any(|c| c == *m)).map(|m| m.to_string()).collect()
}

/// `"super+shift+comma"`: the comparison form of a combo.
pub fn combo_id(mods: &[String], key: &str) -> String {
    let mut parts = sort_mods(mods);
    parts.push(canonical(key));
    parts.join("+")
}

/// Parse a human combo (`"super + ,"`, `"SUPER + comma"`, `"super+/"`) into
/// its [`combo_id`]. Splits on `+` with surrounding whitespace, so a literal
/// `+` key is written `plus`.
pub fn parse_combo_id(text: &str) -> Option<String> {
    let tokens: Vec<&str> = text.split('+').map(str::trim).filter(|s| !s.is_empty()).collect();
    let (key, mods) = tokens.split_last()?;
    let mods: Vec<String> = mods.iter().map(|m| m.to_string()).collect();
    Some(combo_id(&mods, key))
}

/// Keycap text for one canonical token.
pub fn display(canonical_token: &str) -> String {
    let named = match canonical_token {
        "super" => "Super",
        "shift" => "Shift",
        "ctrl" => "Ctrl",
        "alt" => "Alt",
        "return" => "Enter",
        "backspace" => "Backspace",
        "space" => "Space",
        "escape" => "Esc",
        "tab" => "Tab",
        "delete" => "Delete",
        "print" => "Print",
        "comma" => ",",
        "slash" => "/",
        "period" => ".",
        "bracketleft" => "[",
        "bracketright" => "]",
        "minus" => "-",
        "equal" => "=",
        "semicolon" => ";",
        "apostrophe" => "'",
        "grave" => "`",
        "backslash" => "\\",
        "plus" => "+",
        "left" => "←",
        "right" => "→",
        "up" => "↑",
        "down" => "↓",
        "mouse:272" => "Left drag",
        "mouse:273" => "Right drag",
        "mouse_down" => "Scroll down",
        "mouse_up" => "Scroll up",
        "xf86audioraisevolume" => "Volume up",
        "xf86audiolowervolume" => "Volume down",
        "xf86audiomute" => "Mute",
        "xf86audiomicmute" => "Mic mute",
        "xf86monbrightnessup" => "Brightness up",
        "xf86monbrightnessdown" => "Brightness down",
        "xf86audionext" => "Next track",
        "xf86audioprev" => "Previous track",
        "xf86audioplay" => "Play / pause",
        "xf86audiostop" => "Stop",
        "xf86calculator" => "Calculator",
        _ => "",
    };
    if !named.is_empty() {
        return named.to_string();
    }
    if let Some(rest) = canonical_token.strip_prefix("xf86") {
        return capitalize(rest);
    }
    if canonical_token.chars().count() == 1 {
        return canonical_token.to_uppercase();
    }
    if canonical_token.starts_with('f') && canonical_token[1..].chars().all(|c| c.is_ascii_digit()) {
        return canonical_token.to_uppercase();
    }
    capitalize(canonical_token)
}

/// Keycaps for a whole combo: one per modifier, then the key.
pub fn display_combo(mods: &[String], key: &str) -> Vec<String> {
    let mut caps: Vec<String> = sort_mods(mods).iter().map(|m| display(m)).collect();
    caps.push(display(&canonical(key)));
    caps
}

/// Keycaps for a human combo (`"super + ,"`), for guide text. Unparseable
/// input comes back as one cap holding the original text.
pub fn display_text_combo(text: &str) -> Vec<String> {
    let tokens: Vec<&str> = text.split('+').map(str::trim).filter(|s| !s.is_empty()).collect();
    match tokens.split_last() {
        None => vec![text.to_string()],
        Some((key, mods)) => {
            let mods: Vec<String> = mods.iter().map(|m| m.to_string()).collect();
            display_combo(&mods, key)
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbols_and_keysym_names_canonicalize_the_same() {
        assert_eq!(parse_combo_id("super + ,"), parse_combo_id("SUPER + comma"));
        assert_eq!(parse_combo_id("super + /"), parse_combo_id("super+slash"));
        assert_eq!(parse_combo_id("super + enter"), parse_combo_id("SUPER + RETURN"));
        assert_eq!(parse_combo_id("super + shift + esc"), parse_combo_id("SHIFT + SUPER + Escape"));
    }

    #[test]
    fn display_uses_what_the_user_types() {
        assert_eq!(display_combo(&["SUPER".into()], "comma"), vec!["Super", ","]);
        assert_eq!(display_combo(&["SUPER".into(), "SHIFT".into()], "slash"), vec!["Super", "Shift", "/"]);
        assert_eq!(display_combo(&["SUPER".into()], "RETURN"), vec!["Super", "Enter"]);
        assert_eq!(display_combo(&["SUPER".into(), "SHIFT".into()], "Escape"), vec!["Super", "Shift", "Esc"]);
        assert_eq!(display_combo(&[], "XF86AudioRaiseVolume"), vec!["Volume up"]);
        assert_eq!(display_combo(&["SUPER".into()], "mouse:272"), vec!["Super", "Left drag"]);
        assert_eq!(display_combo(&["SUPER".into()], "bracketleft"), vec!["Super", "["]);
        assert_eq!(display_combo(&["SUPER".into()], "f5"), vec!["Super", "F5"]);
        assert_eq!(display_combo(&["SUPER".into()], "e"), vec!["Super", "E"]);
    }

    #[test]
    fn unknown_xf86_keys_still_read() {
        assert_eq!(display("xf86launcha"), "Launcha");
    }

    #[test]
    fn guide_text_combos_render_as_caps() {
        assert_eq!(display_text_combo("super + shift + b"), vec!["Super", "Shift", "B"]);
        assert_eq!(display_text_combo("super + ,"), vec!["Super", ","]);
    }
}
