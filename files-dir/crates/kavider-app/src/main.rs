//! Desktop entry point. Identical UI to the Android build — this is the fast iteration
//! path (`cargo run -p kavider-app`).

fn main() -> Result<(), slint::PlatformError> {
    kavider_app::run()
}
