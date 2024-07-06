use std::sync::Arc;

use crate::{
    aabb::Aabb,
    material::{self, Material},
    range::Range,
    ray::Ray,
    vec3::{Point3, Vec3},
};

use super::{HitRecord, Hittable};

pub struct TriangleMesh {
    vertices: Box<[Point3<f32>]>,
    face_indices: Box<[(u32, u32, u32)]>,
    normals: Box<[Vec3<f32>]>,
    material: Arc<Material>,
}

impl TriangleMesh {
    pub fn new(
        vertices: Vec<Point3<f32>>,
        face_indices: Vec<(u32, u32, u32)>,
        normals: Vec<Vec3<f32>>,
        material: Arc<Material>,
    ) -> Self {
        TriangleMesh {
            vertices: vertices.into_boxed_slice(),
            face_indices: face_indices.into_boxed_slice(),
            normals: normals.into_boxed_slice(),
            material,
        }
    }

    pub fn vertex(&self, index: u32) -> &Point3<f32> {
        &self.vertices[index as usize]
    }

    pub fn face(&self, index: u32) -> TriangeRef<'_> {
        TriangeRef {
            mesh: self,
            index,
            material: self.material.clone(),
        }
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Point3<f32>> {
        self.vertices.iter()
    }

    pub fn faces(&self) -> impl Iterator<Item = TriangeRef<'_>> {
        self.face_indices
            .iter()
            .enumerate()
            .map(|(i, _)| self.face(i as u32))
    }
}

pub struct TriangeRef<'a> {
    mesh: &'a TriangleMesh,
    index: u32,
    material: Arc<Material>,
}

impl<'a> TriangeRef<'a> {
    pub fn geometry(&self) -> TriangleGeometry<'a> {
        let (a, b, c) = self.mesh.face_indices[self.index as usize];

        TriangleGeometry::new(
            self.index,
            self.mesh.vertex(a),
            self.mesh.vertex(b),
            self.mesh.vertex(c),
            self.material.clone(),
        )
    }
}

#[derive(Debug)]
pub struct TriangleGeometry<'a> {
    v0: &'a Point3<f32>,
    v1: &'a Point3<f32>,
    v2: &'a Point3<f32>,
    material: Arc<Material>,
    bbox: Aabb,
    id: u32,
}

impl<'a> TriangleGeometry<'a> {
    pub fn new(
        id: u32,
        v0: &'a Point3<f32>,
        v1: &'a Point3<f32>,
        v2: &'a Point3<f32>,
        material: Arc<Material>,
    ) -> Self {
        let bbox = Aabb::from_boxes(Aabb::from_points(*v0, *v1), Aabb::from_points(*v2, *v2));

        TriangleGeometry {
            v0,
            v1,
            v2,
            material,
            bbox,
            id,
        }
    }
}

impl<'a> Hittable for TriangleGeometry<'a> {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        // Moller-Trumbore algorithm
        let e1 = *self.v1 - *self.v0;
        let e2 = *self.v2 - *self.v0;
        let p = ray.direction.cross(e2);
        let det = e1.dot(p);

        if det.abs() < 1e-6 {
            return None;
        }

        let inv_det = 1.0 / det;
        let t = ray.origin - *self.v0;
        let u = t.dot(p) * inv_det;

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = t.cross(e1);
        let v = ray.direction.dot(q) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = e2.dot(q) * inv_det;
        if !hit_range.contains(t) {
            return None;
        }

        let normal = e1.cross(e2).unit();

        Some(HitRecord::new(
            ray,
            normal,
            ray.evaluate(t),
            t,
            self.material.clone(),
            Default::default(),
        ))
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn id(&self) -> u32 {
        self.id
    }
}
