# Architecture

Three crates, one direction of data flow. Only the third knows that Android or a
window exists.

    kavita-api      typed async Kavita client. No UI, no Android. Mock-server testable.
          |
    kavider-core    domain models, repositories (cache-then-network), session, settings,
                    image pipeline, progress, viewport maths. No UI dependencies; builds
                    and tests on the desktop.
          |
    kavider-app     Slint UI + view-models + Android entry point (cdylib) and a desktop
                    binary. No networking, no business logic.

Rules that bind all three:

- The UI emits intents; the core updates state; the UI renders state. Nothing flows back
  the other way except through the core.
- Boundaries are traits (`KavitaApi`, `Store`, `Clock`, …) so the core is tested with
  fakes and never with a live server.
- No `unwrap`, `expect` or `panic!` outside tests; no `unsafe` outside the Android glue
  (and there is none in M0 — the `#[unsafe(no_mangle)]` attribute on `android_main` is
  an attribute, not an `unsafe` block).
- Dependencies stay minimal; each non-trivial one is justified in `docs/DECISIONS.md`.

## What exists after M0

| Crate | Contents |
|-------|----------|
| `kavita-api` | placeholder; M1 fills it (typed DTOs generated from the Kavita OpenAPI document, auth, refresh de-duplication, mock-server fixtures, `examples/cli.rs`) |
| `kavider-core` | `viewport::Viewport` — pure zoom/pan geometry, unit-tested (`zoom_at`, `pan_to`, `reset`) |
| `kavider-app` | `run()` wiring the Slint window, `android_main` for the Android entry point, `main.rs` for desktop, `ui/main.slint` with the M0 spike layout |

## M0 data flow (the pattern the rest of the app will follow)

    TouchArea / ScaleRotateGestureHandler  (UI, ui/main.slint)
            |  zoom-by(factor, anchor) / pan-to(x, y) / reset-view()
            v
    kavider-app::wire  (view-model glue, src/lib.rs)
            |  mutates
            v
    kavider-core::viewport::Viewport   (single source of truth, unit-tested)
            |  pushes scale / pan back
            v
    MainWindow.zoom-scale / pan-x / pan-y  -> rendered by the Rectangle transform

The in-flight pinch scale is shown as a *preview* only; the committed viewport value
always comes from the core.

## Planned shape (M1–M5, not built yet)

- `kavita-api`: `reqwest` (HTTP/2, gzip/brotli) + `rustls` with
  `rustls-platform-verifier` so the Android trust store (including user-installed CAs)
  is honoured; `serde` DTOs that tolerate unknown and missing fields.
- `kavider-core`: repositories with cache-then-network over `rusqlite` (bundled), a
  bounded LRU of decoded pages, disk cache with a size cap, debounced progress writer,
  session with single-flight token refresh.
- `kavider-app`: view-models per screen (onboarding, Home, Libraries, series list/detail,
  search, reader, settings), strings in English and Turkish, M3 spacing scale
  (4/8/12/16/24), skeleton/empty/error states, and the reader's paged/vertical modes.
- Secrets: never the password — refresh token or API key only, in app-private storage;
  Android Keystore wrapping documented as hardening in M5.

## Build and packaging

`cargo-apk` 0.10.0 + NativeActivity, no Gradle. The library target is a `cdylib` named
`kavider_app`; `kavider-app/src/main.rs` is the desktop binary that calls the same
`run()`. The `material` style is selected at compile time in `build.rs`, so desktop and
Android render identically.

See `docs/DECISIONS.md` D-007/D-008 for the packaging and SDK-pinning rationale and
`docs/SPIKE.md` section 2 for the open risks.
