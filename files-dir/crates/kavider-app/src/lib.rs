//! Kavider application: Slint UI, view-model glue, and the Android entry point.
//!
//! The UI never owns state. It emits intents (`zoom-by`, `pan-to`, `reset-view`, the
//! reader's direction buttons, …); the core mutates state; the UI renders what the core
//! pushes back. See `docs/ARCHITECTURE.md`.

use std::cell::RefCell;
use std::rc::Rc;

use kavider_core::viewport::Viewport;
use slint::ComponentHandle as _;

slint::include_modules!();

/// Starts the UI on the current thread. Returns when the window closes.
pub fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    wire(&ui);
    ui.run()
}

/// Connects every UI intent to the core, and pushes core state back into the UI.
fn wire(ui: &MainWindow) {
    // The single source of truth for the zoom/pan of the reader surface. It lives in
    // `kavider-core` so it can be unit-tested without a window or a device
    // (crates/kavider-core/src/viewport.rs).
    let viewport = Rc::new(RefCell::new(Viewport::default()));

    let viewport_for_zoom = Rc::clone(&viewport);
    let ui_weak = ui.as_weak();
    ui.on_zoom_by(move |factor, anchor_x, anchor_y| {
        if let Ok(mut viewport) = viewport_for_zoom.try_borrow_mut() {
            viewport.zoom_at(factor, anchor_x, anchor_y);
            push_viewport(&ui_weak, &viewport);
        }
    });

    let viewport_for_pan = Rc::clone(&viewport);
    let ui_weak = ui.as_weak();
    ui.on_pan_to(move |x, y| {
        if let Ok(mut viewport) = viewport_for_pan.try_borrow_mut() {
            viewport.pan_to(x, y);
            push_viewport(&ui_weak, &viewport);
        }
    });

    let viewport_for_reset = Rc::clone(&viewport);
    let ui_weak = ui.as_weak();
    ui.on_reset_view(move || {
        if let Ok(mut viewport) = viewport_for_reset.try_borrow_mut() {
            viewport.reset();
            push_viewport(&ui_weak, &viewport);
        }
    });

    // Reader direction is UI state for now; M4 moves it into per-series settings.
    let ui_weak = ui.as_weak();
    ui.on_note(move |line| {
        if let Some(ui) = ui_weak.upgrade() {
            // SharedString has no push(); build the new log as a String and hand it back.
            let mut log = ui.get_log().to_string();
            log.push('\n');
            log.push_str(line.as_str());
            ui.set_log(log.into());
        }
    });
}

/// Mirrors the core's viewport into the UI properties that drive the transform.
fn push_viewport(ui: &slint::Weak<MainWindow>, viewport: &Viewport) {
    if let Some(ui) = ui.upgrade() {
        ui.set_zoom_scale(viewport.scale);
        ui.set_pan_x(viewport.x);
        ui.set_pan_y(viewport.y);
    }
}

#[cfg(target_os = "android")]
mod android {
    /// Android entry point. `android-activity` owns the exported NativeActivity glue; this
    /// is a plain Rust function, not `extern "C"`.
    #[unsafe(no_mangle)]
    fn android_main(app: slint::android::AndroidApp) {
        if let Err(error) = slint::android::init(app) {
            eprintln!("kavider: android backend init failed: {error}");
            return;
        }
        if let Err(error) = super::run() {
            eprintln!("kavider: event loop failed: {error}");
        }
    }
}
