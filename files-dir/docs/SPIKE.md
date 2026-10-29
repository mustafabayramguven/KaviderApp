# M0 — Feasibility spike findings

Question set (brief section 8, M0): can a **Rust-only Slint Android app** prove

- (a) it builds to an APK in a CI-like environment,
- (b) soft-keyboard text input, a password field, and paste,
- (c) bottom navigation with the `material` style,
- (d) decoding a large image with pinch, pan, and double-tap zoom at acceptable smoothness?

Verdict: **PARTIAL / PENDING DEVICE VERIFICATION.** Everything below marked
*verified* was checked against Slint 1.18.1 source and published crate metadata, or
executed locally. Everything marked *unverified* needs one physical device (or an
emulator) run, which this environment cannot provide. No fallback is being adopted
silently — the decision gate is closed only after the device test.

## 1. What is verified

| # | Claim | Evidence |
|---|-------|----------|
| 1 | Slint's Android backend is `android-activity`-based and enabled by the `backend-android-activity-06` feature, which selects `native-activity` + `android-activity` 0.6 | `api/rs/slint/Cargo.toml` (v1.18.1), feature `backend-android-activity-06`; `api/rs/slint/android.rs` module docs |
| 2 | The Android entry point is a plain Rust `fn android_main(app: slint::android::AndroidApp)`, not `extern "C"` | `api/rs/slint/android.rs` (`init`, `init_with_event_listener`, doc example) |
| 3 | The APK library target must be `cdylib`; Slint documents `cargo apk run --target aarch64-linux-android --lib` and `cargo install cargo-apk` | `api/rs/slint/android.rs` module docs; `cargo-apk` README |
| 4 | Built-in widget style `material` exists, is Material-3 based, adapts to light/dark automatically, is selectable at compile time, and `native` resolves to `material` on Android | `docs/.../std-widgets/style.mdx`, `internal/compiler/widgets/material/*` |
| 5 | Password input, a reveal toggle, paste, select-all, copy, cut, undo/redo, and `input-type: password` are all supported by `LineEdit` — no custom widget needed | `std-widgets/views/lineedit.mdx` (`paste()`, `select-all()`, `copy()`); `internal/compiler/widgets/material/lineedit.slint` (visibility / visibility-off icons) |
| 6 | Soft keyboard is driven by Slint's own `InputMethodRequest` path (JNI into a bundled Java helper for the native-activity backend) | `internal/backends/android-activity/androidwindowadapter.rs` (`input_method_request`, `JavaHelper::set_imm_data`, `show_or_hide_soft_input`); `internal/backends/android-activity/java/SlintAndroidJavaHelper.java` |
| 7 | Multi-touch reaches Slint on Android as per-pointer events (`MotionAction::PointerDown`, pointer ids, `InternalEvent::Touch`) | `androidwindowadapter.rs` (`MotionAction::PointedDown` handling, `p.pointer_id()`, `TouchPhase::Started`) |
| 8 | Slint 1.18 has a first-class pinch/rotate element: `ScaleRotateGestureHandler` with `enabled`, `active`, `scale`, `rotation`, `center` and `started`/`updated`/`ended`/`cancelled`, plus `transform-scale` / `transform-rotation` / `transform-origin` for applying it | `tests/cases/elements/scalerotategesturehandler.slint`, `..._nested.slint`; `examples/native-gestures/` |
| 9 | Single-finger pan and double-tap coexist with the pinch recognizer as *sibling* elements (the pattern used by Slint's own `native-gestures` example) | `examples/native-gestures/native-gestures.slint` (`pan-detection := TouchArea` + `gesture := ScaleRotateGestureHandler`) |
| 10 | `TouchArea` provides `clicked`, `double-clicked`, `pointer-event(...)` with `PointerEventKind.{down,up,move,cancel}`, `pressed`, `enabled`, `has-hover` | `tests/cases/elements/toucharea_doubleclick.slint`, `toucharea.slint` |
| 11 | Exact public theming API: `Palette.{background,foreground,alternate-background,alternate-foreground,control-background,control-foreground,accent-background,accent-foreground,selection-background,selection-foreground,border,color-scheme}` and `StyleMetrics.{layout-spacing,layout-padding,dark-color-scheme,default-font-family}` | `internal/compiler/widgets/material/style-base.slint` |
| 12 | Edge-to-edge insets are available to layout: `Window.safe-area-insets.{left,top,right,bottom}` | `examples/native-gestures/native-gestures.slint` |
| 13 | There is **no** built-in bottom `NavigationBar` in the `material` style (39 files: button, checkbox, combobox, datepicker, groupbox, lineedit, menu, progressindicator, radiogroup, scrollview, slider, spinbox, spinner, switch, tableview, tabwidget, textedit, time-picker …) | directory listing of `internal/compiler/widgets/material/` |
| 14 | `Package.metadata.android` schema for `cargo-apk` 0.10.0: `package`, `apk_name`, `build_targets`, `resources`, `strip`, `[.sdk] min_sdk_version`/`target_sdk_version`, `[[.uses_permission]] name`, `[.application] label`/`debuggable` | `cargo-apk` README (`cargo-apk-0.10.0` is the release tag; crate version 0.10.0, MSRV 1.70) |
| 15 | Latest stable Slint is **1.18.1**; latest `cargo-apk` is **0.10.0** (a separate, newer fork `cargo-apk2` 1.5.0 exists and is *not* upstream) | crates.io API |
| 16 | Google's SDK repository offers `platforms;android-36`, `platforms;android-37.0/37.1/37.2` (no plain `android-37`), `build-tools;37.0.0`, and NDK `26.1.10909125` / `27.2.12479018` | `dl.google.com/android/repository/repository2-3.xml` |
| 17 | `actions/checkout@v7.0.1`, `actions/setup-java@v6.1.0`, `actions/upload-artifact@v7.0.2` each declare `using: node24` | each action's `action.yml` at that release tag |
| 18 | Rust 1.99.1 with `edition = "2024"` is available and the pure-Rust viewport math compiles and its test passes locally with `rustc --test` | local run, this workspace (`crates/kavider-core/src/viewport.rs`) |

## 2. What is NOT verified (needs a device)

1. **The APK actually building.** `cargo-apk` 0.10.0 is from late 2023; Android SDK
   platform naming changed since (`android-37.2` style versioned platforms). CI is the
   first real test. If it fails on platform/NDK discovery, the fallbacks are, in order:
   (a) install and build against `platforms;android-36` with NDK r26.1 — already what
   the committed workflow does; (b) raise the Android NDK pin; (c) replace `cargo-apk`
   with `cargo-apk2` or `xbuild` (both documented alternatives, neither Rust-only-breaking).
2. **Pinch-to-zoom on Android specifically.** Slint's own `ScaleRotateGestureHandler`
   is only fed when the platform produces pinch gestures; the Android backend is
   confirmed to forward *raw multi-pointer* `Touch` events (claim 7), but whether the
   core gesture recognizer synthesises `PinchGesture` from those, on the
   `native-activity` backend, is not proven by any source I can read. This is the
   single highest-risk item of M0 and the reason the decision gate stays open.
   Contingency if it fails: two-pointer tracking inside the app via `TouchArea`'s
   `pointer-event` + pointer ids (still pure Rust), or the documented Kotlin
   `ScaleGestureDetector` bridge — which would be a *documented* exception, per brief
   section 3, not a silent one.
3. **Soft keyboard, password field, paste, long-press text handles** — wired per
   claims 5/6, never observed on a device.
4. **`TouchArea` pan coexisting with the gesture recognizer on Android** — proven in
   Slint's example on desktop, unproven on the Android backend.
5. **Smoothness of pan/zoom on a large image** (M0 (d)) — no device, no measurement.
   Note the spike deliberately uses an uncompressed, *un-downscaled* 1600x2400 image
   (a 15 MB ARGB8888 bitmap at 1x) to make the failure case loud; the real reader
   pipeline (downscale, LRU, bounded decode) is M4.
6. **Cold start, memory, APK size, fps** — all measurement items from brief section 7,
   deliberately deferred to M5's performance pass; nothing is claimed here.
7. **Autofill hints** (`autofillHints` for username/password) — Slint's `LineEdit` has
   no per-field autofill attribute; the nearest hook is the platform input-method path
   used for the soft keyboard. Not implemented; recorded in `DECISIONS.md`.
8. **16 KiB page-size alignment / `extractNativeLibs`** and other current Play
   requirements — unverified for `cargo-apk` 0.10.0 output.

## 3. Device test matrix (run this, then the gate closes)

Install `kavider-debug-apk` (Actions artifact) with `adb install -r`, then:

| Step | Expected |
|------|----------|
| Cold start | App opens on the Connect screen, material light or dark matching the system |
| Tap the server field | Soft keyboard appears, typed characters land in the field |
| Long-press the server field | System paste/select handles appear |
| Copy a URL elsewhere, tap "Paste → server" | Field content becomes the pasted URL |
| Type in the password field | Characters are masked; the eye icon reveals/hides them |
| Bottom navigation | Four items (Connect / Reader / Zoom / Log) switch panes; the selected item shows an accent pill; system back gesture leaves the app cleanly from the first pane |
| Reader pane | LTR / RTL / Webtoon buttons change the label; the slider scrubs the page indicator |
| Zoom pane, two fingers | Pinch zooms the image around the gesture centre; the live label shows a changing percentage |
| Zoom pane, one finger drag | Image pans |
| Zoom pane, double tap | View resets to fit (scale 1.0, centred) |
| Rotate the device | Layout reflows; insets stay correct (no content under the status bar or nav bar) |

Report the observed result of each row. Any row that fails is a bug, not a
documentation note.

## 4. Decision gate (brief section 3)

- Slint **satisfies** (a) by construction and (c) by source; (b) and (d) are wired to
  documented, first-party APIs.
- The gate therefore stays **open on one item only**: item 2 of section 2, pinch on
  Android. If the device test shows pinch does not arrive, the resulting fallback is
  proposed *with the device evidence*, not applied silently — as the brief requires.
- No UniFFI + Jetpack Compose fallback is warranted by anything observed so far.

## 5. Sources (all read from upstream at tag `v1.18.1` unless noted)

- `slint-ui/slint` `api/rs/slint/android.rs` — Android entry point, features, cargo-apk usage.
- `slint-ui/slint` `api/rs/slint/Cargo.toml` — feature graph, `compat-1-18`, renderers.
- `slint-ui/slint` `internal/backends/android-activity/{androidwindowadapter.rs,javahelper.rs,java/SlintAndroidJavaHelper.java}` — input, IME, pointer events.
- `slint-ui/slint` `internal/compiler/widgets/material/*` — widget inventory, `lineedit.slint`, `style-base.slint`.
- `slint-ui/slint` `examples/native-gestures/*`, `tests/cases/elements/{scalerotategesturehandler,toucharea_doubleclick}.slint` — gesture + touch API.
- `slint-ui/slint` `docs/astro/src/content/docs/reference/std-widgets/{style.mdx,views/lineedit.mdx}` — style selection, LineEdit API.
- `rust-mobile/cargo-apk` README (crate 0.10.0, tag `cargo-apk-0.10.0`) — `[package.metadata.android]` schema.
- `crates.io` API — slint 1.18.1, cargo-apk 0.10.0, cargo-apk2 1.5.0, i-slint-backend-testing 1.18.1.
- `dl.google.com/android/repository/repository2-3.xml` — SDK platform / build-tools / NDK revisions.
- `actions/checkout@v7.0.1`, `actions/setup-java@v6.1.0`, `actions/upload-artifact@v7.0.2` `action.yml` — Node 24.
- `docs.slint.dev` Android page — supported deployment path (cargo-apk / xbuild).
