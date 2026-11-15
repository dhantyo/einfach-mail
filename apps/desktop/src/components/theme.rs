//! The Fluent palette and the status tints derived from it.
//!
//! `theme/einfach-fluent.json` is the only source of raw colors in the desktop crate. Everything
//! else reads them through `cx.theme()`.

use std::rc::Rc;

use gpui_kit::component::{ActiveTheme as _, Theme, ThemeConfig};
use gpui_kit::{App, Hsla, SharedString};
use serde::Deserialize;
use serde_json::{Map, Value};

const FLUENT_THEME_SET: &str = include_str!("theme/einfach-fluent.json");
const STATUS_TINT_OPACITY: f32 = 0.12;
/// How strongly the accent tints a calendar event at rest, under the pointer, and when selected.
/// An event has no outline: the tint and the accent strip at its leading edge carry the color.
pub(crate) const EVENT_TINT_OPACITY: f32 = 0.12;
pub(crate) const EVENT_HOVER_TINT_OPACITY: f32 = 0.15;
pub(crate) const EVENT_SELECTED_TINT_OPACITY: f32 = 0.18;
/// The virtual family that GPUI maps to each platform's UI font.
const SYSTEM_UI_FONT: &str = ".SystemUIFont";
/// The Windows 11 UI family. GPUI maps `.SystemUIFont` to Segoe UI on Windows.
const WINDOWS_UI_FONTS: [&str; 1] = ["Segoe UI Variable Text"];
/// Linux UI families that ship static semibold or bold faces.
///
/// GPUI maps `.SystemUIFont` to "IBM Plex Sans" on Linux, which desktops rarely install, and
/// resolves every weight of a missing family to its fallback's regular face. GPUI's Linux text
/// system also loads each face at regular weight, so a variable-only family renders one weight.
const LINUX_UI_FONTS: [&str; 4] = ["Noto Sans", "Cantarell", "Ubuntu", "DejaVu Sans"];

/// Installs the Fluent light and dark themes as the app's themes and applies the active mode's.
///
/// Each theme is the base theme from `theme/einfach-fluent.json` with the system overlay on top,
/// which sets the UI font family from [`ui_font_family`] for the installed fonts.
/// Call once at startup, after GPUI Kit initializes and before any window opens. Later
/// appearance changes only need `Theme::change`, which reloads the installed themes.
pub(crate) fn install(cx: &mut App) {
    let font_family = ui_font_family(
        None,
        platform_ui_fonts(),
        &cx.text_system().all_font_names(),
    );
    let system_overlay = ThemeOverlay::system(&font_family);
    let [light, dark] = fluent_themes(&[&system_overlay]).map(|theme| {
        let config: ThemeConfig = serde_json::from_value(Value::Object(theme))
            .expect("the composed Fluent theme is valid for GPUI Kit");
        Rc::new(config)
    });
    Theme::update(cx, |theme| {
        theme.light_theme = light;
        theme.dark_theme = dark;
    });
    let mode = cx.theme().mode;
    Theme::change(mode, None, cx);
}

/// Theme keys that one overlay sets on the light and the dark theme.
///
/// Applying an overlay merges its `colors` map into the theme's map key by key and replaces
/// every other key it names. Keys the overlay does not name keep the theme's value.
struct ThemeOverlay {
    light: Map<String, Value>,
    dark: Map<String, Value>,
}

impl ThemeOverlay {
    /// Returns the overlay that sets the UI font family on both themes.
    fn system(font_family: &str) -> Self {
        let keys = Map::from_iter([(
            "font.family".to_owned(),
            Value::String(font_family.to_owned()),
        )]);
        Self {
            light: keys.clone(),
            dark: keys,
        }
    }

    fn apply_to(&self, light: &mut Map<String, Value>, dark: &mut Map<String, Value>) {
        apply_theme_keys(light, &self.light);
        apply_theme_keys(dark, &self.dark);
    }
}

fn apply_theme_keys(theme: &mut Map<String, Value>, overlay_keys: &Map<String, Value>) {
    for (key, value) in overlay_keys {
        match (theme.get_mut(key), value) {
            (Some(Value::Object(theme_colors)), Value::Object(overlay_colors))
                if key == "colors" =>
            {
                theme_colors.extend(overlay_colors.clone());
            }
            _ => {
                theme.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Returns the light and the dark Fluent theme with `overlays` applied in order.
fn fluent_themes(overlays: &[&ThemeOverlay]) -> [Map<String, Value>; 2] {
    let [mut light, mut dark] = base_fluent_themes();
    for overlay in overlays {
        overlay.apply_to(&mut light, &mut dark);
    }
    [light, dark]
}

/// The bundled theme set as JSON objects, before GPUI Kit parses them.
#[derive(Deserialize)]
struct FluentThemeSet {
    /// The light theme, then the dark theme.
    themes: [Map<String, Value>; 2],
}

fn base_fluent_themes() -> [Map<String, Value>; 2] {
    let theme_set: FluentThemeSet = serde_json::from_str(FLUENT_THEME_SET)
        .expect("the bundled Fluent theme set holds the light and then the dark theme");
    theme_set.themes
}

fn platform_ui_fonts() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
        &WINDOWS_UI_FONTS
    } else if cfg!(target_os = "linux") {
        &LINUX_UI_FONTS
    } else {
        &[]
    }
}

/// Returns the first installed family among `desktop_font` and `preferred`, or `.SystemUIFont`.
///
/// `desktop_font` is a Pango font description such as "Noto Sans 11". It names the longest
/// installed family that starts the description and ends at a word boundary. `None` means the
/// desktop reported no font.
fn ui_font_family(
    desktop_font: Option<&str>,
    preferred: &[&str],
    installed: &[String],
) -> SharedString {
    let is_installed = |family: &str| installed.iter().any(|name| name == family);
    let desktop_family = desktop_font.and_then(|description| {
        installed
            .iter()
            .filter(|name| {
                description
                    .strip_prefix(name.as_str())
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
            })
            .max_by_key(|name| name.len())
            .map(String::as_str)
    });
    desktop_family
        .or_else(|| {
            preferred
                .iter()
                .copied()
                .find(|family| is_installed(family))
        })
        .unwrap_or(SYSTEM_UI_FONT)
        .to_owned()
        .into()
}

/// The meaning a status message carries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StatusTone {
    Informational,
    Success,
    Warning,
    Error,
}

/// Colors for one status message.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StatusColors {
    /// A light tint of the tone, drawn behind the message.
    pub(crate) background: Hsla,
    /// The full-strength tone, for the status icon.
    pub(crate) icon: Hsla,
}

/// Returns the colors for `tone` from the active theme.
pub(crate) fn status_colors(tone: StatusTone, cx: &App) -> StatusColors {
    let theme = cx.theme();
    let icon = match tone {
        StatusTone::Informational => theme.info,
        StatusTone::Success => theme.success,
        StatusTone::Warning => theme.warning,
        StatusTone::Error => theme.danger,
    };
    StatusColors {
        background: icon.opacity(STATUS_TINT_OPACITY),
        icon,
    }
}

#[cfg(test)]
mod tests;
