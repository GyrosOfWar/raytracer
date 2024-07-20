use color_eyre::Result;
use ordered_float::OrderedFloat;
use tracing::info;

pub fn find_interval<T: PartialOrd>(slice: &[T], item: T) -> usize {
    slice
        .binary_search_by(|probe| probe.partial_cmp(&item).expect("no NaNs allowed"))
        .unwrap_or_else(|e| e)
        .min(slice.len() - 2)
}

pub fn max_value(values: &[f32]) -> f32 {
    values
        .iter()
        .copied()
        .max_by_key(|v| OrderedFloat(*v))
        .unwrap_or(0.0)
}

pub fn is_sorted<T: PartialOrd>(slice: &[T]) -> bool {
    is_sorted_by(slice, |a, b| a < b)
}

pub fn is_sorted_by<T: PartialOrd, F>(slice: &[T], mut compare: F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    slice.windows(2).all(|w| compare(&w[0], &w[1]))
}

pub fn measure<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let start = std::time::Instant::now();
    let value = f();
    let elapsed = start.elapsed();
    info!("{} took {:?}", name, elapsed);
    value
}

pub fn try_measure<T>(name: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let start = std::time::Instant::now();
    let result = f()?;
    let elapsed = start.elapsed();
    info!("{} took {:?}", name, elapsed);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::find_interval;
    use crate::util::{is_sorted, is_sorted_by};

    #[test]
    fn test_is_sorted() {
        let numbers = vec![0.0, 1.0, 2.0, 3.0];
        assert!(is_sorted(&numbers));

        let numbers = vec![2.0, 1.0, 3.0, 4.0];
        assert!(!is_sorted(&numbers));
    }

    #[test]
    fn test_is_sorted_by() {
        let numbers = vec![3.0, 2.0, 1.0, 0.0, -1.0];
        assert!(is_sorted_by(&numbers, |a, b| a > b));
    }

    #[test]
    fn test_find_interval() {
        let values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let idx = find_interval(&values, 4.0);
        assert_eq!(idx, 4);

        let idx = find_interval(&values, 7.0);
        assert_eq!(idx, 4);

        let idx = find_interval(&values, -1.0);
        assert_eq!(idx, 0);
    }
}
