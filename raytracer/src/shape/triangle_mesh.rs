use std::fmt;
use std::sync::Arc;

use super::{Shape, ShapeIntersection, ShapeSample, SurfaceInteraction};
use crate::bounds::Bounds3;
use crate::math::DirectionCone;
use crate::ray::RayLike;
use crate::vec::{Point2, Point3, Vec3};

struct TriangleMeshData {
    vertices: Box<[Point3]>,
    face_indices: Box<[(u32, u32, u32)]>,
    normals: Box<[Vec3]>,
    uv: Box<[Point2]>,
}

impl fmt::Debug for TriangleMeshData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TriangleMeshData")
            .field("vertices", &self.vertices.len())
            .field("face_indices", &self.face_indices.len())
            .field("normals", &self.normals.len())
            .field("uv", &self.uv.len())
            .finish()
    }
}

impl TriangleMeshData {
    pub fn new(
        vertices: Vec<Point3>,
        face_indices: Vec<(u32, u32, u32)>,
        normals: Vec<Vec3>,
        uv: Vec<Point2>,
    ) -> Self {
        TriangleMeshData {
            vertices: vertices.into_boxed_slice(),
            face_indices: face_indices.into_boxed_slice(),
            normals: normals.into_boxed_slice(),
            uv: uv.into_boxed_slice(),
        }
    }

    pub fn vertex(&self, index: u32) -> Point3 {
        self.vertices[index as usize]
    }

    pub fn normal(&self, index: u32) -> Option<&Vec3> {
        self.normals.get(index as usize)
    }

    pub fn uv(&self, index: u32) -> Point2 {
        self.uv[index as usize]
    }
}

#[derive(Clone, Debug)]
pub struct TriangleMesh {
    data: Arc<TriangleMeshData>,
}

impl TriangleMesh {
    pub fn new(
        vertices: Vec<Point3>,
        face_indices: Vec<(u32, u32, u32)>,
        normals: Vec<Vec3>,
        uv: Vec<Point2>,
    ) -> Self {
        let data = TriangleMeshData::new(vertices, face_indices, normals, uv);
        TriangleMesh {
            data: Arc::new(data),
        }
    }

    pub fn face(&self, index: u32) -> TriangleRef {
        TriangleRef {
            mesh: self.data.clone(),
            index,
        }
    }

    pub fn faces(&self) -> impl Iterator<Item = TriangleRef> + '_ {
        self.data
            .face_indices
            .iter()
            .enumerate()
            .map(|(i, _)| self.face(i as u32))
    }
}

#[derive(Debug)]
pub struct TriangleRef {
    // TODO this could eventually be replaced by an index to some global storage for vertex data
    mesh: Arc<TriangleMeshData>,
    index: u32,
}

impl TriangleRef {
    pub fn vertices(&self) -> (Point3, Point3, Point3) {
        let (v0, v1, v2) = self.mesh.face_indices[self.index as usize];
        (
            self.mesh.vertex(v0),
            self.mesh.vertex(v1),
            self.mesh.vertex(v2),
        )
    }

    pub fn normals(&self) -> Option<(Vec3, Vec3, Vec3)> {
        let (v0, v1, v2) = self.mesh.face_indices[self.index as usize];
        match (
            self.mesh.normal(v0),
            self.mesh.normal(v1),
            self.mesh.normal(v2),
        ) {
            (Some(a), Some(b), Some(c)) => Some((*a, *b, *c)),
            _ => None,
        }
    }

    pub fn uv(&self, a: f32, b: f32) -> Point2 {
        if self.mesh.uv.len() == 0 {
            Point2::default()
        } else {
            let (v0, v1, v2) = self.mesh.face_indices[self.index as usize];
            let uv0 = self.mesh.uv(v0);
            let uv1 = self.mesh.uv(v1);
            let uv2 = self.mesh.uv(v2);
            Point2::tri_lerp(uv0, uv1, uv2, a, b)
        }
    }
}

impl Shape for TriangleRef {
    fn bounds(&self) -> Bounds3 {
        todo!()
    }

    fn normal_bounds(&self) -> DirectionCone {
        todo!()
    }

    fn intersect(&self, ray: impl RayLike, t_max: f32) -> Option<ShapeIntersection> {
        let (v0, v1, v2) = self.vertices();

        // Möller-Trumbore algorithm
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let p = ray.direction().cross(&e2);
        let det = e1.dot(p);

        if det.abs() < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        let t = ray.origin() - v0;
        let u = t.dot(p) * inv_det;

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = t.cross(&e1);
        let v = ray.direction().dot(q) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = e2.dot(q) * inv_det;

        if t >= t_max {
            return None;
        }

        let normal = if let Some((n0, n1, n2)) = self.normals() {
            // interpolate normals based on barycentric coordinates
            n0 * (1.0 - u - v) + n1 * u + n2 * v
        } else {
            todo!()
            // default_normal(v0, v1, v2)
        };
        let uv = self.uv(u, v);

        Some(ShapeIntersection {
            interaction: SurfaceInteraction {
                point: ray.evaluate(t),
                wo: -ray.direction(),
                normal,
                uv,
            },
            t_hit: t,
        })
    }

    fn intersect_p(&self, ray: impl RayLike, t_max: f32) -> bool {
        let (v0, v1, v2) = self.vertices();

        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let p = ray.direction().cross(&e2);
        let det = e1.dot(p);

        if det.abs() < f32::EPSILON {
            return false;
        }

        let inv_det = 1.0 / det;
        let t = ray.origin() - v0;
        let u = t.dot(p) * inv_det;

        if u < 0.0 || u > 1.0 {
            return false;
        }

        let q = t.cross(&e1);
        let v = ray.direction().dot(q) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return false;
        }

        let t = e2.dot(q) * inv_det;

        if t >= t_max {
            return false;
        }

        true
    }

    fn sample(&self, u: Point2) -> ShapeSample {
        todo!()
    }

    fn pdf(&self) -> f32 {
        todo!()
    }

    fn area(&self) -> f32 {
        todo!()
    }
}
