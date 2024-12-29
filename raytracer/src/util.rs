use ordered_float::OrderedFloat;

use crate::math;

pub fn find_interval<F>(sz: usize, pred: F) -> usize
where
    F: Fn(usize) -> bool,
{
    let mut size = sz as isize - 2;
    let mut first = 1isize;

    while size > 0 {
        // Evaluate predicate at midpoint and update `first` and `size`
        let half = size as usize >> 1;
        let middle = first + half as isize;
        let pred_result = pred(middle as usize);
        first = if pred_result { middle + 1 } else { first };
        size = if pred_result {
            size - (half as isize + 1)
        } else {
            half as isize
        };
    }

    math::clamp(first - 1, 0, sz as isize - 2) as usize
}

pub fn max_value(values: &[f32]) -> f32 {
    values
        .iter()
        .copied()
        .max_by_key(|v| OrderedFloat(*v))
        .unwrap_or(0.0)
}

pub fn min_value(values: &[f32]) -> f32 {
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

#[cfg(test)]
mod tests {
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
}
