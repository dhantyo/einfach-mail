#[cfg(not(feature = "ui-preview"))]
use std::ffi::OsStr;

use super::{NavigationSurfaceAppearance, PLATFORM_CAPTION, PlatformCaption, window_options};
#[cfg(not(feature = "ui-preview"))]
use super::{SetupIssue, configured_callback_port, navigation_surface_appearance_from_environment};
use crate::components::NAVIGATION_PANE_WIDTH;
use crate::features::MAIL_MINIMUM_WIDTH;
#[cfg(not(feature = "ui-preview"))]
use einfach_mail_microsoft_graph::CallbackPort;
use gpui_kit::component::TITLE_BAR_HEIGHT;
use gpui_kit::{WindowBackgroundAppearance, px};

#[test]
fn window_names_the_app_for_the_taskbar_and_the_compositor() {
    let options = window_options(
        PLATFORM_CAPTION,
        NavigationSurfaceAppearance::Opaque,
        px(16.),
    );
    let title = options
        .titlebar
        .and_then(|titlebar| titlebar.title)
        .expect("the main window sets a title");

    assert_eq!(title.as_ref(), "Einfach Mail");
    assert_eq!(options.app_id.as_deref(), Some("einfach-mail"));
}

#[cfg(not(feature = "ui-preview"))]
#[test]
fn desktop_auth_requires_a_registered_fixed_loopback_port() {
    assert_eq!(
        configured_callback_port(None),
        Err(SetupIssue::MissingCallbackPort)
    );
    assert_eq!(
        configured_callback_port(Some("0")),
        Err(SetupIssue::InvalidCallbackPort)
    );
    assert_eq!(
        configured_callback_port(Some("invalid")),
        Err(SetupIssue::InvalidCallbackPort)
    );
    assert!(matches!(
        configured_callback_port(Some("43123")),
        Ok(CallbackPort::Fixed(port)) if port.get() == 43123
    ));
}

#[cfg(not(feature = "ui-preview"))]
#[test]
fn navigation_translucency_requires_explicit_wayland_opt_in() {
    assert_eq!(
        navigation_surface_appearance_from_environment(None, Some(OsStr::new("wayland-0"))),
        NavigationSurfaceAppearance::Opaque
    );
    assert_eq!(
        navigation_surface_appearance_from_environment(
            Some(OsStr::new("1")),
            Some(OsStr::new("wayland-0"))
        ),
        NavigationSurfaceAppearance::Translucent
    );
    assert_eq!(
        navigation_surface_appearance_from_environment(Some(OsStr::new("1")), Some(OsStr::new(""))),
        NavigationSurfaceAppearance::Opaque
    );
    assert_eq!(
        navigation_surface_appearance_from_environment(Some(OsStr::new("1")), None),
        NavigationSurfaceAppearance::Opaque
    );
}

#[test]
fn window_background_matches_navigation_surface_appearance() {
    assert_eq!(
        window_options(
            PLATFORM_CAPTION,
            NavigationSurfaceAppearance::Opaque,
            px(16.)
        )
        .window_background,
        WindowBackgroundAppearance::Opaque
    );
    assert_eq!(
        window_options(
            PLATFORM_CAPTION,
            NavigationSurfaceAppearance::Translucent,
            px(16.)
        )
        .window_background,
        WindowBackgroundAppearance::Transparent
    );
}

#[test]
fn window_minimum_keeps_the_navigation_pane_beside_the_narrowest_mail_layout() {
    let rem_size = px(16.);
    let minimum = window_options(
        PLATFORM_CAPTION,
        NavigationSurfaceAppearance::Opaque,
        rem_size,
    )
    .window_min_size
    .expect("the main window sets a minimum size");

    assert_eq!(
        minimum.width,
        NAVIGATION_PANE_WIDTH.to_pixels(rem_size) + MAIL_MINIMUM_WIDTH.to_pixels(rem_size)
    );
    assert!(minimum.height > px(0.));
}

#[test]
fn a_hidden_platform_caption_makes_the_system_title_bar_transparent_and_keeps_the_title() {
    let options = |caption| window_options(caption, NavigationSurfaceAppearance::Opaque, px(16.));
    let hidden = options(PlatformCaption::Hidden);
    let drawn = options(PlatformCaption::Drawn);

    let hidden_titlebar = hidden
        .titlebar
        .expect("the main window sets titlebar options");
    let drawn_titlebar = drawn
        .titlebar
        .expect("the main window sets titlebar options");
    assert!(hidden_titlebar.appears_transparent);
    assert!(!drawn_titlebar.appears_transparent);
    assert_eq!(hidden_titlebar.title.as_deref(), Some("Einfach Mail"));
    let minimum_height = |minimum: Option<gpui_kit::Size<gpui_kit::Pixels>>| {
        minimum.expect("the main window sets a minimum size").height
    };
    assert_eq!(
        minimum_height(hidden.window_min_size),
        minimum_height(drawn.window_min_size) + TITLE_BAR_HEIGHT,
        "the app's title bar adds its height to the minimum"
    );
}

#[test]
fn only_windows_hides_the_platform_caption() {
    let expected = if cfg!(target_os = "windows") {
        PlatformCaption::Hidden
    } else {
        PlatformCaption::Drawn
    };
    assert_eq!(PLATFORM_CAPTION, expected);
}
