use num_traits::clamp;
use ordered_float::OrderedFloat;

use crate::math;

pub fn find_interval<P>(sz: usize, pred: P) -> usize
where
    P: Fn(usize) -> bool,
{
    let mut size: isize = sz as isize - 2;
    let mut first: isize = 1;

    while size > 0 {
        let half = (size as usize) >> 1;
        let middle = first + half as isize;
        let pred_result = pred(middle as usize);
        if pred_result {
            first = middle + 1;
            size -= half as isize + 1;
        } else {
            size = half as isize;
        }
    }

    clamp(first - 1 as isize, 0, size - 1) as usize
}

pub fn max_value(values: &[f32]) -> f32 {
    values
        .iter()
        .copied()
        .max_by_key(|v| OrderedFloat(*v))
        .unwrap_or(0.0)
}
