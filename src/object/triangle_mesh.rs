use crate::{
    aabb::Aabb,
    range::Range,
    ray::Ray,
    vec3::{Point3, Vec3},
};

use super::{HitRecord, Hittable};

pub struct TriangleMesh {
    vertices: Box<[Point3<f32>]>,
    face_indices: Box<[(u32, u32, u32)]>,
    normals: Box<[Vec3<f32>]>,
}

impl TriangleMesh {
    pub fn new(
        vertices: Vec<Point3<f32>>,
        face_indices: Vec<(u32, u32, u32)>,
        normals: Vec<Vec3<f32>>,
    ) -> Self {
        TriangleMesh {
            vertices: vertices.into_boxed_slice(),
            face_indices: face_indices.into_boxed_slice(),
            normals: normals.into_boxed_slice(),
        }
    }

    pub fn vertex(&self, index: u32) -> &Point3<f32> {
        &self.vertices[index as usize]
    }

    pub fn face(&self, index: u32) -> TriangeRef<'_> {
        TriangeRef { mesh: self, index }
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
}

impl<'a> TriangeRef<'a> {
    pub fn geometry(&self) -> TriangleGeometry<'a> {
        let (a, b, c) = self.mesh.face_indices[self.index as usize];

        TriangleGeometry {
            a: self.mesh.vertex(a),
            b: self.mesh.vertex(b),
            c: self.mesh.vertex(c),
        }
    }
}

#[derive(Debug)]
pub struct TriangleGeometry<'a> {
    pub a: &'a Point3<f32>,
    pub b: &'a Point3<f32>,
    pub c: &'a Point3<f32>,
}

impl<'a> Hittable for TriangleGeometry<'a> {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        todo!()
    }

    fn bounding_box(&self) -> Aabb {
        todo!()
    }

    fn id(&self) -> u32 {
        todo!()
    }
}
