use gpui_kit::component::{
    ActiveTheme, ColorTokens, Disableable, Icon, StyledExt, TitleBar,
    avatar::Avatar,
    button::{Button, ButtonVariants as _},
};
#[cfg(test)]
use gpui_kit::test::TestSupportExt as _;
use gpui_kit::{
    AnyElement, App, Context, IntoElement, Rems, Render, Role, Window, div, prelude::*, rems,
};

use crate::components::{AppIcon, InfoBar, NAVIGATION_PANE_WIDTH, NAVIGATION_RAIL_WIDTH, NavItem};
use crate::features::{
    AppearancePreference, MAIL_SPLIT_WINDOW_WIDTH, SettingsViewState,
    account_cache_open_error_message, render_settings,
};
use einfach_mail_application::AccountCacheOpenError;

use super::Workspace;
use super::state::{CompactNavigation, WorkspaceContent, WorkspaceSection};
use super::title_bar::{app_draws_title_bar, render_title_bar};

/// The destinations in menu order.
const DESTINATIONS: [WorkspaceSection; 4] = [
    WorkspaceSection::Mail,
    WorkspaceSection::Calendar,
    WorkspaceSection::Accounts,
    WorkspaceSection::Settings,
];

/// The rail toggle's width: the rail less its padding, so the toggle keeps its place when the
/// pane opens.
const COMPACT_NAVIGATION_TOGGLE_WIDTH: Rems = rems(NAVIGATION_RAIL_WIDTH.0 - 1.);

/// Builds a destination's [`NavItem`] from its id, label, and icon.
type DestinationItem = fn(&'static str, &'static str, AppIcon) -> NavItem;

impl<P: einfach_mail_application::AccountProvider + 'static> Workspace<P> {
    fn render_navigation(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        match self.shown_compact_navigation(window) {
            None => self.render_navigation_pane(None, cx),
            Some(CompactNavigation::Pane) => {
                let toggle = self.render_compact_navigation_toggle(cx);
                self.render_navigation_pane(Some(toggle), cx)
            }
            Some(CompactNavigation::Rail) => self.render_navigation_rail(cx),
        }
    }

    /// What a compact window shows in place of the navigation pane, or `None` in a wide window,
    /// which always shows the pane. A window is compact below Mail's split layout.
    fn shown_compact_navigation(&self, window: &Window) -> Option<CompactNavigation> {
        let compact_below = MAIL_SPLIT_WINDOW_WIDTH.to_pixels(window.rem_size());
        (window.bounds().size.width < compact_below).then_some(self.compact_navigation)
    }

    /// The full pane. A compact window adds the `toggle` that closes it above the account.
    fn render_navigation_pane(&self, toggle: Option<Button>, cx: &mut Context<Self>) -> AnyElement {
        self.render_navigation_surface(NAVIGATION_PANE_WIDTH, cx)
            .children(toggle.map(render_compact_navigation_toggle_row))
            .when_some(
                self.account_view.read(cx).navigation_profile(),
                |navigation, profile| {
                    navigation.child(Self::render_active_account_identity(&profile, cx))
                },
            )
            .child(self.render_contextual_navigation())
            .child(self.render_destination_switcher(cx))
            .into_any_element()
    }

    /// The icon rail: the toggle that opens the pane, the current destination's rail items, and
    /// the destinations in a column.
    fn render_navigation_rail(&self, cx: &mut Context<Self>) -> AnyElement {
        let toggle = self.render_compact_navigation_toggle(cx);
        self.render_navigation_surface(NAVIGATION_RAIL_WIDTH, cx)
            .child(render_compact_navigation_toggle_row(toggle))
            .child(self.render_contextual_rail(cx))
            .child(self.render_destination_rail(cx))
            .into_any_element()
    }

    /// The title bar on the theme's title-bar surface, made translucent with the navigation pane.
    fn render_title_bar(&self, cx: &App) -> TitleBar {
        let surface = self
            .navigation_surface_appearance
            .navigation_background(cx.theme().tokens.title_bar.into());
        render_title_bar(surface, cx)
    }

    fn render_navigation_surface(
        &self,
        width: Rems,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + ParentElement + Styled + use<P> {
        let navigation = div()
            .id("workspace-navigation")
            .role(Role::Navigation)
            .aria_label("Main navigation")
            .w(width)
            .h_full()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .bg(self
                .navigation_surface_appearance
                .navigation_background(cx.theme().tokens.sidebar.into()))
            .text_color(cx.theme().sidebar_foreground)
            .border_r_1()
            .border_color(cx.theme().sidebar_border);
        #[cfg(test)]
        let navigation = navigation.test_support();
        navigation
    }

    fn render_compact_navigation_toggle(&self, cx: &mut Context<Self>) -> Button {
        let (label, icon) = match self.compact_navigation {
            CompactNavigation::Rail => ("Open navigation pane", AppIcon::OpenNavigation),
            CompactNavigation::Pane => ("Close navigation pane", AppIcon::CloseNavigation),
        };
        Button::new("compact-navigation-toggle")
            .debug_selector(|| "compact-navigation-toggle".to_owned())
            .ghost()
            .icon(Icon::new(icon))
            .accessibility_label(label)
            .tooltip(label)
            .w(COMPACT_NAVIGATION_TOGGLE_WIDTH)
            .track_focus(&self.compact_navigation_toggle_focus)
            .on_click(cx.listener(|workspace, _, _, cx| workspace.toggle_compact_navigation(cx)))
    }

    fn render_contextual_navigation(&self) -> AnyElement {
        match &self.content {
            WorkspaceContent::Mail {
                folder_navigation, ..
            } => folder_navigation.clone().into_any_element(),
            WorkspaceContent::Calendar { navigation, .. } => navigation.clone().into_any_element(),
            _ => div().flex_1().into_any_element(),
        }
    }

    fn render_contextual_rail(&self, cx: &App) -> AnyElement {
        match &self.content {
            WorkspaceContent::Mail {
                folder_navigation, ..
            } => folder_navigation.read(cx).render_rail(cx),
            WorkspaceContent::Calendar { navigation, .. } => navigation.read(cx).render_rail(cx),
            _ => div().flex_1().into_any_element(),
        }
    }

    fn render_active_account_identity(
        profile: &einfach_mail_domain::AccountProfile,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = cx.theme().semantic_tokens().colors;
        let identity = div()
            .id("active-account-identity")
            .role(Role::Group)
            .aria_label(format!("Active account: {}", profile.label()))
            .w_full()
            .flex()
            .items_center()
            .gap_3()
            .px_5()
            .py_3()
            .child(Avatar::new().name(profile.label().to_owned()).size_8())
            .child(Self::render_active_account_details(profile, colors));
        #[cfg(test)]
        let identity = identity.test_support();
        identity
    }

    fn render_active_account_details(
        profile: &einfach_mail_domain::AccountProfile,
        colors: ColorTokens,
    ) -> impl IntoElement {
        let mailbox_address = profile
            .mailbox()
            .map(|mailbox| mailbox.address().to_owned())
            .filter(|address| !profile.label().eq_ignore_ascii_case(address));
        div()
            .min_w_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .w_full()
                    .text_sm()
                    .font_semibold()
                    .truncate()
                    .child(profile.label().to_owned()),
            )
            .when_some(mailbox_address, |details, address| {
                details.child(
                    div()
                        .w_full()
                        .text_xs()
                        .text_color(colors.muted_foreground)
                        .truncate()
                        .child(address),
                )
            })
    }

    fn render_destination_switcher(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Self::render_destination_group(cx)
            .children(self.render_destinations(NavItem::icon_only, cx))
    }

    fn render_destination_rail(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Self::render_destination_group(cx)
            .flex_col()
            .children(self.render_destinations(NavItem::rail, cx))
    }

    fn render_destination_group(
        cx: &mut Context<Self>,
    ) -> impl IntoElement + ParentElement + Styled + use<P> {
        let group = div()
            .id("workspace-destination-switcher")
            .role(Role::Group)
            .aria_label("Destinations")
            .flex()
            .gap_1()
            .p_2()
            .border_t_1()
            .border_color(cx.theme().sidebar_border);
        #[cfg(test)]
        let group = group.test_support();
        group
    }

    fn render_destinations(&self, item: DestinationItem, cx: &mut Context<Self>) -> [NavItem; 4] {
        DESTINATIONS.map(|section| self.render_navigation_item(section, item, cx))
    }

    fn render_navigation_item(
        &self,
        section: WorkspaceSection,
        item: DestinationItem,
        cx: &mut Context<Self>,
    ) -> NavItem {
        let (id, label, icon) = section.navigation_identity();
        item(id, label, icon)
            .selected(self.content.selected_section() == section)
            .unavailable(self.navigation_disabled_reason(section, cx))
            .track_focus(self.navigation_focus(section))
            .on_click(cx.listener(move |workspace, _, _, cx| workspace.navigate_to(section, cx)))
    }

    fn render_workspace_content(
        &self,
        colors: ColorTokens,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        match &self.content {
            WorkspaceContent::Mail { view, .. } => {
                div().size_full().child(view.clone()).into_any_element()
            }
            WorkspaceContent::Calendar { view, .. } => {
                div().size_full().child(view.clone()).into_any_element()
            }
            WorkspaceContent::Settings => self.render_settings_workspace_content(cx),
            WorkspaceContent::Accounts | WorkspaceContent::Opening(_) => {
                self.render_account_workspace_content(colors)
            }
        }
    }

    fn render_account_workspace_content(&self, colors: ColorTokens) -> gpui_kit::AnyElement {
        let notices = self.account_workspace_notices();
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .child(div().flex_1().min_h_0().child(self.account_view.clone()))
            .children(notices)
            .into_any_element()
    }

    fn account_workspace_notices(&self) -> Option<gpui_kit::Div> {
        let error = self.session_open_error;
        let opening = self.content.opening_kind();
        if error.is_none() && opening.is_none() {
            return None;
        }
        Some(
            div()
                .w_full()
                .max_w(rems(44.))
                .mx_auto()
                .flex()
                .flex_col()
                .gap_3()
                .px_8()
                .pb_8()
                .children(error.map(Self::render_session_open_error))
                .children(opening.map(Self::render_opening_progress)),
        )
    }

    fn render_session_open_error(error: AccountCacheOpenError) -> impl IntoElement {
        InfoBar::alert(
            "account-cache-error",
            account_cache_open_error_message(error),
        )
    }

    fn render_opening_progress(opening: super::state::OpeningContent) -> impl IntoElement {
        let label = match opening {
            super::state::OpeningContent::Mail => "Opening mail…",
            super::state::OpeningContent::Calendar => "Opening calendar…",
        };
        div().flex().child(
            Button::new("account-operation-progress")
                .debug_selector(|| "account-operation-progress".to_owned())
                .label(label)
                .loading(true)
                .disabled(true),
        )
    }

    fn render_settings_workspace_content(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let on_change = cx.listener(|workspace, index, window, cx| {
            if let Some(appearance) = AppearancePreference::from_radio_index(*index) {
                workspace.change_appearance(appearance, window, cx);
            }
        });
        div()
            .size_full()
            .child(render_settings(
                SettingsViewState::new(self.appearance_preference, self.appearance_settings_notice),
                on_change,
            ))
            .into_any_element()
    }
}

impl<P: einfach_mail_application::AccountProvider + 'static> Render for Workspace<P> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().semantic_tokens().colors;
        let draws_title_bar =
            self.title_bar_forced || app_draws_title_bar(window.window_decorations());
        div()
            .id("workspace")
            .size_full()
            .flex()
            .flex_col()
            .when(draws_title_bar, |workspace| {
                workspace.child(self.render_title_bar(cx))
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex()
                    .child(self.render_navigation(window, cx))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .h_full()
                            .overflow_hidden()
                            .child(self.render_workspace_content(colors, cx)),
                    ),
            )
            .into_any_element()
    }
}

/// The row that holds the compact navigation toggle at the top of the rail or the opened pane.
fn render_compact_navigation_toggle_row(toggle: Button) -> impl IntoElement {
    div().w_full().p_2().pb_0().child(toggle)
}
