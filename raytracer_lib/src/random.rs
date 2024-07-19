#[allow(unused)]
mod rand {
    use std::cell::RefCell;

    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
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

    pub fn choose<T>(a: T, b: T, factor: f32) -> T {
        let n = random();
        if n < factor {
            a
        } else {
            b
        }
    }
}

pub fn random() -> f32 {
    rand::random()
}

pub fn random_range(min: f32, max: f32) -> f32 {
    rand::random_range(min, max)
}

pub fn random_int(min: i32, max: i32) -> i32 {
    rand::random_int(min, max)
}

pub fn choose<T>(a: T, b: T, factor: f32) -> T {
    rand::choose(a, b, factor)
}
