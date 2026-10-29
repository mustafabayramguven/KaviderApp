//! `kavider-core` — domain models, repositories, session and settings, image
//! pipeline, progress tracking and viewport maths.
//!
//! No UI dependencies: this crate builds and tests on the desktop, and every
//! boundary it touches (server, storage, clock) is a trait so tests can pass fakes.

pub mod viewport;
