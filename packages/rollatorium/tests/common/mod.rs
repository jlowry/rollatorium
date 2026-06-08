use rollatorium::roll;

/// Helper function to get total from roll
pub fn r(expr: &str) -> f64 {
    roll(&expr).unwrap().total
}
