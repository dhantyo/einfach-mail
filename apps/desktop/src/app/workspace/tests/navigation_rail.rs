use einfach_mail_application::{MailSession, MailWorkflow};
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{Bounds, Pixels, Role, size};

use super::destination_switcher::FolderAccountSession;
use super::navigation::press_enter;
use super::*;
use crate::components::{NAVIGATION_PANE_WIDTH, NAVIGATION_RAIL_WIDTH};
use crate::features::MAIL_SPLIT_WINDOW_WIDTH;
use crate::test_support::mail::{OfflineProvider, fixture};
use crate::test_support::wait_for_ui_condition;

pub(super) const COMPACT_WIDTH: Pixels = px(640.);
pub(super) const OPEN_PANE: &str = "Open navigation pane";
const CLOSE_PANE: &str = "Close navigation pane";
/// The rail's controls in tab order while Mail shows the synthetic Inbox and Archive.
const MAIL_RAIL_TAB_ORDER: [&str; 8] = [
    OPEN_PANE, "New mail", "Inbox", "Archive", "Mail", "Calendar", "Accounts", "Settings",
];

pub(super) type WorkspaceTestView = gpui_kit::Entity<Workspace<TestProvider>>;

/// Opens Mail, with the synthetic Inbox and Archive, in a window `width` wide.
pub(super) async fn open_mail_at_width(
    cx: &mut TestAppContext,
    width: Pixels,
) -> (WorkspaceTestView, &mut VisualTestContext) {
    cx.update(gpui_kit::init);
    let (profile, cache, _) = fixture(synthetic_account_id());
    let mail: Arc<dyn MailSession> = Arc::new(MailWorkflow::new(
        profile.id().clone(),
        OfflineProvider {
            body_fetches: Arc::new(AtomicUsize::new(0)),
        },
        cache,
    ));
    let factory: Arc<dyn AccountSessionFactory> = Arc::new(TestAccountSessionFactory {
        session: Arc::new(FolderAccountSession { mail }),
        fallback_removals: Arc::new(AtomicBool::new(false)),
        session_open_attempts: Arc::new(AtomicUsize::new(0)),
        opened_account_ids: Arc::new(Mutex::new(Vec::new())),
    });
    let workflow = Arc::new(AccountWorkflow::new(TestProvider::default()));
    let (view, context) =
        cx.add_window_view(|_, cx| Workspace::new(Ok(AccountServices::new(workflow, factory)), cx));
    context.simulate_resize(size(width, px(800.)));
    context.cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.test_set_connected_account(connected_account(profile), cx);
            cx.notify();
        });
    });
    context.run_until_parked();
    draw_account_view(context);
    click(context, "nav-mail");
    wait_for_ui_condition(context, |window, _| {
        window.try_find("folder:inbox").is_some()
            && window.find_by_label("New mail").disabled() != Some(true)
    })
    .await;
    (view, context)
}

pub(super) fn navigation_bounds(context: &mut VisualTestContext) -> Bounds<Pixels> {
    context.update(|window, _| window.find_by_label("Main navigation").bounds())
}

fn shows_account_identity(context: &mut VisualTestContext) -> bool {
    context.update(|window, _| {
        window
            .try_find_by_label("Active account: Ada Example")
            .is_some()
    })
}

pub(super) fn rem_pixels(context: &mut VisualTestContext, width: gpui_kit::Rems) -> Pixels {
    let rem_size = context.update(|window, _| window.rem_size());
    width.to_pixels(rem_size)
}

/// Moves focus forward until the control named `label` has it, and fails after a full cycle.
fn tab_to(context: &mut VisualTestContext, label: &str) {
    for _ in 0..32 {
        context.update(|window, cx| window.focus_next(cx));
        if context.update(|window, _| window.find_by_label(label).focused()) == Some(true) {
            return;
        }
    }
    panic!("Tab never reaches {label}");
}

fn selected_folder_id(view: &WorkspaceTestView, context: &mut VisualTestContext) -> String {
    context.cx.update(|cx| {
        let mail = view.read(cx).test_mail_view().expect("Mail is open");
        mail.read(cx)
            .test_selected_folder_id()
            .expect("Mail shows a folder")
            .to_owned()
    })
}

#[gpui_kit::test]
async fn a_compact_window_shows_the_rail_and_gives_the_content_the_freed_width(
    cx: &mut TestAppContext,
) {
    let (_view, context) = open_mail_at_width(cx, COMPACT_WIDTH).await;
    let navigation = navigation_bounds(context);
    let message_list = context.update(|window, _| window.find("mail-message-list").bounds());

    assert_eq!(
        navigation.size.width,
        rem_pixels(context, NAVIGATION_RAIL_WIDTH)
    );
    assert!(message_list.left() >= navigation.right());
    assert!(
        message_list.size.width > COMPACT_WIDTH - rem_pixels(context, NAVIGATION_PANE_WIDTH),
        "the message list {message_list:?} gets the width the pane gave up"
    );
    assert!(
        !shows_account_identity(context),
        "the account identity waits in the opened pane"
    );
    context.update(|window, _| {
        for label in MAIL_RAIL_TAB_ORDER {
            let item = window.find_by_label(label);
            let bounds = item.bounds();
            assert_eq!(item.role(), Some(Role::Button), "{label} is a button");
            assert_eq!(item.description(), Some(label), "{label} has a tooltip");
            assert!(
                bounds.left() >= navigation.left() && bounds.right() <= navigation.right(),
                "{label} {bounds:?} sits in the rail {navigation:?}"
            );
        }
    });
}

#[gpui_kit::test]
async fn the_rail_replaces_the_pane_only_below_the_mail_split_width(cx: &mut TestAppContext) {
    let split_width = {
        let rem_size = px(16.);
        MAIL_SPLIT_WINDOW_WIDTH.to_pixels(rem_size)
    };
    let (_view, context) = open_mail_at_width(cx, split_width).await;

    assert_eq!(
        navigation_bounds(context).size.width,
        rem_pixels(context, NAVIGATION_PANE_WIDTH)
    );
    assert!(context.debug_bounds("compact-navigation-toggle").is_none());
    assert!(shows_account_identity(context));

    context.simulate_resize(size(split_width - px(1.), px(800.)));
    context.run_until_parked();
    draw_account_view(context);

    assert_eq!(
        navigation_bounds(context).size.width,
        rem_pixels(context, NAVIGATION_RAIL_WIDTH)
    );
    assert!(context.debug_bounds("compact-navigation-toggle").is_some());
}

#[gpui_kit::test]
async fn tab_order_in_the_rail_runs_from_the_toggle_to_the_destinations_and_enter_opens_a_folder(
    cx: &mut TestAppContext,
) {
    let (view, context) = open_mail_at_width(cx, COMPACT_WIDTH).await;
    assert_eq!(selected_folder_id(&view, context), "inbox");

    for label in MAIL_RAIL_TAB_ORDER {
        context.update(|window, cx| window.focus_next(cx));
        let focused = context.update(|window, _| window.find_by_label(label).focused());
        assert_eq!(focused, Some(true), "Tab reaches {label} in rail order");
        if label == "Archive" {
            press_enter(context);
            draw_account_view(context);
            assert_eq!(selected_folder_id(&view, context), "archive");
        }
    }
}

#[gpui_kit::test]
async fn enter_opens_each_destination_from_the_rail(cx: &mut TestAppContext) {
    let (view, context) = open_mail_at_width(cx, COMPACT_WIDTH).await;

    for (label, section) in [
        ("Calendar", WorkspaceTestSection::Calendar),
        ("Accounts", WorkspaceTestSection::Accounts),
        ("Settings", WorkspaceTestSection::Settings),
        ("Mail", WorkspaceTestSection::Mail),
    ] {
        context.update(|window, cx| window.blur(cx));
        tab_to(context, label);
        press_enter(context);
        draw_account_view(context);
        assert!(
            context
                .cx
                .update(|cx| view.read(cx).test_selected_section() == section),
            "Enter on {label} opens it"
        );
        assert_eq!(
            navigation_bounds(context).size.width,
            rem_pixels(context, NAVIGATION_RAIL_WIDTH),
            "{label} keeps the rail"
        );
    }
}

#[gpui_kit::test]
async fn the_toggle_opens_the_full_pane_and_closes_it_again(cx: &mut TestAppContext) {
    let (_view, context) = open_mail_at_width(cx, COMPACT_WIDTH).await;
    tab_to(context, OPEN_PANE);

    press_enter(context);
    draw_account_view(context);

    let navigation = navigation_bounds(context);
    let message_list = context.update(|window, _| window.find("mail-message-list").bounds());
    assert_eq!(
        navigation.size.width,
        rem_pixels(context, NAVIGATION_PANE_WIDTH)
    );
    assert!(message_list.left() >= navigation.right());
    assert!(shows_account_identity(context));
    let (toggle_focused, folders) = context.update(|window, _| {
        (
            window.find_by_label(CLOSE_PANE).focused(),
            window.find_by_label("Mail folders").bounds(),
        )
    });
    assert_eq!(toggle_focused, Some(true), "the toggle keeps focus");
    assert!(folders.left() >= navigation.left() && folders.right() <= navigation.right());

    press_enter(context);
    draw_account_view(context);

    assert_eq!(
        navigation_bounds(context).size.width,
        rem_pixels(context, NAVIGATION_RAIL_WIDTH)
    );
    let toggle_focused = context.update(|window, _| window.find_by_label(OPEN_PANE).focused());
    assert_eq!(toggle_focused, Some(true), "the toggle keeps focus");
}

#[gpui_kit::test]
async fn the_calendar_rail_offers_new_event_and_the_opened_pane_shows_the_mini_month(
    cx: &mut TestAppContext,
) {
    let (_view, context) = open_mail_at_width(cx, COMPACT_WIDTH).await;
    click(context, "nav-calendar");

    assert!(
        context.debug_bounds("calendar-mini-days").is_none(),
        "the rail has no room for the mini month"
    );
    let rail = navigation_bounds(context);
    let new_event = context.update(|window, _| {
        let item = window.find_by_label("New event…");
        assert_eq!(item.role(), Some(Role::Button));
        assert_eq!(
            item.description(),
            Some("New event…"),
            "New event has a tooltip"
        );
        item.bounds()
    });
    assert!(
        new_event.left() >= rail.left() && new_event.right() <= rail.right(),
        "New event {new_event:?} sits in the rail {rail:?}"
    );

    click(context, "compact-navigation-toggle");

    let navigation = navigation_bounds(context);
    let mini_month = context
        .debug_bounds("calendar-mini-days")
        .expect("the opened pane shows the mini month");
    assert!(
        mini_month.left() >= navigation.left() && mini_month.right() <= navigation.right(),
        "the mini month {mini_month:?} sits in the pane {navigation:?}"
    );
}
