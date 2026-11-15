//! Dimensions the application shell shares with the features it hosts.

use gpui_kit::{Rems, rems};

/// The width of the navigation pane. Mail and Calendar choose their layouts as if the pane took
/// this width, also while a compact window shows the narrower rail, so opening the pane never
/// changes their layout.
pub(crate) const NAVIGATION_PANE_WIDTH: Rems = rems(16.);

/// The width of the icon rail that replaces the navigation pane in a compact window.
pub(crate) const NAVIGATION_RAIL_WIDTH: Rems = rems(3.5);
