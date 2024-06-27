pub fn random() -> f32 {
    let rand_int = unsafe { libc::rand() };
    let f = rand_int as f64 / (libc::RAND_MAX as f64 + 1.0);
    f as f32
}

pub fn random_range(min: f32, max: f32) -> f32 {
    min + (max - min) * random()
}
