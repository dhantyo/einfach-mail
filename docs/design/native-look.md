# Native look on Windows 11 and Linux

Status: proposed design, in progress. [ADR 0069](../decisions/0069-native-window-chrome-and-system-look.md) records the lasting choices. It was decided on a fork and is not yet accepted by the upstream repository owner. Units N2 to N4 are committed on the fork; see the [progress log](#progress-log).

The product owner wants Einfach Mail to feel native on Windows 11 and on Linux, to look modern, and to "just work". The [Fluent restyle](fluent-workspace.md) gave the app one visual language. This design covers what sits around and under that language: the window chrome, the system preferences the app follows, the theme structure that carries them, and the UI font. It adds no mail data and no new destination.

Every claim about Windows behaviour in this document comes from reading source code or Microsoft documentation. Nobody has looked at the app on a Windows machine yet ([ADR 0068](../decisions/0068-windows-ui-preview-build.md)). Such claims are marked **unverified on Windows**.

## Read in this order

1. [Findings](#findings) records what the locked GPUI and GPUI Kit versions provide, with file and line evidence, and what the app does today.
2. [Recommendation](#recommendation) is the design.
3. [Rejected alternatives](#rejected-alternatives) lists what was considered and why it lost.
4. [Unit map](#unit-map) splits the work into small units, cheapest and most visible first.
5. [Platform neutrality and verification](#platform-neutrality-and-verification) says how each unit stays type-checkable and how it is checked.

## Defaults chosen during design

The owner can change these. Record any change here and in ADR 0069.

- The app follows the system accent colour by default. No accent setting is added until it has a persisted behaviour behind it.
- The window title is "Einfach Mail" and the Wayland and X11 application id is `einfach-mail`, matching the XDG directory prefix the app already uses.
- The window corner radius for client-decorated Linux windows is 8 px, the same as `radius.lg` and as Windows 11 top-level windows.
- Windows uses Mica, not Mica Alt. Microsoft recommends Mica Alt for apps with tabs in the title bar; Einfach Mail has none.
- The Windows caption buttons come from GPUI Kit's `TitleBar` first. Exact Windows 11 caption geometry is a later, conditional unit (N11).
- High contrast uses two Einfach high-contrast variants on both platforms first. Reading the Windows contrast-theme system colours is a later step.

## Decisions while the owner was away

The owner can reverse these. Each was made while implementing N2 to N6.

- **N3 found cause 1 already mitigated by GPUI Kit 0.9.1.** `Theme::edit` calls `resolve_default_font` (`gpui-component-0.9.1/src/theme/system_font.rs:32-49`, `theme/mod.rs:336-340`), which replaces `.SystemUIFont` on the theme with the installed family GPUI's fallback stack resolved it to. In this repository's Linux container the build before N3 already drew semibold text in DejaVu Sans Bold. N3 still lands the chooser: it makes the family an explicit, ordered choice instead of the fallback stack's order, which puts Ubuntu and the variable Adwaita Sans first. The Fluent review's regular-weight captures were not reproduced here.
- **N3 passes no desktop font yet.** The chooser takes the portal's `font-name` string and is tested with it, but startup passes none until N7 adds the `SystemLook` reader. Until then Linux uses the platform list.
- **N4 builds no `SystemLook` type.** The install function composes the base theme, a list of variant overlays that is empty until N9 and N10, and a system overlay built in code that holds only the UI font family. N7 adds the snapshot type and its inputs.
- **N5 gives the title bar the navigation pane's surface through the theme.** `einfach-fluent.json` sets `title_bar.background` and `title_bar.border` to the sidebar values, and the workspace replaces `TitleBar`'s default gradient with that flat fill, made translucent with the pane when the Linux opt-in is on. The bar's text uses the theme's body size and foreground.
- **N5 adds a preview override, `EINFACH_MAIL_UI_PREVIEW_TITLE_BAR=always`,** which draws the title bar under server decorations. The design names overrides only for accent, contrast, and reduced motion, but X11 without a compositor always grants server decorations, so a capture cannot show the bar otherwise. Under server decorations `TitleBar` draws no caption buttons (`title_bar.rs:269-272`), so those captures show the bar without them.
- **N5 does not raise the minimum window height on Linux.** The decoration mode is known only after the window opens, so a client-decorated window keeps the 30 rem minimum and gives 34 px of it to the title bar.
- **N6 sets `appears_transparent` directly instead of starting from `TitleBar::title_bar_options()`.** That function also sets a macOS traffic-light position and always hides the caption; the app sets only the title and `appears_transparent`, from one `PLATFORM_CAPTION` constant (`cfg!(target_os = "windows")`) that also feeds the title-bar rule, so both builds compile and test both cases on Linux. N6 needs no `#[cfg(target_os = "windows")]` line. The minimum window height grows by the 34 px title bar where the caption is hidden.

## Findings

Paths below are relative to the Cargo registry source directory (`~/.cargo/registry/src/index.crates.io-*/`). Versions are the ones in `Cargo.lock`: `gpui-kit` 0.9.1, `gpui-component` 0.9.1, `gpui-base` 0.9.1, `gpui-pre` 0.3.12 with its Linux, Windows, and wgpu backends, and `ashpd` 0.13.13.

### What the app does today

- `apps/desktop/src/app/bootstrap.rs:87-93` builds `WindowOptions` with only a background and a minimum size. It sets no `titlebar`, no title, no `app_id`, and no `window_decorations`. No code in `apps/desktop/src` calls `set_window_title` or `set_app_id`.
- The window background is `Opaque`, or `Transparent` behind the Linux opt-in from ADR 0059 (`apps/desktop/src/app/workspace/state.rs:36-41`, `bootstrap.rs:50-76`).
- `components/theme.rs:18` installs the two themes from `einfach-fluent.json` once at startup. The JSON sets no `font.family` and no `window.radius`, so GPUI Kit's defaults apply: `.SystemUIFont` and 0 (`gpui-component-0.9.1/src/theme/mod.rs:778-784`).
- `Workspace::apply_appearance` (`apps/desktop/src/app/workspace.rs:166-169`) calls `Theme::change` with the window appearance, and `observe_window_appearance` (`workspace.rs:126-133`) re-applies it when the system switches between light and dark under the System preference. Light and dark therefore already follow the system on both platforms.

### Window background and Mica

`WindowBackgroundAppearance` has `Opaque`, `Transparent`, `Blurred`, `MicaBackdrop`, and `MicaAltBackdrop` in the locked version (`gpui-pre-0.3.12/src/platform.rs:2897-2919`). ADR 0059 was written against GPUI Fast 0.1.13 and lists only the first three; the default `gpui-pre` backend has the Mica variants now.

On Windows (`gpui-pre-windows-0.3.12/src/window.rs:918-944`), `MicaBackdrop` sets `DWMWA_SYSTEMBACKDROP_TYPE` to `DWMSBT_MAINWINDOW` and `MicaAltBackdrop` to `DWMSBT_TABBEDWINDOW`. The helper returns early, silently, below build 22621 (`window.rs:1606-1627`). The renderer clears to transparent for every background except `Opaque` (`directx_renderer.rs:354-357`) and presents through a premultiplied-alpha composition swap chain (`directx_renderer.rs:1305-1320`). Two consequences:

- On Windows 10 or Windows 11 before 22H2, asking for Mica gives a transparent window with no backdrop. The app must check the build itself before asking.
- No `DwmExtendFrameIntoClientArea` call exists in the Windows backend. Whether the system backdrop shows under GPUI's transparent client area without it is **unverified on Windows**. If it does not, Mica is invisible and the fallback below applies.

GPUI Kit's `Root` paints `tokens.background` over the whole window surface (`gpui-component-0.9.1/src/root.rs:718-731`). A backdrop is only visible where that token and the surfaces above it are transparent, so Mica needs theme work, not only a window option.

On Linux the backend has no Mica; ADR 0059's opt-in translucency and issue #274 still cover Linux.

### Window decorations and the title bar

- `WindowOptions::titlebar` (`platform.rs:2609`) takes `TitlebarOptions` (`platform.rs:2824-2836`): `title`, `appears_transparent` ("Should the default system titlebar be hidden ... (macOS and Windows only)"), and the macOS traffic-light position. `WindowOptions::window_decorations` (`platform.rs:2675-2677`) asks for `WindowDecorations::Server` or `Client` (`platform.rs:827-833`) on X11 and Wayland; window creation requests `Server` when it is `None` (`gpui-pre-0.3.12/src/window.rs:1725-1726`).
- `Window` exposes the native behaviours a custom title bar needs: `window_decorations()` (`window.rs:3258`), `window_controls()` (`window.rs:3273`), `start_window_move()` (`window.rs:3237`), `start_window_resize()` (`window.rs:2721`), `show_window_menu()` (`window.rs:3229`), `zoom_window()` (`window.rs:3224`), `minimize_window()` (`window.rs:6927`), and `request_decorations()` (`window.rs:2698`). `App::button_layout()` (`app.rs:1829`) returns the Linux desktop's button layout, read from the settings portal (`gpui-pre-linux-0.3.12/src/linux/xdg_desktop_portal.rs:70`, `:106-117`).
- `WindowControlArea` (`window.rs:752-761`) marks an element as `Drag`, `Close`, `Max`, or `Min`. On Windows, the hit test turns them into `HTCAPTION`, `HTCLOSE`, `HTMAXBUTTON`, and `HTMINBUTTON` (`gpui-pre-windows-0.3.12/src/events.rs:966-990`), but only when the system title bar is hidden (`events.rs:993-996`). Windows 11 shows the snap-layout flyout when the pointer rests over an area that answers `HTMAXBUTTON`, and `DefWindowProc` gives `HTCAPTION` its drag, double-click maximize, and right-click system menu. Both are **unverified on Windows** for this app.
- With `appears_transparent`, the Windows backend keeps the top frame out of the non-client area (`events.rs:775-797`) and marks the window as non-rude so the taskbar stays on top (`window.rs:1677-1686`). DWM still owns the shadow and the Windows 11 rounded corners, because the window keeps its normal frame styles. **Unverified on Windows.**
- The native Windows caption follows the system appearance only: `configure_dwm_dark_mode` is called at creation and on `ImmersiveColorSet` changes with the system value (`gpui-pre-windows-0.3.12/src/window.rs:576`, `events.rs:1240-1251`, `util.rs:134-149`). `App::set_window_appearance` is a no-op off macOS (`gpui-pre-0.3.12/src/app.rs:1812-1822`). Today a user who picks Dark in Settings while Windows is light gets a light caption above a dark app. **Unverified on Windows**, read from source.
- The window title defaults to an empty string on Windows (`gpui-pre-windows-0.3.12/src/window.rs:485-492`), so the taskbar, Alt+Tab, and the native caption show no name. ADR 0066 removed the brand row because "the system title bar names the app"; nothing names it.

GPUI Kit's `TitleBar` (`gpui-component-0.9.1/src/title_bar.rs`):

- `TitleBar::title_bar_options()` (`:60-66`) sets `appears_transparent: true`; `TitleBar::window_options()` (`:82-92`) adds `app_owns_titlebar_drag`, which only macOS reads.
- The bar is a fixed 34 px tall (`:16`). Its content area is a `WindowControlArea::Drag` region (`:387`). On Linux it maximizes on double-click (`:350-352`) and opens `show_window_menu` on right-click when client-decorated (`:389-404`). It starts a window move on drag (`:358-378`).
- Caption buttons are 34 px wide (`:213`) with Lucide window icons (`:147-153`). On Windows they carry `WindowControlArea::Min`, `Max`, and `Close` (`:222-224`), so the system handles them. On Linux they call `minimize_window`, `zoom_window`, and `remove_window` (`:225-244`). Close hovers in the theme's `danger` colour (`:169-193`).
- On Linux it draws no buttons under server-side decorations (`:269-272`). It shows the buttons the window manager supports (`window_controls()`, `:277-295`), always on the right, and ignores `App::button_layout()`.
- Its background is a gradient of `title_bar` and `background` (`:22-37`, `:345-348`), overridable through `Styled` because `refine_style` runs after it (`:349`). `title_bar.background` is a theme key (`gpui-component-0.9.1/src/theme/schema.rs:623-627`).
- `ControlIcon` is private, so button width and glyphs cannot be changed from the app.

Windows 11 caption buttons are 46 by 32 effective pixels with Segoe Fluent Icons glyphs, and the close hover is red in both light and dark ([Microsoft: title bar design](https://learn.microsoft.com/en-us/windows/apps/design/basics/titlebar-design)). `TitleBar`'s buttons are narrower and use Lucide glyphs, and our dark `danger` is `#FF99A4`, a pink close hover. This is the main visible gap from a native Windows caption.

On Linux:

- On Wayland, GPUI starts every window as client-decorated (`gpui-pre-linux-0.3.12/src/linux/wayland/window.rs:626`). When the compositor has no `xdg-decoration` manager, `request_decorations(Server)` logs and stays client-decorated (`window.rs:2189-2208`). GNOME's Mutter does not implement `xdg-decoration`; KWin, Hyprland, and Sway do. The app draws no title bar of its own, so **on GNOME Wayland the window has no title, no caption buttons, and no drag area** today. This follows from source and has not been seen in a real GNOME session. The resize band and shadow still exist, because GPUI Kit's `Root` wraps every window in `window_border()` (`gpui-component-0.9.1/src/root.rs:734-742`, `window_border.rs:165-260`).
- `window_border()` reads the corner radius from the theme's `window.radius` (`window_border.rs:28-30`, `schema.rs:75-76`), which defaults to 0.

### System preferences

| Preference | Windows 11 | Linux | What GPUI or GPUI Kit provides |
|---|---|---|---|
| Light or dark | `UISettings` foreground colour (`gpui-pre-windows-0.3.12/src/util.rs:160-170`) | Portal `color-scheme` (`gpui-pre-linux-0.3.12/src/linux/xdg_desktop_portal.rs:49-53`, `:119-123`) | `Window::appearance()` and `observe_window_appearance`. Already used. |
| Accent colour | `UISettings::GetColorValue(UIColorType::Accent*)` in `windows` 0.62.2 (`windows-0.62.2/src/Windows/UI/ViewManagement/mod.rs:1977`) | Portal `org.freedesktop.appearance` `accent-color` (`ashpd-0.13.13/src/desktop/settings.rs:294`, change stream `:329`) | Nothing. GPUI's portal reader reads only colour scheme, cursor theme and size, and button layout (`xdg_desktop_portal.rs:49-117`). |
| High contrast | `AccessibilitySettings::HighContrast` (`windows-0.62.2/src/Windows/UI/ViewManagement/mod.rs:15`) | Portal `contrast` (`settings.rs:121`, `:307`, `:340`) | Nothing. |
| Reduced motion | `UISettings::AnimationsEnabled` (`mod.rs:1907`, change event `:2040`) | Portal `reduced-motion` (`settings.rs:162`, `:313`, `:348`) | `App::reduce_motion()` and `App::set_reduce_motion()` (`gpui-pre-0.3.12/src/app.rs:1403-1416`), honoured by GPUI Base motion (`gpui-base-0.9.1/src/motion.rs:126-146`, `:404`). Nothing sets it from the system; it defaults to false. The desktop tests already set it (`apps/desktop/src/test_support.rs:15`). |
| UI font | `SPI_GETICONTITLELOGFONT`, falling back to "Segoe UI" (`gpui-pre-windows-0.3.12/src/direct_write.rs:1730-1748`) | Hard-coded "IBM Plex Sans" (`gpui-pre-linux-0.3.12/src/linux/platform.rs:130`) | `.SystemUIFont` maps to the platform's name (`gpui-pre-0.3.12/src/text_system.rs:1262-1272`). |

The portal accent, contrast, and reduced-motion keys are implemented by GNOME's portal backend. Whether KDE's and Hyprland's portal backends answer them is **unverified**; an unanswered key means "no preference".

### Why unread rows do not render semibold on Linux

The [Fluent review](fluent-workspace.md#known-gaps) recorded that unread rows (`apps/desktop/src/features/mail/view/message_list.rs:292-299`), headers, and every other `font_semibold()` text render at regular weight in the Linux captures. The source shows two causes, the first certain and the second dependent on the installed font:

1. On Linux, GPUI resolves `.SystemUIFont` to the literal family "IBM Plex Sans" (`gpui-pre-linux-0.3.12/src/linux/platform.rs:130`, `gpui-pre-wgpu-0.3.12/src/cosmic_text_system.rs:386`). GPUI Kit does not bundle that font, and desktops rarely install it. When the family is missing, `resolve_font_uncached` walks `fallback_font_stack` (`gpui-pre-0.3.12/src/text_system.rs:261-273`, `:604-614`). Each entry in that stack is built with `font(...)`, which has the default weight (`text_system.rs:1160-1167`), and the lookup resolves the candidate as it is, not with the requested weight (`text_system.rs:579-601`). Every weight therefore lands on the fallback family's regular face, and the result is cached for the requested font. A request for "Noto Sans" at semibold would find its semibold face; a request for `.SystemUIFont` at semibold never can.
2. The Linux text system loads every face at `Weight::NORMAL` (`cosmic_text_system.rs:410-413`, `:575-577`), while cosmic-text applies the `wght` axis from that weight (`cosmic-text-0.19.0/src/font/mod.rs:141-143`). A family installed only as a variable font therefore renders at weight 400 even when named directly. Static families with real semibold or bold faces are not affected. This second cause is read from source; it has not been reproduced.

Windows resolves `.SystemUIFont` to "Segoe UI", which has a semibold face, so this bug should not occur there. **Unverified on Windows.**

## Recommendation

### 1. Window identity

Set `TitlebarOptions::title` to "Einfach Mail" and `WindowOptions::app_id` to `einfach-mail` in `window_options`. Windows then names the taskbar button and Alt+Tab entry, KDE and X11 window managers show the title, and Wayland compositors can group the window. This is platform-neutral data and needs no new code path.

### 2. Window chrome

Windows 11: the app draws its own title bar with GPUI Kit's `TitleBar`, inside the app surface.

- Build `WindowOptions` from `TitleBar::title_bar_options()` with the title from section 1, so `appears_transparent` hides the system caption.
- Render `TitleBar` as the first row of the workspace. It holds the app name at the left, in the theme's body size and foreground. Its background is the theme's `title_bar.background`, which is the navigation pane's surface when the window is opaque and transparent over Mica. This also fixes the light caption over a dark app, because the caption now comes from the theme.
- Drag, double-click maximize, the system menu (right-click and Alt+Space), and the snap-layout flyout come from Windows through `WindowControlArea`, not from app code. DWM keeps the shadow and rounded corners. All of this is **unverified on Windows** and is the acceptance check of unit N6.
- The caption buttons stay GPUI Kit's for now. The app sets `danger` for close hover from the theme; the dark close hover will look pink until N11 or a theme fix.

Linux: keep server-side decorations wherever the compositor grants them, as ADR 0059 decided. KDE, Hyprland, Sway, and X11 window managers keep their own title bars and tiling behaviour. When the compositor answers with client-side decorations (GNOME on Wayland), draw the same `TitleBar`. It then provides the drag area, double-click maximize, the right-click window menu, and caption buttons. The app never asks for client-side decorations on Linux.

One rule decides it, read at render time:

> The app draws its title bar when the platform caption is hidden (Windows) or the window is client-decorated (Linux).

The rule is a small pure function of `cfg!(target_os = "windows")` and `window.window_decorations()`. Set `window.radius` to 8 in the theme so the GNOME client-decorated frame has rounded corners that match its title bar.

Known gap on Linux: `TitleBar` puts minimize, maximize, and close on the right and ignores `App::button_layout()`. GNOME's default layout shows only close. Record it, and raise it upstream with GPUI Kit before writing app-owned controls.

### 3. Mica on Windows 11

Request `WindowBackgroundAppearance::MicaBackdrop` only on Windows build 22621 or later, checked with `windows-version`. On earlier builds, and on Linux, keep the current background. When Mica is requested, install the theme's backdrop variant (section 5): `background`, `title_bar.background`, and `sidebar.background` become transparent, so Mica shows behind the title bar and the navigation pane. The message list, the reader, the calendar grid, and the Accounts and Settings pages keep opaque surfaces, as ADR 0059 requires for work panes. Microsoft describes Mica as opaque and wallpaper-tinted, so it does not lower text contrast. Mica comes last in the unit map, because it is the least certain: if the backdrop is not visible in the owner's look at the preview, the unit ships nothing and the opaque Fluent surfaces stay.

### 4. Follow the system

Add one app-owned snapshot, `SystemLook`, with four optional values: accent colour, high contrast, reduced motion, and the desktop UI font name. "None" means the platform gave no answer, and the app keeps its own default.

- Linux: read the four values through `ashpd` 0.13's `Settings` portal with the same features GPUI uses (`async-io`, `settings`), plus `org.gnome.desktop.interface` `font-name`, which GPUI already reads the same way for the cursor theme (`xdg_desktop_portal.rs:55`). Subscribe to the change streams.
- Windows: read `UISettings` (accent and its shades, `AnimationsEnabled`) and `AccessibilitySettings::HighContrast` through `windows` 0.62, which `gpui-pre-windows` already enables with `UI_ViewManagement`. Re-read on `UISettings::ColorValuesChanged` and `AnimationsEnabledChanged`.
- Validate at the boundary: accent components must be finite and in range, and a font name must match an installed family (`TextSystem::all_font_names()`, `gpui-pre-0.3.12/src/text_system.rs:289-294`) before the app uses it. Never log the values.

What the app does with each value:

- **Reduced motion** sets `App::set_reduce_motion`. GPUI Kit components already honour it.
- **Accent** replaces the `primary.*` keys, `info.background`, and the selected-row tint `list.active.background` in the active theme. On Windows, use the system's own shades the way WinUI does: `AccentDark1` in light mode and `AccentLight2` in dark mode. On Linux, derive the light and dark fills from the single portal colour. In both cases a pure function adjusts lightness until text on the fill reaches 4.5:1 and the fill reaches 3:1 against the pane background, and falls back to the Einfach blue if it cannot. The focus `ring` stays the theme's own colour, because the ADR 0066 halo test depends on it and Windows focus visuals are not accent-coloured.
- **High contrast** installs the "Einfach High Contrast" light or dark variant (section 5) instead of the Fluent pair.
- **UI font** feeds section 6.

The snapshot is read once before the window opens, so the first frame is already right, and again on each change notification, which reinstalls the theme through the same function.

### 5. Theme structure

Keep one theme identity: `einfach-fluent.json` stays the only file with raw colours, and Einfach Mail looks like itself on both platforms. Platform inputs are injected; they do not replace the palette.

- The theme set gains variant overlays: "backdrop" (Mica surfaces) and "high contrast", each with a light and a dark block that lists only the keys it changes. Install merges an overlay's `colors` map over the base theme's map before parsing it into `ThemeConfig` (a `serde_json` object `extend`, no new code beyond that).
- System values (accent, font family) are injected as the last overlay, built in code from validated values, not from literals.
- `components/theme.rs` keeps one install function, which now takes the `SystemLook` and the window material and installs `Theme.light_theme` and `Theme.dark_theme` from base, variant, and system overlays. It runs at startup and whenever the snapshot changes, then calls `Theme::change` with the current mode. It stays the only writer of theme state, which keeps ADR 0066's rule. ADR 0066's "installs once" becomes "installs at startup and on system-look changes".
- Exact-value and contrast tests pin the base output with an empty snapshot, so the injection cannot drift the default palette. Each variant gets its own contrast test.

Separate Windows 11, Adwaita, and Breeze presets were rejected (see below).

### 6. Typography and density

Set `font.family` in the installed theme to a concrete, installed family chosen at startup, instead of leaving `.SystemUIFont`:

- **Windows 11**: "Segoe UI Variable Text" if `all_font_names()` lists it, otherwise `.SystemUIFont`, which is Segoe UI. Whether DirectWrite lists the variable family under that name, and whether its semibold instance renders, is **unverified on Windows**.
- **Linux**: the desktop font from the portal's `font-name` (a Pango string such as "Noto Sans 11"; take the longest prefix that is an installed family), then the first installed family from a short list of families that ship static semibold or bold faces ("Noto Sans", "Cantarell", "Ubuntu", "DejaVu Sans"), then `.SystemUIFont`. This removes cause 1 above.

Cause 2 remains for desktops whose UI font is variable-only. Unread rows keep the accent dot and the accessible "Unread" description, so state never depends on weight alone, and the variable-weight load at `cosmic_text_system.rs:412` should be reported upstream to GPUI Kit.

Keep the 16 px base size and the rem scale from ADR 0066. The system text-scaling preference (Windows Text size, GNOME `text-scaling-factor`) is the zoom question in issue #296 and is out of scope here. Density stays as the Fluent restyle set it; Windows 11 and GNOME both suit its compact rows.

### 7. Ownership

| Piece | Owner | Platform code |
|---|---|---|
| Window options, title, app id, material choice | `apps/desktop/src/app/bootstrap.rs` | `cfg!` only, plus the Windows build check |
| Title bar rule and `TitleBar` row | `apps/desktop/src/app/workspace/` render | none |
| `SystemLook` snapshot, validation, change subscription | new `apps/desktop/src/app/system_look/` | one `linux.rs` and one `windows.rs` reader behind `#[cfg(target_os = ...)]`, and an empty reader elsewhere |
| Theme overlays, accent derivation, font choice | `apps/desktop/src/components/theme.rs` and the JSON | none |

The readers are compiled into the `ui-preview` build as well, so the Windows preview shows the system accent and fonts. They hold no account data. They must not live under `apps/desktop/src/platform/`, which ADR 0068 compiles out of the preview. The preview gets environment overrides for accent, contrast, and reduced motion, like its existing appearance override, so Linux screenshots can show each state without changing the desktop.

## Rejected alternatives

| Option | Why it lost |
|---|---|
| `native-theme-gpui` 0.7.0 | It requires `gpui-kit`, `gpui-component`, and `gpui-base` `^0.8.1`, which excludes our 0.9.1, so it would pull a second GPUI Kit and component library into the graph. It also replaces the app's palette with preset themes. |
| `native-theme` 0.7.0 (core reader, `windows` and `portal` features) | Covers accent, contrast, reduced motion, and fonts on both platforms, MIT OR Apache-2.0 OR 0BSD, MSRV 1.88, compatible `ashpd` and `windows` versions. But it is a full theme model (24 colour roles, 26 widget themes, 17 presets) that would become a second owner of theme values, it merges reader output over presets so "no answer" is hard to tell apart from a preset value, its releases changed minor version three times in five weeks (0.5.9 on 2026-09-25 to 0.7.0 on 2026-10-27), and it adds `native-theme-derive`, `serde_with`, `configparser`, and a 580 KB crate for four values. Revisit if the app needs per-widget metrics. |
| `system-theme` 0.3.0 | Small and neutral (scheme, contrast, accent, change stream), MIT/Apache-2.0. But it has no reduced-motion or font value, declares no MSRV, has had no release since 2026-01-21, takes only the main accent without the Windows shades, and opens a second D-Bus connection on its own thread. |
| `dark-light` 3.0.0 | Reports light or dark only, which GPUI already does. |
| Separate Windows 11, Adwaita, and Breeze presets | Three palettes to keep in step and test for contrast, and the app would stop looking like one product. The native cues that matter (accent, font, chrome, materials, contrast) are injected instead. |
| Ask for client-side decorations everywhere on Linux | A GNOME-style header bar on KDE and in tiling compositors breaks their own title bars, tiling, and window rules. ADR 0059 chose server decorations. |
| Keep the system caption on Windows | No Mica in the caption, no app name inside the app surface, and the caption ignores the in-app light or dark choice. |
| App-drawn caption buttons now | Possible with the public `window_control_area` API, but it reimplements a GPUI Kit component before anyone has seen the existing one on Windows. Kept as conditional unit N11. |
| A separate workspace crate for the readers, so the Windows reader type-checks from Linux | It would make `cargo check --target x86_64-pc-windows-msvc` work for that crate on Linux, but adds a crate that owns no product rule. The Windows preview workflow already compiles the Windows reader. Reconsider if Windows CI turnaround becomes the bottleneck. |
| Mica Alt | Meant for apps with tabs in the title bar. |
| Embedding a font (for example IBM Plex Sans) to fix the weight bug | It would ship a font that is native on neither platform, and Linux variable fonts would still be affected. |

## Unit map

Each unit is one reviewable change. "UI-only" means it changes only `apps/desktop` presentation and theme code, with no provider, persistence, or application crate change.

| Unit | Scope | Acceptance criteria | Verification | UI-only |
|---|---|---|---|---|
| N1 | ADR 0069 and this design | The owner accepts or amends the ADR. | Review. | Docs only |
| N2 | Window identity: title "Einfach Mail" and app id `einfach-mail` in `window_options` | The options carry both values in normal and preview builds. Windows shows the name on the taskbar and in Alt+Tab. | Unit test on `window_options`. On Hyprland, `hyprctl clients -j` shows the title and class. Windows: owner's look at the preview `.exe`. | Yes |
| N3 | Font family: choose an installed UI family at startup (section 6) and install it as `font.family` | Unread rows, page titles, and pane headers render semibold on Linux with a static-weight UI font. A missing family never panics and falls back to `.SystemUIFont`. | Pure tests of the chooser (installed names, portal string, platform list). Xvfb or Hyprland screenshot of Mail in light and dark. Windows: owner's look; confirm Segoe UI Variable. | Yes |
| N4 | Theme plumbing: base plus variant overlays plus system overlay, one install function, `window.radius` 8 | With an empty snapshot, the installed themes equal today's values exactly. No visible change. | Existing exact-value and contrast tests, plus a test that an overlay changes only its keys. | Yes |
| N5 | Linux client-side title bar: draw `TitleBar` when the window is client-decorated | On GNOME Wayland the window has a title, caption buttons, drag, double-click maximize, a right-click window menu, and rounded corners. Under server decorations nothing changes. | Pure test of the title-bar rule, and a headless render test with the rule forced on (the GPUI test window always reports server decorations). Manual look in a GNOME Wayland session, which this repository has not had; unverified until then. | Yes |
| N6 | Windows custom title bar: `appears_transparent`, `TitleBar` with the app name, theme-driven caption colours | Drag, double-click maximize, Alt+Space and right-click system menu, snap-layout flyout on hover over maximize, resize at every edge, DWM shadow and rounded corners, caption follows the in-app appearance. | Pure test of the rule and the window options on Linux. Windows preview workflow build. Owner's look and a checklist of each native behaviour on Windows 11. **Every behaviour is unverified on Windows until then.** | Yes |
| N7 | `SystemLook` readers and reduced motion | The app follows the system reduced-motion setting at start and on change. Missing portal or API answers leave defaults. | Pure tests of validation and of the snapshot-to-effect mapping. Linux: toggle GNOME's reduce-animation setting, or use the preview override, and see sheets open without animation. Windows: preview workflow build, owner toggles "Animation effects". | Yes |
| N8 | Accent injection | Primary buttons, the selection pill, the unread dot, and the selected-row tint use the system accent, with contrast guarded and the Einfach blue as fallback. | Pure tests of the shade and contrast function for light, dark, and extreme accents. Xvfb screenshots with the preview accent override. Windows: owner changes the accent in Settings, Personalization, Colors. | Yes |
| N9 | High contrast variants | With the system high-contrast or contrast setting on, the app installs the Einfach High Contrast pair; every text and boundary meets 7:1 and 3:1. | Contrast tests for both variants. Preview override screenshots. Windows: owner turns on a contrast theme; whether the app should use the theme's system colours instead is recorded as a follow-up. | Yes |
| N10 | Mica on Windows 11 22621 and later, with the backdrop variant | Mica shows behind the title bar and navigation pane; work panes stay opaque; older Windows and Linux are unchanged. | Pure test of the material choice. Windows preview build. Owner's look on Windows 11 22H2 or later. **Unverified on Windows**: if Mica is not visible, the unit lands only the opaque fallback and records the finding. | Yes |
| N11 | Conditional: Windows caption buttons at Windows 11 geometry (46 by 32, Segoe Fluent Icons, red close hover in dark) | Only if the owner's look after N6 finds GPUI Kit's buttons wrong. First ask GPUI Kit upstream for a width and glyph option; build app-owned buttons with `window_control_area` only if upstream declines. | Same as N6. | Yes |

N2 and N3 can land in either order. N4 must land before N8, N9, and N10. N7 does not need it. N5 and N6 share the title-bar rule; land N5 first because it can be checked further on Linux. N8 and N9 can run in parallel after N4 and N7.

## Platform neutrality and verification

`cargo check --target x86_64-pc-windows-msvc` cannot run for the whole desktop graph on Linux, because `aws-lc-sys` and `openssl-sys` build scripts need MSVC ([ADR 0068](../decisions/0068-windows-ui-preview-build.md)). The design keeps nearly all new code compiled and tested on Linux:

- Use `cfg!(target_os = "windows")` (an expression, so both branches compile on Linux) for choices such as the title-bar rule, the font preference list, and whether to ask for Mica.
- Keep `#[cfg(target_os = "windows")]` to exactly two places: `app/system_look/windows.rs` and the build-number check for Mica. Each is a thin call into `windows` or `windows-version` that returns plain values. Everything that interprets those values is a pure function tested on Linux.
- `#[cfg(target_os = "linux")]` covers `app/system_look/linux.rs`, which type-checks and runs on Linux.
- New dependencies are target-specific (`ashpd` on Linux, `windows` and `windows-version` on Windows) and add no lockfile package. `deny.toml` checks only Linux targets, so the Windows-only entries are not covered by `cargo deny check bans` until a Windows target is added there.

How each kind of evidence is gathered:

| Evidence | Where | Covers |
|---|---|---|
| Pure and headless GPUI tests | Linux, `cargo test` with and without `ui-preview` | Window options, the title-bar rule, overlay merging, accent shading and contrast, font choice, snapshot validation, reduced-motion effect |
| Screenshots | Linux, `scripts/preview-shots.sh` on Hyprland, or Xvfb where available | Font weight, accent and contrast variants through preview overrides, the title-bar row when forced |
| Real compositor session | Linux, GNOME Wayland | The client-side title bar of N5. Not available in this repository's sessions so far. |
| Windows preview workflow build | GitHub `windows-latest` runner | That the Windows reader and Mica check compile and link |
| Owner's look | Windows 11, the preview `.exe` | Every Windows behaviour in N2, N3, N6 to N11 |

Record each unit's evidence in the progress log below, and separate automated from manual evidence, as the Fluent review did.

## Progress log

Each unit appends one line when it merges: date, unit, PR, and verification evidence.

- 2026-11-15, N2, local commit on the fork (no PR yet): `window_options` sets the title "Einfach Mail" and the app id `einfach-mail`. Automated: `window_names_the_app_for_the_taskbar_and_the_compositor` runs in the normal and the `ui-preview` test builds (the bootstrap tests now compile in both; the tests that need the real providers stay normal-build only); fmt, Clippy, and desktop tests pass in both builds. Manual: none; Hyprland `hyprctl clients -j` and the Windows taskbar and Alt+Tab are not checked.
- 2026-11-15, N3, local commit on the fork (no PR yet): `install` sets `font.family` on both Fluent themes to the first installed family of the desktop font (none passed until N7), "Segoe UI Variable Text" on Windows, or "Noto Sans", "Cantarell", "Ubuntu", "DejaVu Sans" on Linux, else `.SystemUIFont`. Automated: four pure chooser tests (portal string, missing desktop font, platform lists, empty fallback) and a headless test that an empty font list keeps `.SystemUIFont` without panicking; fmt, Clippy, and desktop tests pass in both builds. Manual: the `ui-preview` build on Xvfb at 1280 by 800 picked DejaVu Sans (this container has no Noto Sans, Cantarell, or Ubuntu); unread rows, the page title, group and pane headers render bold in light and dark ([light](../project/evidence/native-font-mail-light.png), [dark](../project/evidence/native-font-mail-dark.png)). A capture before N3 looked the same, see the decisions above. Windows and Segoe UI Variable are not checked.
- 2026-11-15, N4, local commit on the fork (no PR yet): `install` composes the base JSON themes with overlays in order (`ThemeOverlay`: a `colors` map merges key by key, other keys replace) and ends with the system overlay, which holds only the UI font family; `einfach-fluent.json` sets `window.radius` 8. Automated: the existing exact-value, shape, and contrast tests pass unchanged, the shape test now pins `window.radius`, and new tests check that an overlay changes only its keys and that later overlays win; fmt, Clippy, and desktop tests pass in both builds, and the full `scripts/check-linux-local.sh` gate passes. Manual: `ui-preview` captures of Mail in light and dark on Xvfb differ from the N3 captures only in the minute digits of the message times (ImageMagick compare, one 8 by 649 px region). The 8 px corners appear only on client-decorated windows, which Xvfb does not produce, so they are not checked.
- 2026-11-15, N5, local commit on the fork (no PR yet): the workspace draws GPUI Kit's `TitleBar` with the app name as its first row when `window.window_decorations()` is client-side (`app_draws_title_bar` in `app/workspace/title_bar.rs`); under server decorations the layout is unchanged. Automated: a pure test of the rule (client, tiled client, server), headless tests that the test window (always server-decorated) draws no title bar, that a forced title bar is 34 px tall, spans the window, and sits above the navigation pane at 1280 px and the rail at 640 px with the message list below it, and that Tab still starts in the rail; a theme test that the title bar shares the sidebar fill and border; a preview parse test for the new override; fmt, Clippy, and desktop tests pass in both builds. Manual: `ui-preview` captures on Xvfb with the override ([light](../project/evidence/native-titlebar-mail-light.png), [dark](../project/evidence/native-titlebar-mail-dark.png), [640 px](../project/evidence/native-titlebar-mail-compact-light.png)); no caption buttons or rounded corners there, because Xvfb grants server decorations. A real GNOME Wayland session, caption buttons, drag, double-click maximize, the window menu, and the 8 px corners are not checked.
- 2026-11-15, N6, local commit on the fork (no PR yet): on Windows, `window_options` sets `appears_transparent` and adds the 34 px title bar to the minimum height, and the title-bar rule also draws `TitleBar` wherever the platform caption is hidden, so the Windows caption takes the theme's title-bar surface, app name, and the in-app Light or Dark choice. Linux is unchanged. Automated: pure tests of the rule with a hidden caption, of the window options for both caption choices, and that only Windows hides the caption; fmt, Clippy, and desktop tests pass in both builds, and the full `scripts/check-linux-local.sh` gate passes. Manual: none. The Windows preview workflow build, drag, double-click maximize, Alt+Space and right-click system menu, the snap-layout flyout, resize at every edge, DWM shadow and rounded corners, and the caption following the in-app appearance are **unverified on Windows**.

## Sources

- [ADR 0059: Use a shared, adaptive mail and calendar workspace](../decisions/0059-native-mail-and-calendar-workspace.md)
- [ADR 0066: Use a Fluent visual language and a destination switcher](../decisions/0066-fluent-visual-language-and-navigation.md)
- [ADR 0068: Build the synthetic UI preview for Windows](../decisions/0068-windows-ui-preview-build.md)
- [Fluent workspace design](fluent-workspace.md)
- [GPUI Kit research](../research/gpui-kit.md) and [dependency review](../project/dependency-review.md)
- Locked crate sources: `gpui-pre` 0.3.12, `gpui-pre-windows` 0.3.12, `gpui-pre-linux` 0.3.12, `gpui-pre-wgpu` 0.3.12, `gpui-component` 0.9.1, `gpui-base` 0.9.1, `gpui-kit` 0.9.1, `ashpd` 0.13.13, `cosmic-text` 0.19.0, `windows` 0.62.2, `windows-version` 0.1.7
- Published crate sources from `static.crates.io`, read for the rejected alternatives: `native-theme` 0.7.0, `native-theme-gpui` 0.7.0, `system-theme` 0.3.0, `dark-light` 3.0.0
- [Microsoft: Mica material](https://learn.microsoft.com/en-us/windows/apps/design/style/mica)
- [Microsoft: Title bar design](https://learn.microsoft.com/en-us/windows/apps/design/basics/titlebar-design)
- [Microsoft: Apply Mica in Win32 desktop apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-mica-win32)
- [Microsoft: Support snap layouts for desktop apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-snap-layout-menu)
- [XDG Desktop Portal Settings interface](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html)
- [Wayland `xdg-decoration` protocol](https://wayland.app/protocols/xdg-decoration-unstable-v1)
