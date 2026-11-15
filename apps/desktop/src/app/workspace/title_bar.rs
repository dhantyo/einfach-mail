//! The title bar the workspace draws where the platform draws none.

use gpui_kit::component::{ActiveTheme as _, TitleBar};
use gpui_kit::{App, Background, Decorations, ParentElement as _, Styled as _, div};

use crate::app::bootstrap::WINDOW_TITLE;

/// Whether the platform draws the window's caption.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlatformCaption {
    /// The platform or the window manager draws the caption wherever the window is
    /// server-decorated.
    Drawn,
    /// The window asks the platform to hide its caption, and the app draws the title bar.
    Hidden,
}

/// The caption choice for this build: Windows hides its caption so the title bar follows the
/// app's theme, and Linux keeps the window manager's.
pub(crate) const PLATFORM_CAPTION: PlatformCaption = if cfg!(target_os = "windows") {
    PlatformCaption::Hidden
} else {
    PlatformCaption::Drawn
};

/// Returns whether the app draws its own title bar in a window with these `decorations`.
///
/// The app draws it when the platform `caption` is hidden, or when the window is
/// client-decorated, such as one on GNOME under Wayland. Otherwise the window manager draws it.
pub(crate) fn app_draws_title_bar(caption: PlatformCaption, decorations: Decorations) -> bool {
    caption == PlatformCaption::Hidden || matches!(decorations, Decorations::Client { .. })
}

/// Builds GPUI Kit's [`TitleBar`] with the app name on `background`.
///
/// The bar keeps GPUI Kit's drag area, double-click maximize, window menu, and caption buttons.
pub(super) fn render_title_bar(background: Background, cx: &App) -> TitleBar {
    TitleBar::new()
        .bg(background)
        .child(div().text_color(cx.theme().foreground).child(WINDOW_TITLE))
}
