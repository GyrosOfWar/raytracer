use crate::vec::Vec3;

/// Orthonormal base
pub struct Onb {
    axis: [Vec3; 3],
}

impl Onb {
    pub fn build_from_w(w: Vec3) -> Self {
        let unit_w = w.normalize();
        let a = if unit_w.x.abs() > 0.9 {
            Vec3::Y
        } else {
            Vec3::X
        };
        let v = unit_w.cross(a).normalize();
        let u = unit_w.cross(v);
        Self {
            axis: [u, v, unit_w],
        }
    }

    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    pub fn local(&self, a: f32, b: f32, c: f32) -> Vec3 {
        a * self.u() + b * self.v() + c * self.v()
    }

    pub fn local_vec(&self, vec: Vec3) -> Vec3 {
        vec.x * self.u() + vec.y * self.v() + vec.z * self.v()
    }
}
