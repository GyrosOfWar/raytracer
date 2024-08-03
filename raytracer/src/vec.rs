use glam::Mat3;

pub type Point3 = glam::Vec3;
pub type Vec3 = glam::Vec3;

#[derive(Debug, Copy, Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - n * Vec3::dot(v, n) * 2.0
}

pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parallel = n * -(1.0 - r_out_perp.length_squared()).abs().sqrt();
    r_out_perp + r_out_parallel
}

pub trait Vec3Ext {
    fn near_zero(&self) -> bool;
    fn at(&self, axis: Axis) -> f32;
    fn get(&self, index: usize) -> f32;
}

impl Vec3Ext for Vec3 {
    fn near_zero(&self) -> bool {
        let s = 1e-8;
        self.x.abs() < s && self.y.abs() < s && self.z.abs() < s
    }

    fn at(&self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    fn get(&self, index: usize) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("index out of bounds"),
        }
    }
}

pub trait MatExt {
    fn get(&self, i: usize, j: usize) -> f32;
    fn set(&mut self, i: usize, j: usize, value: f32);
}

impl MatExt for Mat3 {
    fn get(&self, i: usize, j: usize) -> f32 {
        self.row(i).get(j)
    }

    fn set(&mut self, i: usize, j: usize, value: f32) {
        let vec = match i {
            0 => &mut self.x_axis,
            1 => &mut self.y_axis,
            2 => &mut self.z_axis,
            _ => panic!("index out of bounds"),
        };

        let dst = match j {
            0 => &mut vec.x,
            1 => &mut vec.y,
            2 => &mut vec.z,
            _ => panic!("index out of bounds"),
        };

        *dst = value;
    }
}

pub mod random {
    use glam::Vec2;

    use super::Vec3;
    use crate::random::{random, random_range};

    pub fn gen() -> Vec3 {
        Vec3::new(random(), random(), random())
    }

    pub fn gen_2d() -> Vec2 {
        Vec2::new(random(), random())
    }

    pub fn gen_range(min: f32, max: f32) -> Vec3 {
        Vec3::new(
            random_range(min, max),
            random_range(min, max),
            random_range(min, max),
        )
    }

    pub fn gen_unit_sphere() -> Vec3 {
        loop {
            let p = gen_range(-1.0, 1.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn gen_unit_disk() -> Vec3 {
        loop {
            let p = Vec3::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0), 0.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    pub fn gen_unit_vector() -> Vec3 {
        gen_unit_sphere().normalize()
    }

    pub fn gen_on_hemisphere(normal: Vec3) -> Vec3 {
        let on_unit_sphere = gen_unit_vector();
        if on_unit_sphere.dot(normal) > 0.0 {
            on_unit_sphere
        } else {
            -on_unit_sphere
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Mat3;

    use super::MatExt;

    #[test]
    fn mat_set_value() {
        let mut mat = Mat3::ZERO;
        mat.set(0, 0, 5.0);
        assert_eq!(mat.x_axis.x, 5.0);
        assert_eq!(mat.get(0, 0), 5.0);
    }
}
