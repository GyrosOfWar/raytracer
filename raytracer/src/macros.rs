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
