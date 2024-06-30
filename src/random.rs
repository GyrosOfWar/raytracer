mod rand {
    use rand::{rngs::SmallRng, Rng, SeedableRng};
    use std::cell::RefCell;

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
}

mod libc {
    use libc::{rand, RAND_MAX};

    pub fn random() -> f32 {
        let r = unsafe { rand() } as f64;

        (r / (RAND_MAX as f64 + 1.0)) as f32
    }

    pub fn random_range(min: f32, max: f32) -> f32 {
        min + (max - min) * random()
    }

    pub fn random_int(min: i32, max: i32) -> i32 {
        random_range(min as f32, (max + 1) as f32) as i32
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
