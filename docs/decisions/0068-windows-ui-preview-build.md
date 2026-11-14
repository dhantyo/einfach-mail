# ADR 0068: Build the synthetic UI preview for Windows

- Status: Proposed; amends the platform scope of [ADR 0010](0010-linux-first-development.md); decided on a fork and not yet accepted by the upstream repository owner
- Date: 2026-11-14
- Owners: Einfach Mail maintainers

## Context

[ADR 0010](0010-linux-first-development.md) makes Linux the sole development and validation target and moves Windows work to M5. The person who now leads GUI work uses Windows 11 and cannot run the app. [ADR 0066](0066-fluent-visual-language-and-navigation.md) gives the workspace a Windows 11 style, so a Windows reviewer is needed for GUI work.

The `ui-preview` feature runs the real views with synthetic in-memory data. It uses no sign-in, credential store, SQLite cache, or user preferences file. The code paths that are Linux-specific are compiled out of it:

- `apps/desktop/src/main.rs` drops the `platform` module, so the Linux single-instance lock in `apps/desktop/src/platform/linux/single_instance.rs` is not built.
- `apps/desktop/src/app/bootstrap.rs` gates the providers, the SQLCipher cache, the XDG settings store, and the Linux translucency check behind `not(feature = "ui-preview")`.
- `apps/desktop/src/features/settings/appearance.rs` gates its XDG path behind the same condition.
- `crates/persistence` compiles to an empty crate on targets other than Linux.

Evidence gathered on Linux on 2026-11-14:

- `cargo check --locked --package einfach-mail-desktop --features ui-preview --bin einfach-mail --target x86_64-pc-windows-msvc` stops in the C build scripts of `aws-lc-sys` (through `reqwest` and `rustls`) and `openssl-sys` (through the vendored SQLCipher). A Linux host has no MSVC compiler or Windows Perl. The desktop crate still depends on the providers and persistence when `ui-preview` is enabled, so these build scripts run even though the preview never calls them.
- With `--keep-going`, `gpui-pre-windows`, `gpui-kit`, the `keyring` Windows store, `sspi`, and `hickory-resolver` type-check for that target. `gpui-pre-windows` and `gpui-fast-windows` are already in `Cargo.lock`.
- In a scratch copy where the four real-path crates were optional, the preview type-checked for `x86_64-pc-windows-msvc` with no errors or warnings.
- Nothing has been built, linked, or launched on Windows.

The repository's own CI uses Linux runners, and its GitHub Actions have no compute quota.

## Decision

Allow a Windows build of the `ui-preview` developer preview now, so that GUI work can be reviewed on Windows 11. Linux remains the only verified target for the real application. Sign-in, credential storage, the encrypted cache, installation, and every other part of the real Windows application stay in M5 under ADR 0010.

`.github/workflows/windows-preview.yml` builds the preview on a GitHub-hosted `windows-latest` runner and uploads `einfach-mail.exe` as a short-lived artifact. It runs on manual dispatch and when the workflow file itself changes. It builds the locked, unmodified dependency graph:

- Vendored OpenSSL for SQLCipher uses the runner's Strawberry Perl, set through `OPENSSL_SRC_PERL`.
- `aws-lc-sys` uses the MSVC compiler. It falls back to the prebuilt NASM objects shipped in the crate when the runner has no NASM.

The build uses the release profile with `CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true`. `apps/desktop/src/main.rs` rejects `ui-preview` builds without debug assertions, and this keeps that guard in place while producing an optimized binary.

The workflow is not a required check and makes no claim about Windows support. It does not change any manifest, the lockfile, or Rust source.

## Alternatives

| Option | Benefits | Costs and risks |
| --- | --- | --- |
| Build the unmodified graph on a Windows runner (chosen) | No manifest or source change. The build also compiles the provider crates for Windows, which M5 needs anyway. | Needs the runner's Perl and MSVC for C dependencies the preview never calls. A runner image change can break the build. |
| Make the providers and persistence optional desktop dependencies | The preview no longer needs a C toolchain or Perl. Type-checking this shape passed in a scratch copy. | Changes the desktop manifest and its feature contract, and every real-app build path needs the new feature. Defer it unless the unmodified build fails. |
| Ask the Windows reviewer to build locally | No CI change. | Needs Visual Studio Build Tools, Strawberry Perl, and a long first build on the reviewer's machine. |
| Wait for M5 | Keeps ADR 0010 unchanged. | The GUI lead cannot see the UI they are responsible for. |

## Consequences

- AGENTS.md, the roadmap, setup, and status record the preview exception. ADR 0010 stays in force for the real application.
- The preview binary links the MSVC runtime dynamically. A Windows 11 machine without the Microsoft Visual C++ redistributable may fail to start it. Zed builds Windows with `+crt-static`, but `openssl-src` 300.6.1 passes no runtime option to the vendored OpenSSL build, so a static runtime could mix C runtimes. This change does not try it.
- The binary is a console-subsystem executable, so a console window opens beside the app.
- `deny.toml` still checks only Linux targets. Adding the Windows target needs a `cargo deny check bans` run, which was not available when this decision was written.
- Verify the decision by running the workflow, downloading the artifact, and launching it on Windows 11 with each preview scenario in [development setup](../project/setup.md). Record the result in [project status](../project/status.md). Until then, claim only that the preview type-checks for Windows.
- If the unmodified build fails in a C dependency, make the real-path crates optional for the preview in a follow-up change.

## Sources

- [ADR 0010: Focus current development on Linux](0010-linux-first-development.md)
- [ADR 0066: Use a Fluent visual language and a destination switcher](0066-fluent-visual-language-and-navigation.md)
- [GPUI Kit research](../research/gpui-kit.md), which records Windows 10 or later with MSVC as a GPUI Kit platform
- `aws-lc-sys` 0.47.0 `builder/main.rs` (`AWS_LC_SYS_PREBUILT_NASM`) and `openssl-src` 300.6.1 `src/lib.rs` (`OPENSSL_SRC_PERL`, MSVC `no-asm` fallback)
- [Zed `.cargo/config.toml`](https://github.com/zed-industries/zed/blob/main/.cargo/config.toml), which sets `+crt-static` for Windows
