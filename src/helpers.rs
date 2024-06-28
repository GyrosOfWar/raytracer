use rand::Rng;

pub fn random() -> f32 {
    rand::thread_rng().gen()
}

pub fn random_range(min: f32, max: f32) -> f32 {
    rand::thread_rng().gen_range(min..max)
}

pub fn random_int(min: i32, max: i32) -> i32 {
    rand::thread_rng().gen_range(min..max)
}
