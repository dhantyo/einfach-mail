//! The accent pill that marks the selected item of a list, as in the Windows 11 list item.

use std::time::Duration;

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{
    Animation, AnimationExt as _, App, Div, ElementId, Rems, SharedString, div, ease_out_quint,
    prelude::*, rems,
};

pub(crate) const PILL_WIDTH: Rems = rems(0.1875);
pub(crate) const PILL_HEIGHT: Rems = rems(1.);
/// Whether a pill that appears grows from nothing. The system reduced-motion setting turns this
/// off for every pill at once.
pub(crate) const ANIMATE_SELECTION_PILL: bool = true;
const GROW_DURATION: Duration = Duration::from_millis(200);

/// The pill at the leading edge of a selected item, centered vertically in the item.
///
/// The pill is an overlay without listeners, so pointer events reach the item beneath it. When
/// `animated` is set, a pill that newly appears grows from its center to full height in 200 ms
/// and decelerates. The slot that carries `selector` keeps its full size, so layout and tests
/// do not depend on the animation.
pub(crate) fn selection_pill(item_id: &str, left: Rems, animated: bool, cx: &App) -> Div {
    let selector = format!("{item_id}-pill");
    let color = cx.theme().primary;
    let pill = div().w(PILL_WIDTH).rounded_full().bg(color);
    let pill = if animated {
        pill.with_animation(
            ElementId::from(SharedString::from(format!("{item_id}-pill-grow"))),
            Animation::new(GROW_DURATION).with_easing(ease_out_quint()),
            |pill, progress| pill.h(PILL_HEIGHT * progress),
        )
        .into_any_element()
    } else {
        pill.h(PILL_HEIGHT).into_any_element()
    };
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
                .flex()
                .flex_col()
                .justify_center()
                .child(pill),
        )
}
