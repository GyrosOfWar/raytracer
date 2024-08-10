use std::ops::Mul;

use crate::ray::Ray;
use crate::vec::{vec3, Mat4, Point3, Vec3};

#[derive(Debug, Clone)]
pub struct Transform {
    matrix: Mat4,
    inverse: Mat4,
}

impl Transform {
    pub fn new(matrix: Mat4, inverse: Mat4) -> Self {
        Transform { inverse, matrix }
    }

    pub fn identity() -> Self {
        Transform::new(Mat4::IDENTITY, Mat4::IDENTITY)
    }

    pub fn from_matrix(matrix: Mat4) -> Self {
        Transform::new(matrix, matrix.inverse())
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        Transform::new(
            Mat4::from_translation(vec3(x, y, z)),
            Mat4::from_translation(vec3(-x, -y, -z)),
        )
    }

    pub fn rotate_x(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_x(angle), Mat4::from_rotation_x(-angle))
    }

    pub fn rotate_y(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_y(angle), Mat4::from_rotation_y(-angle))
    }

    pub fn rotate_z(angle: f32) -> Self {
        Transform::new(Mat4::from_rotation_z(angle), Mat4::from_rotation_z(-angle))
    }

    fn rotate(sin_theta: f32, cos_theta: f32, axis: Vec3) -> Transform {
        let a = axis.normalized();
        let mut m = Mat4::IDENTITY;

        // Compute rotation of first basis vector
        m.set(0, 0, a.x * a.x + (1.0 - a.x * a.x) * cos_theta);
        m.set(0, 1, a.x * a.y * (1.0 - cos_theta) - a.z * sin_theta);
        m.set(0, 2, a.x * a.z * (1.0 - cos_theta) + a.y * sin_theta);
        m.set(0, 3, 0.0);

        // Compute rotations of second and third basis vectors
        m.set(1, 0, a.x * a.y * (1.0 - cos_theta) + a.z * sin_theta);
        m.set(1, 1, a.y * a.y + (1.0 - a.y * a.y) * cos_theta);
        m.set(1, 2, a.y * a.z * (1.0 - cos_theta) - a.x * sin_theta);
        m.set(1, 3, 0.0);

        m.set(2, 0, a.x * a.z * (1.0 - cos_theta) - a.y * sin_theta);
        m.set(2, 1, a.y * a.z * (1.0 - cos_theta) + a.x * sin_theta);
        m.set(2, 2, a.z * a.z + (1.0 - a.z * a.z) * cos_theta);
        m.set(2, 3, 0.0);

        // return Transform(m, Transpose(m));
        Transform::new(m, m.transpose())
    }

    pub fn rotate_on_axis(angle: f32, axis: Vec3) -> Self {
        todo!()
    }

    pub fn rotate_from_to(from: Vec3, to: Vec3) -> Self {
        todo!()
        // let axis = from.cross(&to).normalized();
        // let angle = from.dot(to).acos();
        // Transform::new(Mat4::from_rotation(angle, axis), Mat4::from_rotation(-angle, axis))
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Transform::new(
            Mat4::from_scale(vec3(x, y, z)),
            Mat4::from_scale(vec3(1.0 / x, 1.0 / y, 1.0 / z)),
        )
    }

    pub fn uniform_scale(s: f32) -> Self {
        Transform::scale(s, s, s)
    }

    pub fn look_at(pos: Point3, look: Point3, up: Vec3) -> Self {
        let mut world_from_camera = Mat4::IDENTITY;
        world_from_camera.set(0, 3, pos.x);
        world_from_camera.set(1, 3, pos.y);
        world_from_camera.set(2, 3, pos.z);
        world_from_camera.set(3, 3, 1.0);

        let dir = (look - pos).normalized();
        let right = up.normalized().cross(&dir).normalized();
        let new_up = dir.cross(&right).normalized();
        world_from_camera.set(0, 0, right.x);
        world_from_camera.set(1, 0, right.y);
        world_from_camera.set(2, 0, right.z);
        world_from_camera.set(3, 0, 0.);
        world_from_camera.set(0, 1, new_up.x);
        world_from_camera.set(1, 1, new_up.y);
        world_from_camera.set(2, 1, new_up.z);
        world_from_camera.set(3, 1, 0.);
        world_from_camera.set(0, 2, dir.x);
        world_from_camera.set(1, 2, dir.y);
        world_from_camera.set(2, 2, dir.z);
        world_from_camera.set(3, 2, 0.);

        let camera_from_world = world_from_camera.inverse();

        Transform::new(camera_from_world, world_from_camera)
    }

    pub fn perspective(fov: f32, n: f32, f: f32) -> Self {
        let perspective = Mat4::from_rows(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, f / (f - n), -f * n / (f - n)],
            [0.0, 0.0, 1.0, 0.0],
        );
        let inv_tan_arg = 1.0 / f32::tan(fov.to_radians() / 2.0);

        Transform::scale(inv_tan_arg, inv_tan_arg, 1.0) * Transform::from_matrix(perspective)
    }

    pub fn transform_point(&self, p: Point3) -> Point3 {
        let m = self.matrix;
        let xp = m.get(0, 0) * p.x + m.get(0, 1) * p.y + m.get(0, 2) * p.z + m.get(0, 3);
        let yp = m.get(1, 0) * p.x + m.get(1, 1) * p.y + m.get(1, 2) * p.z + m.get(1, 3);
        let zp = m.get(2, 0) * p.x + m.get(2, 1) * p.y + m.get(2, 2) * p.z + m.get(2, 3);
        let wp = m.get(3, 0) * p.x + m.get(3, 1) * p.y + m.get(3, 2) * p.z + m.get(3, 3);
        if wp == 1.0 {
            Point3::new(xp, yp, zp)
        } else {
            Point3::new(xp, yp, zp) / wp
        }
    }

    pub fn transform_point_inverse(&self, p: Point3) -> Point3 {
        let m = self.inverse;
        let xp = m.get(0, 0) * p.x + m.get(0, 1) * p.y + m.get(0, 2) * p.z + m.get(0, 3);
        let yp = m.get(1, 0) * p.x + m.get(1, 1) * p.y + m.get(1, 2) * p.z + m.get(1, 3);
        let zp = m.get(2, 0) * p.x + m.get(2, 1) * p.y + m.get(2, 2) * p.z + m.get(2, 3);
        let wp = m.get(3, 0) * p.x + m.get(3, 1) * p.y + m.get(3, 2) * p.z + m.get(3, 3);
        if wp == 1.0 {
            Point3::new(xp, yp, zp)
        } else {
            Point3::new(xp, yp, zp) / wp
        }
    }

    pub fn transform_vector(&self, v: Vec3) -> Vec3 {
        let m = self.matrix;
        Vec3::new(
            m.get(0, 0) * v.x + m.get(0, 1) * v.y + m.get(0, 2) * v.z,
            m.get(1, 0) * v.x + m.get(1, 1) * v.y + m.get(1, 2) * v.z,
            m.get(2, 0) * v.x + m.get(2, 1) * v.y + m.get(2, 2) * v.z,
        )
    }

    pub fn transform_normal(&self, v: Vec3) -> Vec3 {
        todo!()
    }

    pub fn transform_ray(&self, ray: Ray) -> Ray {
        // TODO check against the pbrt source
        Ray::new(
            self.transform_point(ray.origin),
            self.transform_vector(ray.direction),
        )
    }

    pub fn inverse(&self) -> Transform {
        Self {
            matrix: self.inverse,
            inverse: self.matrix,
        }
    }
}

impl Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform {
            matrix: self.matrix * rhs.matrix,
            inverse: rhs.inverse * self.inverse,
        }
    }
}

#[cfg(test)]
mod tests {

    use rand::{thread_rng, Rng};

    use crate::random::random;
    use crate::sample::sample_uniform_sphere;
    use crate::transform::Transform;
    use crate::vec::{Point2, Vec3};

    fn random_transform() -> Transform {
        let mut rng = thread_rng();
        let mut t = Transform::identity();
        let r = || -10.0 + 20.0 * random();
        for _ in 0..10 {
            match rng.gen_range(0..3) {
                0 => t = t * Transform::scale(r().abs(), r().abs(), r().abs()),
                1 => t = t * Transform::translate(r(), r(), r()),
                2 => {
                    let angle = r() * 20.;
                    // let axis = sample_uniform_sphere(Point2::new(rng.uniform(), rng.uniform()));
                    let axis = sample_uniform_sphere(Point2::new(rng.gen(), rng.gen()));
                    t = t * Transform::rotate_on_axis(angle, axis);
                }
                _ => unreachable!(),
            }
        }
        t
    }

    #[test]
    fn rotate_from_to_simple() {
        // Same directions...
        let from = Vec3::new(0., 0., 1.);
        let to = Vec3::new(0., 0., 1.);
        let r = Transform::rotate_from_to(from, to);
        let to_new = r.transform_vector(from);
        assert_eq!(to, to_new);

        let from = Vec3::new(0., 0., 1.);
        let to = Vec3::new(1., 0., 0.);
        let r = Transform::rotate_from_to(from, to);
        let to_new = r.transform_vector(from);
        assert_eq!(to, to_new);

        let from = Vec3::new(0., 0., 1.);
        let to = Vec3::new(0., 1., 0.);
        let r = Transform::rotate_from_to(from, to);
        let to_new = r.transform_vector(from);
        assert_eq!(to, to_new);
    }

    #[test]
    fn rotate_from_to_randoms() {
        for _ in 0..100 {
            let from = sample_uniform_sphere(Point2::new(random(), random()));
            let to = sample_uniform_sphere(Point2::new(random(), random()));

            let r = Transform::rotate_from_to(from, to);
            let to_new = r.transform_vector(from);
            assert!((to_new.length() - 1.).abs() < 1e-3);
            assert!(to.dot(to_new) > 0.999);
        }
    }
}
