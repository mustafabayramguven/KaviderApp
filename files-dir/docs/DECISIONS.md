# Decisions

Every non-obvious choice, with context, the choice, the alternatives, and why.
Append; never rewrite history.

## D-001 — UI toolkit: Slint (built-in `material` style)

- **Context:** brief section 3 requires Slint with its `material` style and the
  `android-activity` backend, with a decision gate after M0.
- **Choice:** Slint 1.18.1, `backend-android-activity-06` feature, `material` style
  forced at compile time via `slint_build::CompilerConfiguration::with_style("material")`.
- **Alternatives:** Fluent (Slint's new default) — not Material-3; the standalone
  `ui-libraries/material` component library — richer M3 set (NavigationBar, cards,
  sheets) but a second dependency with its own release cadence and no built-in
  light/dark system following.
- **Why:** the brief asks for Material 3 and for light/dark following the system; the
  built-in `material` style gives both with zero extra dependencies, and `native`
  already resolves to `material` on Android. Revisit the standalone library in M3+M5
  if bottom navigation, cards or sheets need more M3 fidelity than hand-written
  layout provides.

## D-002 — Pin: Slint `=1.18.1`, single version across the workspace

- **Context:** the Android backend, the material style and `ScaleRotateGestureHandler`
  are all version-sensitive; Slint publishes a new minor every few months.
- **Choice:** exact pin `=1.18.1` in `[workspace.dependencies]`; a bump is a
  deliberate commit that re-runs the M0 spike.
- **Alternatives:** caret range `1.18` (silently picks up 1.19+ behaviour changes);
  tracking `master` (unreviewable churn).
- **Why:** M0 findings are only meaningful against a version we can name.

## D-003 — Renderers: Skia on Android, FemtoVG on desktop

- **Context:** Slint's `Cargo.toml` disables FemtoVG on Android ("it doesn't compile
  without `RUST_FONTCONFIG_DLOPEN=on`, and even then couldn't load fonts"), and the
  Android backend source imports `SkiaRenderer`.
- **Choice:** `target.'cfg(target_os = "android")'` pulls
  `renderer-skia` + `backend-android-activity-06`; other targets pull
  `backend-winit` + `renderer-femtovg`.
- **Alternatives:** software renderer everywhere (slow for the reader, no GPU path);
  WGPU/Vello (experimental here, extra dependency weight).
- **Why:** matches how Slint itself builds for Android, and keeps desktop iteration
  light. Exact feature names must be confirmed by the first green CI run.

## D-004 — Widgets: built-in `material` only; bottom navigation hand-built

- **Context:** the built-in `material` style ships 39 files and **no** `NavigationBar`
  (verified by directory listing); the brief wants an M3 bottom `NavigationBar`.
- **Choice:** compose the bottom bar from `Rectangle` + `TouchArea` + `Palette`
  (accent pill on the selected item), reading colours from `Palette` so light/dark
  still follows the system.
- **Alternatives:** add `slint-ui/ui-libraries/material` for a real `NavigationBar`;
  use the `material` style's `TabWidget` (a top tab bar, wrong pattern).
- **Why:** M0 must prove the *feasibility of the stack*, not spend its budget on a
  component library; the custom bar is ~30 lines, uses only documented APIs, and can be
  replaced later behind the same layout. Recorded here because a reviewer will notice
  the divergence from "M3 NavigationBar" in the brief.

## D-005 — Gesture approach: Slint's `ScaleRotateGestureHandler` + sibling `TouchArea`

- **Context:** pinch/pan/double-tap is M0 item (d). Slint 1.18 added
  `ScaleRotateGestureHandler`; community reports (2024-2025) that multi-touch did not
  reach Slint on Android predate it, and a third-party note claims Android needs a
  Kotlin `ScaleGestureDetector` bridge.
- **Choice:** use the first-party element, structured exactly like Slint's own
  `examples/native-gestures`: a `TouchArea` sibling for single-finger pan and
  double-tap, the gesture handler for pinch. No Kotlin, no JNI in our code.
- **Alternatives:** hand-rolled two-pointer tracking from `TouchArea.pointer-event`
  (kept as contingency); Kotlin `ScaleGestureDetector` via `cargo-apk2`
  (a non-Rust exception the brief allows only if documented and justified).
- **Why:** prefer the supported first-party API and a pattern upstream already ships;
  keep the Kotlin path in reserve until a device proves it necessary. **Status: pinch
  on Android unverified — see `docs/SPIKE.md` section 2, item 2.**

## D-006 — Zoom/pan state lives in Rust, geometry is rendered by Slint

- **Context:** brief section 4 forbids business logic in UI code and requires
  unidirectional data flow.
- **Choice:** `kavider-core::viewport::Viewport` (pure, unit-tested: `zoom_at`,
  `pan_to`, `reset`) is the single source of truth. The UI emits intents
  (`zoom-by`, `pan-to`, `reset-view`); Rust mutates the viewport and pushes
  `zoom-scale`, `pan-x`, `pan-y` back for rendering. During a live pinch the UI shows
  `self.scale` of the in-flight gesture as a preview only; the committed value always
  comes from Rust.
- **Alternatives:** keep the transform purely in `.slint` (fastest to write, but takes
  the reader's core maths out of Rust and out of reach of tests).
- **Why:** this is the smallest possible end-to-end proof of the intended architecture,
  and it makes M0 testable on the host with `cargo test` even though no Android input
  can be simulated here.

## D-007 — Packaging: upstream `cargo-apk` 0.10.0, NativeActivity, no Gradle

- **Context:** brief section 3 allows minimal Android build glue and explicitly suggests
  `cargo-ndk`/Gradle glue; section 10 wants a debug APK artifact from Actions.
- **Choice:** `cargo-apk` 0.10.0 (upstream, `rust-mobile`), all manifest settings in
  `[package.metadata.android]`, no Gradle, no committed wrapper, no Kotlin.
- **Alternatives:** `cargo-apk2` (active fork, adds Kotlin sources support — only
  needed if D-005's contingency is used); `xbuild`; Gradle + `cargo-ndk` (most
  machinery, most moving parts).
- **Why:** smallest possible build surface that still produces a signed debug APK, and
  it is the path Slint documents. Risk — 0.10.0 predates versioned Android platforms —
  is recorded in `SPIKE.md` section 2 with an ordered fallback list. `cargo-apk2` is
  *not* described anywhere in this repo as an upstream release; it is a fork.

## D-008 — Android SDK / NDK pinning in CI

- **Context:** `platforms;android-37` does not exist in Google's repository — the 37
  line is `android-37.0/37.1/37.2`. `targetSdk` is an integer regardless.
- **Choice:** build against `platforms;android-36` + `build-tools;36.0.0` +
  NDK `26.1.10909125`; `min_sdk_version = 26`, `target_sdk_version = 36` in the
  manifest metadata. NDK r26 is era-matched with `cargo-apk` 0.10.0; r27.2.12479018 is
  the documented alternative if r26 is rejected.
- **Alternatives:** compile against `android-37.2` (decimal platform directory that a
  2023-era `cargo-apk` may not discover), or fine-grained `android-37.0/37.1`.
- **Why:** the Google Play target requirement since 2026-08-31 is API 36, so 36 is both
  the lowest-risk and a currently compliant choice. Android 17 / API 37 is a
  deliberate later bump once the toolchain handshake is proven. Do **not** write
  `sdkmanager "platforms;android-37"` — it fails.

## D-009 — CI: pinned Node-24 action majors, host checks separate from the APK job

- **Context:** brief section 10 asks for action versions that run on Node 24, and for
  fmt/clippy/tests/APK.
- **Choice:** two jobs in one workflow. `checks` = fmt + clippy + tests on the host.
  `apk` = JDK 21 (Temurin), Android SDK components, `cargo-apk`, build + upload
  artifact. `actions/checkout@v7.0.1`, `actions/setup-java@v6.1.0`,
  `actions/upload-artifact@v7.0.2` — all verified `using: node24`.
- **Alternatives:** one fat job (slower feedback, an Android failure hides a lint
  failure); floating majors (`@v7`) — Node 24 guaranteed only by the tag, so the exact
  release tag is pinned.
- **Why:** fast signal on pure-Rust regressions; the Android job is allowed to be the
  slow, environment-heavy one. Full commit-SHA pinning is a deliberate follow-up
  (supply-chain hardening) — noted, not claimed.

## D-010 — No clipboard/autofill/IME-hint glue in M0

- **Context:** the brief wants paste support and Android autofill hints.
- **Choice:** M0 tests paste through documented `LineEdit.paste()` /
  `select-all()` / `copy()` plus the platform's own long-press affordance. Autofill
  hints are **not** implemented.
- **Alternatives:** reach into Slint internals for the Android `InputMethodManager`
  state; add a Java helper (a documented non-Rust exception).
- **Why:** the first is not required to answer M0's question, and the second is exactly
  the kind of Android glue the brief wants minimised and documented. Autofill is
  recorded as a gap for M2 (`kavider-app` connect screen) rather than silently skipped.

## D-011 — Logging and error handling in M0

- **Context:** brief section 3 wants `thiserror` + `tracing` with a logcat layer, and
  no `unwrap`/`expect` outside tests.
- **Choice:** M0 adds no logging dependency; error paths use `if let Err(..)` and
  `eprintln!`, and every fallible call is propagated with `?` or matched.
- **Alternatives:** introduce `tracing` + `tracing-android` now.
- **Why:** M0's question is feasibility, and adding two dependencies whose logcat
  behaviour we cannot observe on this machine would be unverifiable ballast. M2 adds
  `tracing` + a logcat layer together with the first real error surface (connect flow).

## D-012 — Build target set: `aarch64-linux-android` in CI, `x86_64` available

- **Context:** brief section 3 wants arm64 primary, x86_64 for the emulator;
  `rust-toolchain.toml` installs both targets, `build_targets` lists both.
- **Choice:** CI builds `aarch64-linux-android` only (the artifact people install);
  the emulator ABI stays configured for local/emulator use.
- **Alternatives:** build both in CI (roughly doubles the APK job).
- **Why:** the M0 artifact is for a real device; adding the emulator ABI is a one-flag
  change when an emulator run is actually wanted.
