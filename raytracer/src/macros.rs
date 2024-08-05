#[macro_export]
macro_rules! assert_in_range {
    ($value:expr, $min:expr, $max:expr) => {
        assert!(
            !$value.is_infinite() && !$value.is_nan(),
            "Value must not be infinite or NaN, got: {}",
            $value
        );
        assert!(
            $value >= $min && $value <= $max,
            "Value out of range: {} not in [{}, {}]",
            $value,
            $min,
            $max
        );
    };
}

#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr,$right:expr,$eps:expr) => {
        let delta = ($left - $right).abs();
        assert!(
            delta < $eps,
            "Values not approximately equal:\n    left = {}\n    right = {}\n    difference: {} > {}",
            $left,
            $right,
            delta,
            $eps
        );
    };
}
