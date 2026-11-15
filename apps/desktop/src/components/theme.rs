//! The Fluent palette and the status tints derived from it.
//!
//! `theme/einfach-fluent.json` is the only source of raw colors in the desktop crate. Everything
//! else reads them through `cx.theme()`.

use std::rc::Rc;

use gpui_kit::component::{ActiveTheme as _, Theme, ThemeSet};
use gpui_kit::{App, Hsla, SharedString};

const FLUENT_THEME_SET: &str = include_str!("theme/einfach-fluent.json");
const STATUS_TINT_OPACITY: f32 = 0.12;
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
/// Both themes use the UI font family from [`ui_font_family`] for the installed fonts.
/// Call once at startup, after GPUI Kit initializes and before any window opens. Later
/// appearance changes only need `Theme::change`, which reloads the installed themes.
pub(crate) fn install(cx: &mut App) {
    let theme_set: ThemeSet = serde_json::from_str(FLUENT_THEME_SET)
        .expect("the bundled Fluent theme set is valid JSON for GPUI Kit");
    let font_family = ui_font_family(
        None,
        platform_ui_fonts(),
        &cx.text_system().all_font_names(),
    );
    let mut themes = theme_set.themes.into_iter().map(|mut config| {
        config.font_family = Some(font_family.clone());
        Rc::new(config)
    });
    let light = themes
        .next()
        .expect("the Fluent theme set starts with the light theme");
    let dark = themes
        .next()
        .expect("the Fluent theme set ends with the dark theme");
    Theme::update(cx, |theme| {
        theme.light_theme = light;
        theme.dark_theme = dark;
    });
    let mode = cx.theme().mode;
    Theme::change(mode, None, cx);
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
