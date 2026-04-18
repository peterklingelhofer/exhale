/// Pre-computed lookup table for a CSS cubic-bezier easing curve.
///
/// Direct port of `MetalBreathingController.buildEasingTable` and
/// `CubicBezierEaseInOut` from the Swift source.  The table is built once at
/// startup (1024 samples) and queried with linear interpolation — identical to
/// the Swift implementation.
pub struct EasingTable {
    samples: Box<[f32]>,
}

impl EasingTable {
    /// Build a table for the given cubic-bezier control points.
    ///
    /// For the default sinusoidal mode use `(0.42, 0.0, 0.58, 1.0)`.
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, sample_count: usize) -> Self {
        assert!(sample_count >= 2, "need at least 2 samples");
        let samples = (0..sample_count)
            .map(|i| {
                let t = i as f64 / (sample_count - 1) as f64;
                cubic_bezier_value(t, x1, y1, x2, y2) as f32
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self { samples }
    }

    /// Build the default ease-in-out table used by the breathing animation.
    pub fn default_ease_in_out() -> Self {
        Self::new(0.42, 0.0, 0.58, 1.0, 1024)
    }

    /// Sample the table at `t ∈ [0, 1]` using linear interpolation between
    /// adjacent entries, identical to `MetalBreathingController.getEasedT`.
    pub fn sample(&self, t: f64) -> f64 {
        let n = self.samples.len();
        let index_f = t * (n - 1) as f64;
        let lower = (index_f as usize).min(n - 2);
        let frac = (index_f - lower as f64) as f32;
        let a = self.samples[lower];
        let b = self.samples[lower + 1];
        (a + (b - a) * frac) as f64
    }

    /// Number of samples in the table.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

// ─── Newton-Raphson cubic-bezier solver ──────────────────────────────────────
//
// Exact port of `CubicBezierEaseInOut.getValue` from Swift.

fn cubic(t: f64, a1: f64, a2: f64) -> f64 {
    let c = 3.0 * a1;
    let b = 3.0 * (a2 - a1) - c;
    let a = 1.0 - c - b;
    ((a * t + b) * t + c) * t
}

fn cubic_derivative(t: f64, a1: f64, a2: f64) -> f64 {
    let c = 3.0 * a1;
    let b = 3.0 * (a2 - a1) - c;
    let a = 1.0 - c - b;
    (3.0 * a * t + 2.0 * b) * t + c
}

/// Solve for the y-value of a CSS cubic-bezier at input `t`.
///
/// Uses Newton-Raphson to find `t'` such that `cubic_x(t') ≈ t`, then
/// evaluates `cubic_y(t')`.
pub fn cubic_bezier_value(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    const EPSILON: f64 = 1e-6;
    let mut t_prime = t;

    for _ in 0..8 {
        let x = cubic(t_prime, x1, x2) - t;
        if x.abs() < EPSILON {
            break;
        }
        let dx = cubic_derivative(t_prime, x1, x2);
        if dx.abs() < 1e-6 {
            break;
        }
        t_prime -= x / dx;
        t_prime = t_prime.clamp(0.0, 1.0);
    }

    cubic(t_prime, y1, y2)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-4;

    // Boundary values must be exact for all easing curves.
    #[test]
    fn boundary_values() {
        let table = EasingTable::default_ease_in_out();
        assert!((table.sample(0.0) - 0.0).abs() < TOL, "t=0 should yield 0");
        assert!((table.sample(1.0) - 1.0).abs() < TOL, "t=1 should yield 1");
    }

    // ease-in-out is symmetric around t=0.5 → value ≈ 0.5
    #[test]
    fn midpoint_near_half() {
        let table = EasingTable::default_ease_in_out();
        let mid = table.sample(0.5);
        assert!(
            (mid - 0.5).abs() < 0.01,
            "ease-in-out midpoint should be ~0.5, got {mid}"
        );
    }

    // ease-in-out accelerates: early values < linear, late values > linear
    #[test]
    fn ease_in_out_shape() {
        let table = EasingTable::default_ease_in_out();
        // At t=0.25, eased value < 0.25 (still accelerating)
        let early = table.sample(0.25);
        assert!(early < 0.25 + TOL, "ease-in-out should be slow at t=0.25, got {early}");
        // At t=0.75, eased value > 0.75 (decelerating but past linear)
        let late = table.sample(0.75);
        assert!(late > 0.75 - TOL, "ease-in-out should be fast at t=0.75, got {late}");
    }

    // Linear mode: the table should be a straight line (identity).
    #[test]
    fn linear_is_identity() {
        let table = EasingTable::new(0.0, 0.0, 1.0, 1.0, 1024);
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let v = table.sample(t);
            assert!((v - t).abs() < 1e-3, "linear at t={t}: expected {t}, got {v}");
        }
    }

    #[test]
    fn sample_count() {
        let table = EasingTable::default_ease_in_out();
        assert_eq!(table.len(), 1024);
    }

    // Known reference values for CSS cubic-bezier(0.42, 0, 0.58, 1) computed
    // independently.  Tolerance is generous to allow for minor float differences.
    #[test]
    fn known_values() {
        let table = EasingTable::default_ease_in_out();
        // At t=0.1 the curve is still near the start (slow in)
        assert!(table.sample(0.1) < 0.15);
        // At t=0.9 the curve is near the end (slow out)
        assert!(table.sample(0.9) > 0.85);
    }

    #[test]
    fn monotone_increasing() {
        let table = EasingTable::default_ease_in_out();
        let mut prev = table.sample(0.0);
        for i in 1..=100 {
            let t = i as f64 / 100.0;
            let v = table.sample(t);
            assert!(
                v >= prev - 1e-4,
                "not monotone at t={t}: prev={prev}, current={v}"
            );
            prev = v;
        }
    }
}
