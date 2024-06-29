use std::cell::RefCell;

use rand::{rngs::SmallRng, Rng, SeedableRng};

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_seed([1; 32]));
}

pub fn random() -> f32 {
    RNG.with_borrow_mut(|r| r.gen())
}

pub fn random_range(min: f32, max: f32) -> f32 {
    RNG.with_borrow_mut(|r| r.gen_range(min..max))
}

pub fn random_int(min: i32, max: i32) -> i32 {
    RNG.with_borrow_mut(|r| r.gen_range(min..max))
}
