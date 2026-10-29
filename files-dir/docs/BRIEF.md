# BRIEF: "Kavider" — a native, Rust-only Kavita client for Android

You are a senior Rust and Android engineer working autonomously in this repository. Build a native Android client for **Kavita** (the self-hosted manga/comic/book server) from scratch, in Rust. The result must be a clean, fast, minimal client that is trivial to connect to a server. Working name: **Kavider**. Application id: `app.kavider.client` unless I say otherwise.

## 0. How to work

- Read this whole brief first.
- If `docs/kavita-openapi.json` exists, treat it as the single source of truth for the Kavita API. If it does not exist, ask me for it once (I will export it from the server's swagger endpoint). Do not guess endpoints, field names, or auth flows from memory: Kavita's API changes between versions.
- Work milestone by milestone (section 8). Finish and verify one milestone, commit, report, then continue. Use small, reviewable commits with clear messages. The working branch must build and pass tests after every commit.
- Make reasonable decisions yourself and record each in `docs/DECISIONS.md` (context, choice, alternatives, why: a few lines each). Ask me only when you are blocked on something only I can provide (server URL and credentials, signing keys, app id).
- You cannot run a physical device. Run everything you can (build, `cargo test`, clippy, desktop run, emulator if available) and state explicitly what is verified and what is NOT verified on a real device. Never claim on-device behavior you did not observe.
- Stay in scope. No telemetry, analytics, ads, or crash-reporting SDKs.

## 1. Product goals (in priority order)

1. Connecting to a server takes under 30 seconds and never needs documentation.
2. Browsing and reading manga/comics is smooth.
3. Calm, clean, Material-3-style UI; small APK; low memory and battery use.
4. Clean, boring, well-tested Rust code that is easy to extend.

## 2. Scope

**v1 IN:** onboarding/login; Home (continue reading / on deck, recently updated, recently added); Libraries; series list (grid, sort, simple filter); series detail (cover, summary, volumes/chapters, read state, "continue" button); search; reader for image-based formats (archives that Kavita serves as page images); reading-progress sync; settings (server, theme, reader defaults, cache size and clear, logout); light/dark following the system.

**v1 OUT (design so they can be added later, do not implement):** EPUB and PDF readers, Kavita+ features, user management/admin, editing collections or reading lists, OPDS, tablet-specific layouts, casting, widgets.

**Later (M6, only after M5 is reviewed):** offline browsing, chapter downloads, queued progress sync. Keep the data layer ready for this (repository pattern, SQLite cache) but do not build it earlier.

## 3. Technology decisions

- **Language:** latest stable Rust, Cargo workspace, edition 2024 if the toolchain supports it. All app code is Rust. Allowed non-Rust files: minimal Android build glue (manifest, Gradle or cargo-ndk config, icons, resources) and any tiny Java/Kotlin support library that a crate requires (for example for the platform TLS verifier). Document each exception.
- **UI:** **Slint** with its built-in `material` style (light and dark) and the Android backend based on `android-activity`. Verify against the current Slint docs. UI layout lives in `.slint` files; all logic lives in Rust.
- **Decision gate:** run the feasibility spike (M0) first. If Slint cannot meet the requirements (text input with soft keyboard, material look, smooth zoomable reader), STOP and propose the fallback to me with evidence. The fallback is a Rust core exposed through UniFFI plus a thin Jetpack Compose UI. Never switch silently.
- **Async/HTTP:** tokio + reqwest (HTTP/2, gzip/brotli). TLS must work on Android with rustls (no OpenSSL): use `rustls-platform-verifier` so the Android trust store, including user-installed CAs, is honored, or a documented alternative.
- **Models:** serde. Write or generate typed DTOs from the OpenAPI document, deserialize only the fields we use, and tolerate unknown or missing fields (`#[serde(default)]`), because servers run different versions.
- **Storage:** SQLite via rusqlite (bundled) for the metadata cache; app-private storage for settings and session. Never store the password: store only the refresh token / API key. Note Android Keystore wrapping as a possible hardening in the docs.
- **Images:** decode off the UI thread, downscale to display size before handing pixels to Slint, bounded in-memory LRU, size-capped disk cache, cancel stale requests.
- **Errors and logging:** `thiserror` for error types, `tracing` with a logcat layer. No `unwrap`/`expect` outside tests. Never log passwords, tokens, or URLs containing API keys.
- **Android:** minSdk 26; targetSdk = latest stable (verify); arm64-v8a primary, armeabi-v7a optional, x86_64 for the emulator. Edge-to-edge with correct system insets; system back gesture works everywhere.

## 4. Architecture

Cargo workspace with three crates:

- `crates/kavita-api`: typed async Kavita client. No UI or Android dependencies. Fully testable with a mock server.
- `crates/kavider-core`: domain models, repositories (cache-then-network), session and settings, image pipeline, progress tracking. No UI dependencies. Builds and tests on desktop.
- `crates/kavider-app`: Slint UI, view models, Android entry point (cdylib) and a desktop binary for fast iteration (`cargo run -p kavider-app`).

Rules:
- Unidirectional data flow: the UI emits intents, the core updates state, the UI renders state.
- No networking or business logic in UI code.
- Depend on traits at boundaries (`KavitaApi`, `Store`, `Clock`, and similar) so the core is tested with fakes.
- Keep dependencies minimal; justify each non-trivial crate in `docs/DECISIONS.md`.

## 5. Connecting to a server (the signature feature)

One onboarding flow, three steps at most.

1. **Server address.** Accept whatever people type: `kavita.example.com`, `192.168.1.20:5000`, `https://example.com/kavita/` (reverse-proxy subpath), trailing slashes, surrounding whitespace, uppercase, a pasted URL that includes a path such as `/login` or `/api/...`, IPv6. Normalize to a canonical base URL. If no scheme is given, try https and http (http first for private IPs, loopback, and `.local` names) with short timeouts (about 5 s) and fast failure. Probe with a lightweight unauthenticated endpoint (find it in the OpenAPI document).
2. **Credentials.** Username and password with a visibility toggle; paste works; autofill hints set. A collapsed "Advanced" section offers API-key login if the OpenAPI document shows a supported flow.
3. **Done.** Land on Home.

Friendly diagnostics instead of raw errors. Cover at least: DNS failure, connection refused, timeout, TLS problem, got HTML instead of Kavita (wrong port or proxy path), server too old or too new, 401 wrong credentials, 403, rate limited. Each message says what happened and the likely fix in one sentence.

TLS and cleartext:
- Self-signed or private CA: on a TLS failure show the certificate SHA-256 fingerprint and let the user explicitly trust it. Pin that fingerprint for that server only, and provide a clear way to remove it.
- Plain `http://` to a non-private host is allowed but shows a clear warning once.
- Networking is native Rust, so Android's own cleartext policy does not apply; these warnings are our responsibility.

Session:
- Automatic token refresh: retry once on 401 and de-duplicate concurrent refreshes.
- On unrecoverable auth failure, return to login with the server address prefilled.
- Remember the last server. The data model supports multiple servers; UI for one is enough in v1.
- `KAVITA_URL`, `KAVITA_USER`, `KAVITA_PASS` environment variables are accepted for tests and dev on desktop and CI only. Never hardcode servers or credentials.

## 6. UI / UX

- **Material 3 look:** seed-color scheme, light and dark following the system (wallpaper dynamic color only if it is cheap), 48dp touch targets, M3 type scale, top app bar, bottom NavigationBar (Home, Libraries, Search, Settings), cards, adaptive cover grid (column count from width), skeleton placeholders, pull to refresh, empty and error states with a retry action, subtle motion that respects the system "remove animations" setting.
- **Clean means little chrome:** covers first, text second. No clutter, no dialogs unless needed. One consistent spacing scale (4/8/12/16/24).
- **Reader:**
  - Modes: paged left-to-right, paged right-to-left (manga), vertical continuous (webtoon).
  - Tap zones: left and right go to previous and next (respecting reading direction), center toggles the overlay.
  - Pinch zoom, double-tap zoom, pan.
  - Fit width, fit height, fit screen.
  - Immersive mode, keep screen on, page slider and indicator.
  - Remember the mode per series.
  - Prefetch the next 2 to 3 pages and the previous 1.
  - Save progress debounced and on exit.
- **Strings:** externalized, English and Turkish.

## 7. Performance and size budgets (measure and report)

- Cold start to first frame: under 1 s on a mid-range device. Report emulator numbers and flag them as approximate.
- Grids: no jank at 60 fps while scrolling 500+ items. Load covers lazily at display size, recycle items, never decode full-size covers.
- Reader: the next page appears without a blank flash on a normal connection. Decoded-page memory is bounded (configurable, default at most 96 MB). Decoding never runs on the UI thread.
- Network: HTTP/2 with compression, request de-duplication, cancel on navigation, conditional requests (ETag / If-Modified-Since) where the server supports them, no background polling.
- Release profile: `lto`, `codegen-units = 1`, `panic = "abort"`, `strip`. Consider `opt-level = 3` only for image-decoding crates and `s` for the rest. Measure the arm64 APK size; target under 20 MB and explain any deviation.
- Permissions: only `INTERNET` and `ACCESS_NETWORK_STATE`.

## 8. Milestones

- **M0, feasibility spike (short):** a minimal Slint Android app proving (a) it builds to an APK in a CI-like environment, (b) soft-keyboard text input, a password field, and paste, (c) bottom navigation with the material style, (d) decoding a large image with pinch, pan, and double-tap zoom at acceptable smoothness, as far as you can verify. Write findings to `docs/SPIKE.md` and apply the decision gate from section 3.
- **M1:** `kavita-api` crate with tests (mock-server fixtures) and `examples/cli.rs` that logs in and lists libraries against a real server (env vars). Table-driven URL-normalization tests with at least 40 cases.
- **M2:** app skeleton for desktop and Android, onboarding/connect flow, session handling, settings storage, CI that builds an APK.
- **M3:** Home, Libraries, series list and detail, search, metadata cache.
- **M4:** reader and progress sync.
- **M5:** settings polish, theming, strings, performance pass with measurements, README, signed release pipeline.
- **M6:** offline (only after M5 is reviewed).

After each milestone, stop and give me a short report: what works, how each thing was verified, what was not verified, risks, and what comes next. I will test the APK on a real device between milestones.

## 9. Quality gates (every milestone)

- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` all pass.
- Unit tests cover: URL normalization; the API client against a mock server (success, 401 then refresh then retry, 5xx, timeout, malformed JSON, unknown fields); token-refresh de-duplication; the LRU image cache; reader page and prefetch logic; progress debouncing.
- Optional integration tests against a real Kavita (docker compose file plus `KAVITA_URL`), skipped when the variable is not set.
- No `unwrap`, `expect`, or `panic!` in non-test code. No `unsafe` outside the Android glue, and justify any that remains.
- `cargo deny` or `cargo audit` is clean or the findings are documented.
- Docs: `README.md` (build, run on desktop, run on a device, CI), `docs/DECISIONS.md`, a short `docs/ARCHITECTURE.md`.

## 10. CI and release

- GitHub Actions on ubuntu-latest: fmt, clippy, tests, debug APK build, APK uploaded as an artifact. Use action versions that run on Node 24 (check the current majors of checkout, setup-java, and upload-artifact).
- Release workflow triggered by `v*` tags:
  - Build a signed release APK using the secrets `KEYSTORE_BASE64`, `KEYSTORE_PASSWORD`, `KEY_ALIAS`, `KEY_PASSWORD`.
  - Verify the signature with `apksigner`.
  - Attach the APK and a SHA-256 file to a GitHub Release with `gh release create`.
  - Derive the version name from the tag and the version code from the run number.
  - Fail loudly if the secrets are missing.
- Never commit keystores or secrets. If Gradle is used, commit the Gradle wrapper.

## 11. Definition of done for v1 (end of M5)

A person with a Kavita server can install the APK, type their server address and credentials, see their libraries, open a series, read a chapter in LTR, RTL, or webtoon mode with zoom, close the app, and reopen it to find the same progress, also visible on the server, all without reading documentation. The final report includes measured numbers for startup time, memory use, and APK size.
