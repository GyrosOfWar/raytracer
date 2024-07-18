use ordered_float::OrderedFloat;

use crate::math;

pub fn find_interval<P>(sz: usize, pred: P) -> usize
where
    P: Fn(usize) -> bool,
{
    let mut size: isize = sz as isize - 2;
    let mut first: isize = 1;

    while size > 0 {
        let half = size as usize >> 1;
        let middle = first as usize + half;
        let pred_result = pred(middle);
        if pred_result {
            first = (middle + 1) as isize;
            size = size - (half + 1) as isize;
        } else {
            size = half as isize;
        }
    }

    math::clamp(size as isize - 1, 0, sz as isize - 2) as usize
}

pub fn max_value(values: &[f32]) -> f32 {
    values
        .iter()
        .copied()
        .max_by_key(|v| OrderedFloat(*v))
        .unwrap_or(0.0)
}

pub fn is_sorted<T: PartialOrd>(slice: &[T]) -> bool {
    slice.windows(2).all(|w| w[0] <= w[1])
}

pub fn is_sorted_by<T: PartialOrd, F>(slice: &[T], mut compare: F) -> bool
where
    F: FnMut(&T, &T) -> bool,
{
    slice.windows(2).all(|w| compare(&w[0], &w[1]))
}

#[cfg(test)]
mod tests {
    use super::find_interval;
    use crate::v2::util::{is_sorted, is_sorted_by};

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
        let target = 4.0;
        let idx = find_interval(values.len(), |idx| values[idx] <= target);
        assert_eq!(idx, 4);
    }
}
