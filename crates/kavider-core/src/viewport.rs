//! Pure viewport math for the M0 gesture feasibility experiment.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub scale: f32,
    pub x: f32,
    pub y: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            scale: 1.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl Viewport {
    /// Multiplies the current scale by `factor`, keeping the image point that sits under
    /// `(anchor_x, anchor_y)` (in viewport coordinates) fixed on screen.
    pub fn zoom_at(&mut self, factor: f32, anchor_x: f32, anchor_y: f32) {
        if !self.scale.is_finite() || self.scale <= 0.0 {
            // A degenerate scale would turn the ratio into NaN or ±infinity and the whole
            // view would never recover, so repair it instead of propagating the damage.
            self.scale = 1.0;
        }
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let next_scale = self.scale * factor;
        let ratio = next_scale / self.scale;
        self.x = anchor_x - (anchor_x - self.x) * ratio;
        self.y = anchor_y - (anchor_y - self.y) * ratio;
        self.scale = next_scale;
    }

    /// Sets the pan offset without touching the scale.
    pub fn pan_to(&mut self, x: f32, y: f32) {
        if x.is_finite() {
            self.x = x;
        }
        if y.is_finite() {
            self.y = y;
        }
    }

    /// Returns to the fit-to-screen state.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::Viewport;

    #[test]
    fn zoom_keeps_the_image_point_under_the_gesture_anchor() {
        let mut viewport = Viewport::default();
        viewport.zoom_at(2.0, 100.0, 80.0);
        assert_eq!(viewport.scale, 2.0);
        assert_eq!((viewport.x, viewport.y), (-100.0, -80.0));
    }

    #[test]
    fn pan_moves_the_viewport_and_leaves_the_scale_alone() {
        let mut viewport = Viewport::default();
        viewport.zoom_at(2.0, 0.0, 0.0);
        viewport.pan_to(-40.0, 25.0);
        assert_eq!((viewport.x, viewport.y), (-40.0, 25.0));
        assert_eq!(viewport.scale, 2.0);
    }

    #[test]
    fn reset_returns_to_fit() {
        let mut viewport = Viewport::default();
        viewport.zoom_at(4.0, 10.0, 10.0);
        viewport.pan_to(-300.0, -300.0);
        viewport.reset();
        assert_eq!(viewport, Viewport::default());
        assert_eq!(viewport.scale, 1.0);
        assert_eq!((viewport.x, viewport.y), (0.0, 0.0));
    }

    #[test]
    fn a_degenerate_scale_is_repaired_instead_of_producing_nan() {
        let mut viewport = Viewport { scale: 0.0, x: 5.0, y: 5.0 };
        viewport.zoom_at(2.0, 0.0, 0.0);
        assert!(viewport.scale.is_finite() && viewport.scale > 0.0);
        assert!(viewport.x.is_finite() && viewport.y.is_finite());
    }
}
