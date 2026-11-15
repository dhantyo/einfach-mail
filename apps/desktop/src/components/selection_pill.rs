//! The accent pill that marks the selected item of a list, as in the Windows 11 list item.

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{App, Div, Rems, div, prelude::*, rems};

pub(crate) const PILL_WIDTH: Rems = rems(0.1875);
pub(crate) const PILL_HEIGHT: Rems = rems(1.);
/// The pill at the leading edge of a selected item, centered vertically in the item.
///
/// The pill is an overlay without listeners, so pointer events reach the item beneath it.
pub(crate) fn selection_pill(item_id: &str, left: Rems, cx: &App) -> Div {
    let selector = format!("{item_id}-pill");
    div()
        .absolute()
        .top_0()
        .bottom_0()
        .left(left)
        .flex()
        .items_center()
        .child(
            div()
                .debug_selector(move || selector)
                .w(PILL_WIDTH)
                .h(PILL_HEIGHT)
                .rounded_full()
                .bg(cx.theme().primary),
        )
}
