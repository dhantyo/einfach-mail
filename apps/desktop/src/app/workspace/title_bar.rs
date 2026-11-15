//! The title bar the workspace draws where the platform draws none.

use gpui_kit::component::{ActiveTheme as _, TitleBar};
use gpui_kit::{App, Background, Decorations, ParentElement as _, Styled as _, div};

use crate::app::bootstrap::WINDOW_TITLE;

/// Returns whether the app draws its own title bar in a window with these `decorations`.
///
/// A client-decorated window, such as one on GNOME under Wayland, has no title bar unless the
/// app draws one. Under server decorations the window manager draws it.
pub(super) fn app_draws_title_bar(decorations: Decorations) -> bool {
    matches!(decorations, Decorations::Client { .. })
}

/// Builds GPUI Kit's [`TitleBar`] with the app name on `background`.
///
/// The bar keeps GPUI Kit's drag area, double-click maximize, window menu, and caption buttons.
pub(super) fn render_title_bar(background: Background, cx: &App) -> TitleBar {
    TitleBar::new()
        .bg(background)
        .child(div().text_color(cx.theme().foreground).child(WINDOW_TITLE))
}
