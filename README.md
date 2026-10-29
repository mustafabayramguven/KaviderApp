# Kavider

A native, Rust-only client for [Kavita](https://www.kavitareader.com/) (self-hosted
manga / comic / book server), for Android. UI in Slint with the built-in `material`
style; everything else is Rust.

Status: **M0 feasibility spike only.** Nothing here is a working client yet — see
`docs/SPIKE.md` for what M0 proves and what it does not.

## Layout

    Cargo.toml                     workspace
    rust-toolchain.toml            pinned toolchain + Android targets
    crates/kavita-api/             typed async Kavita client (M1)
    crates/kavider-core/           domain logic, no UI (viewport math so far)
    crates/kavider-app/            Slint UI, Android entry point, desktop binary
    crates/kavider-app/ui/         .slint layout files
    docs/                          brief, spike findings, decisions, architecture
    .github/workflows/build.yml    fmt + clippy + test + debug APK artifact

## Build everything in CI (no local toolchain required)

Push the repository to GitHub. `.github/workflows/build.yml` runs on every push and
upload it produces:

- `checks` — `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`.
- `apk` — installs the Android SDK components and `cargo-apk`, builds the
  `aarch64-linux-android` debug APK, and uploads it as the `kavider-debug-apk` artifact.

Download the APK from the workflow run summary (Actions → run → Artifacts).

## Run on desktop (for fast UI iteration)

Requires a Rust toolchain and a GPU/display; no Android tooling:

    cargo run -p kavider-app

## Build the APK locally (optional, requires the Android SDK + NDK)

    cargo install cargo-apk --version 0.10.0 --locked
    export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/26.1.10909125"
    cargo apk build --package kavider-app --lib --target aarch64-linux-android

## Run on a device

    adb install -r target/debug/apk/kavider.apk

Then work through the M0 test matrix in `docs/SPIKE.md` and report back.

## Documentation

- `docs/BRIEF.md` — the full product brief this repository is being built against.
- `docs/SPIKE.md` — M0 findings, the decision gate, and what still needs a device.
- `docs/DECISIONS.md` — every non-obvious technical choice with alternatives.
- `docs/ARCHITECTURE.md` — crate boundaries and data flow.

## Quality gates

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

No `unwrap`, `expect` or `panic!` outside tests. No telemetry, analytics, ads or
crash-reporting SDKs — ever.
