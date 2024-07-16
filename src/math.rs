use glam::Vec3A;

pub fn safe_sqrt(u: f32) -> f32 {
    u.max(0.0).sqrt()
}

pub fn abs_cos_theta(v: Vec3A) -> f32 {
    v.z.abs()
}
