# ADR 0069: Draw native window chrome and follow system appearance preferences

- Status: Proposed; amends the window-chrome and translucency bullets of [ADR 0059](0059-native-mail-and-calendar-workspace.md) and the theme installation rule of [ADR 0066](0066-fluent-visual-language-and-navigation.md); decided on a fork and not yet accepted by the upstream repository owner
- Date: 2026-11-14
- Owners: Einfach Mail maintainers

## Context

The product owner wants Einfach Mail to feel native on Windows 11 and Linux, modern, and to work without setup. [ADR 0066](0066-fluent-visual-language-and-navigation.md) gave the app one Fluent palette. [ADR 0068](0068-windows-ui-preview-build.md) proposes a Windows build of the synthetic preview, so the window chrome and system integration can now be reviewed on Windows 11. Nobody has looked at that build on Windows yet.

[ADR 0059](0059-native-mail-and-calendar-workspace.md) chose the system title bar on Linux and limited translucency to an optional effect. It was written against GPUI Fast 0.1.13, whose window background has only `Opaque`, `Transparent`, and `Blurred`.

Evidence from the locked sources (full references in [the native look design](../design/native-look.md#findings)):

- `gpui-pre` 0.3.12 adds `MicaBackdrop` and `MicaAltBackdrop` (`src/platform.rs:2915-2918`). The Windows backend applies them through `DWMWA_SYSTEMBACKDROP_TYPE` and silently does nothing below build 22621 (`gpui-pre-windows` `src/window.rs:918-944`, `:1606-1627`), while still clearing the window to transparent.
- GPUI Kit's `TitleBar` (`gpui-component` 0.9.1 `src/title_bar.rs`) hides the Windows caption with `appears_transparent`, and marks its drag area and caption buttons with `WindowControlArea`. The Windows backend answers those with `HTCAPTION`, `HTMINBUTTON`, `HTMAXBUTTON`, and `HTCLOSE` (`gpui-pre-windows` `src/events.rs:966-990`), which is what Windows needs for drag, double-click maximize, the system menu, and the snap-layout flyout. Its buttons are 34 px wide with Lucide glyphs, narrower than the Windows 11 caption buttons.
- The app sets no window title and no application id. On Windows the title defaults to an empty string (`gpui-pre-windows` `src/window.rs:485-492`).
- The native Windows caption follows the system appearance only, not the in-app Light or Dark choice (`gpui-pre-windows` `src/util.rs:134-149`; `gpui-pre` `src/app.rs:1812-1822`).
- On Wayland, GPUI falls back to client-side decorations when the compositor lacks `xdg-decoration` (`gpui-pre-linux` `src/linux/wayland/window.rs:2189-2208`). GNOME's compositor lacks it, and the app draws no title bar, so on GNOME Wayland the window has no title, caption buttons, or drag area. This follows from source and has not been seen in a real GNOME session.
- GPUI exposes light and dark and an app-wide `reduce_motion` flag, but reads no accent colour, contrast, or reduced-motion preference from either platform. `ashpd` 0.13, already locked through GPUI, reads all three from the Linux settings portal. The `windows` 0.62 crate, already locked with `UI_ViewManagement` enabled, reads them on Windows.
- On Linux, `.SystemUIFont` resolves to the literal family "IBM Plex Sans" (`gpui-pre-linux` `src/linux/platform.rs:130`). When it is missing, GPUI's fallback stack resolves every requested weight to the fallback family's regular face (`gpui-pre` `src/text_system.rs:261-273`, `:579-614`, `:1160-1167`). This is why semibold text renders regular in the Linux captures.

## Decision

1. **Window identity.** The main window has the title "Einfach Mail" and the application id `einfach-mail` on every platform.
2. **Windows chrome.** On Windows, the app hides the system caption and draws GPUI Kit's `TitleBar` as the first row of the workspace, with the app name. Drag, double-click maximize, the system menu, snap layouts, resize, shadow, and rounded corners stay with Windows through `WindowControlArea` and DWM. Its colours come from the theme.
3. **Linux chrome.** On Linux, the app keeps requesting server-side decorations, as ADR 0059 decided. When the compositor answers with client-side decorations, the app draws the same `TitleBar`, with the theme's `window.radius` of 8 px for the frame. The app never requests client-side decorations on Linux. One rule decides both cases: the app draws its title bar when the platform caption is hidden or the window is client-decorated.
4. **Materials.** On Windows 11 build 22621 or later, the window uses the Mica backdrop. Only the title bar and the navigation pane are transparent over it; message, calendar, and settings panes stay opaque. Earlier Windows builds and Linux keep the current background. Linux translucency stays the ADR 0059 opt-in under issue #274. If the owner's look on Windows shows no visible Mica, the app keeps the opaque surfaces.
5. **System preferences.** The desktop owns one `SystemLook` snapshot of accent colour, high contrast, reduced motion, and the desktop UI font, read before the window opens and again on each change. Linux reads it through `ashpd`'s settings portal. Windows reads it through `windows` `UISettings` and `AccessibilitySettings`. Values are validated at that boundary, are never logged, and a missing answer keeps the app's default. Reduced motion sets GPUI's `reduce_motion`. Accent replaces the primary, info, and selected-row colours after a contrast check, with the Einfach blue as fallback. The focus ring keeps its own colour. High contrast selects Einfach high-contrast variants.
6. **Theme structure.** `einfach-fluent.json` stays the only source of raw colours and the app's one identity. It gains backdrop and high-contrast variants as overlays that list only the keys they change. `components/theme.rs` installs base, variant, and validated system overlays into `Theme.light_theme` and `Theme.dark_theme` at startup and on each `SystemLook` change, then calls `Theme::change`. It remains the only writer of theme state. This replaces ADR 0066's "installs both themes once".
7. **Typography.** The theme's `font.family` is a concrete installed family chosen at startup: on Windows 11, Segoe UI Variable Text when installed, otherwise the system UI font; on Linux, the desktop font from the settings portal, then an installed family with static weights, then `.SystemUIFont`. The 16 px base and rem scale from ADR 0066 stay.
8. **Platform code boundary.** `#[cfg(target_os = ...)]` code is limited to the two `SystemLook` readers and the Windows build check for Mica. Every other choice uses `cfg!` or runtime values and is tested on Linux. The readers are part of the `ui-preview` build and live outside `apps/desktop/src/platform/`.

This ADR amends ADR 0059's "Use the system title bar on Linux" to "use the system title bar on Linux whenever the compositor provides one", and its material bullet to name Mica as the Windows material. The rest of ADR 0059 stands. It amends ADR 0066's single installation at startup as described in item 6; the rest of ADR 0066 stands.

## Alternatives

| Option | Benefits | Costs and risks |
| --- | --- | --- |
| GPUI Kit `TitleBar`, server decorations on Linux where granted, one injected theme, `ashpd` and `windows` readers (chosen) | Reuses the component and the libraries already in the graph. Linux desktops keep their own title bars. The app looks like itself everywhere. No lockfile package is added. | Caption buttons are not exactly Windows 11's. Two small readers carry platform code, and the Windows one is only compiled by the Windows preview workflow. Mica visibility is unproven. |
| Keep the system caption on Windows | No new chrome code. | No Mica behind the caption, an empty title, and a caption that ignores the in-app appearance. |
| Client-side decorations on every Linux desktop | One look on all desktops. | Breaks KDE, tiling compositors, and window rules. ADR 0059 rejected it. |
| `native-theme` or `native-theme-gpui` 0.7.0 | One crate for accent, contrast, motion, fonts, and presets. | `native-theme-gpui` requires GPUI Kit 0.8 and would add a second GPUI Kit. `native-theme` is a second owner of theme values with fast-moving minor versions. See the design's rejected alternatives. |
| `system-theme` 0.3.0 | Small, neutral reader. | No reduced motion or font, no release since 2026-01-21, no declared MSRV. |
| Separate Windows 11, Adwaita, and Breeze presets | Each platform's own colours. | Three palettes to test, and the product loses its identity. |

## Consequences

- The workspace renders a title-bar row on Windows and on client-decorated Linux windows. Layouts lose 34 px of height there, and the minimum window height must account for it.
- On GNOME Wayland, the window gains a title bar it does not have today.
- The theme install function takes inputs and runs more than once. The exact-value and contrast tests pin the output for an empty snapshot and for each variant, so injected values cannot drift the default palette.
- The desktop crate gains target-specific direct dependencies on `ashpd` 0.13 (Linux) and `windows` 0.62 and `windows-version` 0.1 (Windows), with the features already in the lockfile. `ashpd` must keep `default-features = false` with `async-io`, because GPUI uses `async-io` and `ashpd` refuses both runtimes at once. `deny.toml` checks only Linux targets, so the Windows entries need a Windows target there before `cargo deny check bans` covers them.
- Variable-only Linux UI fonts still render one weight, because GPUI's Linux text system loads faces at regular weight. Report this upstream. Unread rows keep their dot and accessible description.
- `TitleBar` ignores the Linux button layout, so GNOME shows minimize and maximize beside close. Ask GPUI Kit upstream before building app-owned controls.
- Verify with headless tests and Linux screenshots for everything platform-neutral, with a real GNOME Wayland session for the client-side title bar, and with the owner's look at the Windows preview for every Windows behaviour. Until the owner has looked, claim only that the Windows code compiles in the preview workflow. Implementation units N1 to N11 are in [the native look design](../design/native-look.md#unit-map).

## Sources

- [Native look design](../design/native-look.md), which holds the file and line evidence
- [ADR 0059: Use a shared, adaptive mail and calendar workspace](0059-native-mail-and-calendar-workspace.md)
- [ADR 0066: Use a Fluent visual language and a destination switcher](0066-fluent-visual-language-and-navigation.md)
- [ADR 0068: Build the synthetic UI preview for Windows](0068-windows-ui-preview-build.md)
- [Dependency review: native look system preferences](../project/dependency-review.md#native-look-system-preferences--2026-11-14)
- [Microsoft: Mica material](https://learn.microsoft.com/en-us/windows/apps/design/style/mica)
- [Microsoft: Title bar design](https://learn.microsoft.com/en-us/windows/apps/design/basics/titlebar-design)
- [Microsoft: Support snap layouts for desktop apps](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-snap-layout-menu)
- [XDG Desktop Portal Settings interface](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html)
- [Wayland `xdg-decoration` protocol](https://wayland.app/protocols/xdg-decoration-unstable-v1)
