//! Resolves a `content::tour::Target` to a screen rect + the `gdk4::Monitor`
//! it's on, via `services::hyprland`'s `hyprctl -j` snapshots. `hyprctl -j
//! clients` reports a window's monitor as a numeric id, while GDK/Wayland
//! only exposes a monitor's connector name; `hyprctl -j monitors` is the
//! join between the two (`Client::monitor == Monitor::id`, then match
//! `Monitor::name` against `gdk4::Monitor::connector()`).

use gdk4::prelude::*;

use breadhelp_core::content::tour::Target;
use breadhelp_core::services::hyprland;
use breadhelp_core::services::hyprland::LayerClient;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct Resolved {
    pub rect: Rect,
    pub monitor: gdk4::Monitor,
}

/// The extra layer breadbar creates in pill (auto-hide) mode: the strip you
/// hover at the top edge to bring the bar back. When the bar is collapsed
/// its own surface is parked above the screen (200x200 at y=-62 on the
/// reported machine), so that rect spotlights empty wallpaper.
const PILL_HOTZONE_SUFFIX: &str = "-pill-hotzone";

fn hotzone_namespace(namespace: &str) -> String {
    format!("{namespace}{PILL_HOTZONE_SUFFIX}")
}

/// Pick the layer that actually represents what the user can see for
/// `namespace`.
///
/// Normally that is the named surface itself. A collapsed pill-mode bar is
/// different: its surface sits partly above the top edge (`y < 0`), while
/// `breadbar-pill-hotzone` is the real on-screen target at y=0. Prefer the
/// hotzone in that case, on the same monitor, and fall back to the original
/// surface if there is no usable hotzone (a third-party bar, or a Hyprland
/// that reports no companion layer).
pub fn pick_layer<'a>(layers: &'a [LayerClient], namespace: &str) -> Option<&'a LayerClient> {
    let main = layers.iter().find(|l| l.namespace == namespace);
    if let Some(layer) = main.filter(|l| l.y >= 0 && l.w > 0 && l.h > 0) {
        return Some(layer);
    }
    let hotzone = hotzone_namespace(namespace);
    let same_monitor = main.map(|l| l.monitor.as_str());
    layers
        .iter()
        .find(|l| {
            l.namespace == hotzone
                && l.y >= 0
                && l.w > 0
                && l.h > 0
                && same_monitor.is_none_or(|monitor| l.monitor == monitor)
        })
        .or(main)
}

fn has_layer(layers: &[LayerClient], namespace: &str) -> bool {
    let hotzone = hotzone_namespace(namespace);
    layers
        .iter()
        .any(|l| l.namespace == namespace || l.namespace == hotzone)
}

/// `None` means the target isn't on screen (app not launched yet, or a
/// pure-concept `Target::None` step with no spotlight at all).
pub fn resolve(target: &Target, display: &gdk4::Display) -> Option<Resolved> {
    let (rect, monitor_name) = match target {
        Target::None => return None,
        Target::Namespace(ns) => {
            let layers = hyprland::layers();
            let layer = pick_layer(&layers, ns)?;
            (
                Rect {
                    x: layer.x,
                    y: layer.y,
                    w: layer.w,
                    h: layer.h,
                },
                layer.monitor.clone(),
            )
        }
        Target::WindowClass(class) => {
            let client = hyprland::clients()
                .into_iter()
                .find(|c| c.class.eq_ignore_ascii_case(class))?;
            let name = hyprland::monitors()
                .into_iter()
                .find(|m| m.id == client.monitor)?
                .name;
            (
                Rect {
                    x: client.at.0,
                    y: client.at.1,
                    w: client.size.0,
                    h: client.size.1,
                },
                name,
            )
        }
    };
    let monitor = find_gdk_monitor(display, &monitor_name)?;
    Some(Resolved { rect, monitor })
}

fn find_gdk_monitor(display: &gdk4::Display, name: &str) -> Option<gdk4::Monitor> {
    display
        .monitors()
        .iter::<gdk4::Monitor>()
        .filter_map(|m| m.ok())
        .find(|m| m.connector().as_deref() == Some(name))
}

/// Falls back to this when a step has no target (nothing to resolve a
/// monitor from) or a `launch`ed target never appeared, the tour should
/// still show up on whichever output the user is actually looking at.
pub fn focused_monitor(display: &gdk4::Display) -> Option<gdk4::Monitor> {
    let name = hyprland::monitors().into_iter().find(|m| m.focused)?.name;
    find_gdk_monitor(display, &name)
}

/// Whether `target` is currently resolvable on screen: checked before
/// firing a step's `launch` command, since breadbox/breadclip/breadsearch
/// toggle-close on a second invocation of the same command; unconditionally
/// launching would close an already-open instance instead of opening one.
pub fn is_resolved(target: &Target) -> bool {
    match target {
        Target::None => true,
        Target::Namespace(ns) => has_layer(&hyprland::layers(), ns),
        Target::WindowClass(class) => hyprland::clients()
            .iter()
            .any(|c| c.class.eq_ignore_ascii_case(class)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(namespace: &str, y: i32, w: i32, h: i32, monitor: &str) -> LayerClient {
        LayerClient {
            namespace: namespace.into(),
            x: 0,
            y,
            w,
            h,
            monitor: monitor.into(),
        }
    }

    #[test]
    fn a_normal_bar_targets_its_own_surface() {
        let layers = vec![layer("breadbar", 0, 1920, 32, "eDP-1")];
        let picked = pick_layer(&layers, "breadbar").unwrap();
        assert_eq!((picked.y, picked.w, picked.h), (0, 1920, 32));
    }

    #[test]
    fn a_collapsed_pill_bar_targets_the_hotzone_instead() {
        // The exact pair from HELP-23: the bar surface is parked above the
        // top edge while the real hit area is a 420x14 strip at y=0.
        let layers = vec![
            layer("breadbar", -62, 200, 200, "eDP-1"),
            layer("breadbar-pill-hotzone", 0, 420, 14, "eDP-1"),
        ];
        let picked = pick_layer(&layers, "breadbar").unwrap();
        assert_eq!(picked.namespace, "breadbar-pill-hotzone");
        assert_eq!((picked.y, picked.w, picked.h), (0, 420, 14));
    }

    #[test]
    fn the_hotzone_is_chosen_on_the_same_monitor_as_the_bar() {
        let layers = vec![
            layer("breadbar", -62, 200, 200, "DP-2"),
            layer("breadbar-pill-hotzone", 0, 420, 14, "eDP-1"),
            layer("breadbar-pill-hotzone", 0, 1920, 14, "DP-2"),
        ];
        let picked = pick_layer(&layers, "breadbar").unwrap();
        assert_eq!(picked.monitor, "DP-2");
        assert_eq!(picked.w, 1920);
    }

    #[test]
    fn a_visible_bar_wins_over_a_leftover_hotzone() {
        let layers = vec![
            layer("breadbar", 0, 1920, 32, "eDP-1"),
            layer("breadbar-pill-hotzone", 0, 420, 14, "eDP-1"),
        ];
        let picked = pick_layer(&layers, "breadbar").unwrap();
        assert_eq!(picked.namespace, "breadbar");
        assert_eq!(picked.h, 32);
    }

    #[test]
    fn a_collapsed_bar_without_a_hotzone_keeps_the_old_fallback() {
        let layers = vec![layer("breadbar", -62, 200, 200, "eDP-1")];
        let picked = pick_layer(&layers, "breadbar").unwrap();
        assert_eq!(picked.namespace, "breadbar");
    }

    #[test]
    fn a_hotzone_alone_still_resolves_and_counts_as_present() {
        let layers = vec![layer("breadbar-pill-hotzone", 0, 420, 14, "eDP-1")];
        assert_eq!(
            pick_layer(&layers, "breadbar").unwrap().namespace,
            "breadbar-pill-hotzone"
        );
        assert!(has_layer(&layers, "breadbar"));
    }

    #[test]
    fn unrelated_layers_do_not_resolve() {
        let layers = vec![layer("breadbox", 0, 1920, 1080, "eDP-1")];
        assert!(pick_layer(&layers, "breadbar").is_none());
        assert!(!has_layer(&layers, "breadbar"));
    }
}
