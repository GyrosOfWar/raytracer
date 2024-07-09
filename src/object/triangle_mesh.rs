use std::sync::Arc;

use builders::default_normal;
use nalgebra::Projective3;

use super::{HitRecord, Hittable};
use crate::aabb::Aabb;
use crate::material::helpers::{lambertian, lambertian_texture};
use crate::material::Material;
use crate::range::Range;
use crate::ray::Ray;
use crate::texture::TextureCoordinates;
use crate::vec3::{Point3, Vec3};

#[derive(Debug)]
struct TriangleMeshData {
    vertices: Box<[Point3<f32>]>,
    face_indices: Box<[(u32, u32, u32)]>,
    normals: Box<[Vec3<f32>]>,
    uv: Box<[TextureCoordinates]>,
    material: Arc<Material>,
    transform: Projective3<f32>,
}

impl TriangleMeshData {
    pub fn new(
        vertices: Vec<Point3<f32>>,
        face_indices: Vec<(u32, u32, u32)>,
        normals: Vec<Vec3<f32>>,
        uv: Vec<TextureCoordinates>,
        material: Arc<Material>,
        transform: Projective3<f32>,
    ) -> Self {
        TriangleMeshData {
            vertices: vertices.into_boxed_slice(),
            face_indices: face_indices.into_boxed_slice(),
            normals: normals.into_boxed_slice(),
            uv: uv.into_boxed_slice(),
            material,
            transform,
        }
    }

    pub fn vertex(&self, index: u32) -> Point3<f32> {
        self.transform * self.vertices[index as usize]
    }
}

#[derive(Clone, Debug)]
pub struct TriangleMesh {
    data: Arc<TriangleMeshData>,
}

impl TriangleMesh {
    pub fn new(
        vertices: Vec<Point3<f32>>,
        face_indices: Vec<(u32, u32, u32)>,
        normals: Vec<Vec3<f32>>,
        uv: Vec<TextureCoordinates>,
        material: Arc<Material>,
        transform: Projective3<f32>,
    ) -> Self {
        let data = TriangleMeshData::new(vertices, face_indices, normals, uv, material, transform);
        TriangleMesh {
            data: Arc::new(data),
        }
    }

    pub fn face(&self, index: u32) -> TriangleRef {
        TriangleRef {
            mesh: self.data.clone(),
            index,
            material: self.data.material.clone(),
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
    mesh: Arc<TriangleMeshData>,
    index: u32,
    material: Arc<Material>,
}

impl TriangleRef {
    pub fn vertices(&self) -> (Point3<f32>, Point3<f32>, Point3<f32>) {
        let (v0, v1, v2) = self.mesh.face_indices[self.index as usize];
        (
            self.mesh.vertex(v0),
            self.mesh.vertex(v1),
            self.mesh.vertex(v2),
        )
    }

    pub fn normals(&self) -> Option<(Vec3<f32>, Vec3<f32>, Vec3<f32>)> {
        let (v0, v1, v2) = self.mesh.face_indices[self.index as usize];
        match (
            self.mesh.normals.get(v0 as usize),
            self.mesh.normals.get(v1 as usize),
            self.mesh.normals.get(v2 as usize),
        ) {
            (Some(a), Some(b), Some(c)) => Some((*a, *b, *c)),
            _ => None,
        }
    }

    pub fn uv(&self, a: f32, b: f32) -> TextureCoordinates {
        if self.mesh.uv.len() == 0 {
            TextureCoordinates::default()
        } else {
            let (v0, v1, v2) = self.mesh.face_indices[self.index as usize];
            let uv0 = self.mesh.uv[v0 as usize];
            let uv1 = self.mesh.uv[v1 as usize];
            let uv2 = self.mesh.uv[v2 as usize];
            TextureCoordinates::tri_lerp(uv0, uv1, uv2, a, b)
        }
    }
}

impl Hittable for TriangleRef {
    fn hit(&self, ray: &Ray, hit_range: Range) -> Option<HitRecord> {
        let (v0, v1, v2) = self.vertices();

        // Möller-Trumbore algorithm
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let p = ray.direction.cross(&e2);
        let det = e1.dot(&p);

        if det.abs() < 1e-6 {
            return None;
        }

        let inv_det = 1.0 / det;
        let t = ray.origin - v0;
        let u = t.dot(&p) * inv_det;

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = t.cross(&e1);
        let v = ray.direction.dot(&q) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = e2.dot(&q) * inv_det;
        if !hit_range.contains(t) {
            return None;
        }

        let normal = if let Some((n0, n1, n2)) = self.normals() {
            // interpolate normals based on barycentric coordinates
            n0 * (1.0 - u - v) + n1 * u + n2 * v
        } else {
            default_normal(v0, v1, v2)
        };
        let uv = self.uv(u, v);

        Some(HitRecord::new(
            ray,
            normal,
            ray.evaluate(t),
            t,
            self.material.clone(),
            uv,
        ))
    }

    fn bounding_box(&self) -> Aabb {
        // TODO cache this?
        let (v0, v1, v2) = self.vertices();
        Aabb::from_boxes(Aabb::from_points(v0, v1), Aabb::from_points(v2, v2))
    }

    fn id(&self) -> u32 {
        self.index
    }
}

pub mod builders {
    use std::sync::Arc;

    use nalgebra::Projective3;

    use super::TriangleMesh;
    use crate::material::Material;
    use crate::object::Object;
    use crate::vec3::{Point3, Vec3};

    pub fn default_normal(v0: Point3<f32>, v1: Point3<f32>, v2: Point3<f32>) -> Vec3<f32> {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        e1.cross(&e2).normalize()
    }

    pub fn quad(
        p1: Point3<f32>,
        p2: Point3<f32>,
        p3: Point3<f32>,
        p4: Point3<f32>,
        material: Arc<Material>,
    ) -> Vec<Object> {
        let mesh = TriangleMesh::new(
            vec![p1, p2, p3, p4],
            vec![(0, 1, 2), (1, 2, 3)],
            vec![],
            vec![],
            material,
            Projective3::identity(),
        );

        mesh.faces().map(Object::TriangleRef).collect()
    }
}
