use glam::Vec3A;

pub fn safe_sqrt(u: f32) -> f32 {
    u.max(0.0).sqrt()
}

pub fn abs_cos_theta(v: Vec3A) -> f32 {
    v.z.abs()
}

pub fn lerp(x: f32, a: f32, b: f32) -> f32 {
    (1.0 - x) * a + x * b
}
