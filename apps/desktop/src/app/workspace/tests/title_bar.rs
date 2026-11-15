use gpui_kit::component::TITLE_BAR_HEIGHT;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{Decorations, Pixels, Rems, Tiling};

use super::super::title_bar::app_draws_title_bar;
use super::navigation_rail::{
    COMPACT_WIDTH, OPEN_PANE, WorkspaceTestView, navigation_bounds, open_mail_at_width, rem_pixels,
};
use super::*;
use crate::components::{NAVIGATION_PANE_WIDTH, NAVIGATION_RAIL_WIDTH};

const WIDE_WIDTH: Pixels = px(1280.);

fn force_title_bar(view: &WorkspaceTestView, context: &mut VisualTestContext) {
    context.cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.force_title_bar();
            cx.notify();
        });
    });
    draw_account_view(context);
}

#[test]
fn the_app_draws_its_title_bar_only_in_client_decorated_windows() {
    assert!(app_draws_title_bar(Decorations::Client {
        tiling: Tiling::default()
    }));
    assert!(app_draws_title_bar(Decorations::Client {
        tiling: Tiling::tiled()
    }));
    assert!(!app_draws_title_bar(Decorations::Server));
}

#[gpui_kit::test]
async fn a_server_decorated_window_keeps_the_window_managers_title_bar(cx: &mut TestAppContext) {
    let (_view, context) = open_mail_at_width(cx, WIDE_WIDTH).await;

    assert!(context.update(|window, _| window.try_find("title-bar").is_none()));
    assert_eq!(navigation_bounds(context).top(), px(0.));
}

/// Opens Mail `width` wide with the title bar drawn, and checks that the title bar spans the
/// window above a navigation surface `navigation_width` wide and the message list.
async fn assert_title_bar_above_navigation(
    cx: &mut TestAppContext,
    width: Pixels,
    navigation_width: Rems,
) {
    let (view, context) = open_mail_at_width(cx, width).await;
    force_title_bar(&view, context);

    let title_bar = context.update(|window, _| window.find("title-bar").bounds());
    let navigation = navigation_bounds(context);
    let message_list = context.update(|window, _| window.find("mail-message-list").bounds());
    assert_eq!(title_bar.top(), px(0.));
    assert_eq!(title_bar.size.height, TITLE_BAR_HEIGHT);
    assert_eq!(
        title_bar.size.width, width,
        "the title bar spans the window"
    );
    assert_eq!(navigation.top(), title_bar.bottom());
    assert_eq!(
        navigation.size.width,
        rem_pixels(context, navigation_width),
        "the navigation keeps its width"
    );
    assert!(
        message_list.top() >= title_bar.bottom(),
        "the message list {message_list:?} sits below the title bar {title_bar:?}"
    );
}

#[gpui_kit::test]
async fn a_drawn_title_bar_sits_above_the_navigation_pane_in_a_wide_window(
    cx: &mut TestAppContext,
) {
    assert_title_bar_above_navigation(cx, WIDE_WIDTH, NAVIGATION_PANE_WIDTH).await;
}

#[gpui_kit::test]
async fn a_drawn_title_bar_sits_above_the_rail_in_a_compact_window(cx: &mut TestAppContext) {
    assert_title_bar_above_navigation(cx, COMPACT_WIDTH, NAVIGATION_RAIL_WIDTH).await;
}

#[gpui_kit::test]
async fn a_drawn_title_bar_leaves_the_first_tab_stop_in_the_rail(cx: &mut TestAppContext) {
    let (view, context) = open_mail_at_width(cx, COMPACT_WIDTH).await;
    force_title_bar(&view, context);

    context.update(|window, cx| window.focus_next(cx));
    let focused = context.update(|window, _| window.find_by_label(OPEN_PANE).focused());
    assert_eq!(focused, Some(true), "Tab reaches {OPEN_PANE} first");
}
