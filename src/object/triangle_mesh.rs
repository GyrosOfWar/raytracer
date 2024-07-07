use std::{error::Error, path::Path, sync::Arc};

use crate::{
    aabb::Aabb,
    material::{lambertian, metal, Material},
    range::Range,
    ray::Ray,
    vec3::{Point3, Vec3},
};

use super::{HitRecord, Hittable};

#[derive(Debug)]
struct TriangleMeshData {
    vertices: Box<[Point3<f32>]>,
    face_indices: Box<[(u32, u32, u32)]>,
    normals: Box<[Vec3<f32>]>,
    material: Arc<Material>,
}

impl TriangleMeshData {
    pub fn new(
        vertices: Vec<Point3<f32>>,
        face_indices: Vec<(u32, u32, u32)>,
        normals: Vec<Vec3<f32>>,
        material: Arc<Material>,
    ) -> Self {
        TriangleMeshData {
            vertices: vertices.into_boxed_slice(),
            face_indices: face_indices.into_boxed_slice(),
            normals: normals.into_boxed_slice(),
            material,
        }
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
        material: Arc<Material>,
    ) -> Self {
        let data = TriangleMeshData::new(vertices, face_indices, normals, material);
        TriangleMesh {
            data: Arc::new(data),
        }
    }

    pub fn vertex(&self, index: u32) -> Point3<f32> {
        self.data.vertices[index as usize]
    }

    pub fn face(&self, index: u32) -> TriangleRef {
        TriangleRef {
            mesh: self.data.clone(),
            index,
            material: self.data.material.clone(),
        }
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Point3<f32>> {
        self.data.vertices.iter()
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
            self.mesh.vertices[v0 as usize],
            self.mesh.vertices[v1 as usize],
            self.mesh.vertices[v2 as usize],
        )
    }

    // FIXME?
    pub fn normal(&self) -> Vec3<f32> {
        self.mesh.normals[self.index as usize]
    }
}

impl Hittable for TriangleRef {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        let (v0, v1, v2) = self.vertices();
        // Moller-Trumbore algorithm
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let p = ray.direction.cross(e2);
        let det = e1.dot(p);

        if det.abs() < 1e-6 {
            return None;
        }

        let inv_det = 1.0 / det;
        let t = ray.origin - v0;
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

        // TODO interpolate normals
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
        // TODO cache this?
        let (v0, v1, v2) = self.vertices();
        Aabb::from_boxes(Aabb::from_points(v0, v1), Aabb::from_points(v2, v2))
    }

    fn id(&self) -> u32 {
        self.index
    }
}

pub fn load_from_gltf(path: impl AsRef<Path>) -> Result<Vec<TriangleMesh>, Box<dyn Error>> {
    let (gltf, buffers, images) = gltf::import(path)?;
    let mut meshes = Vec::new();

    for source_mesh in gltf.meshes() {
        let mut vertices = Vec::new();
        let mut face_indices = Vec::new();
        let mut normals = Vec::new();

        for primitive in source_mesh.primitives() {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));

            if let Some(positions) = reader.read_positions() {
                vertices.extend(positions.map(|p| Point3::from_array(p)));
            }

            if let Some(indices) = reader.read_indices() {
                let indices: Vec<_> = indices.into_u32().collect();
                for chunk in indices.chunks(3) {
                    face_indices.push((chunk[0], chunk[1], chunk[2]));
                }
            }

            if let Some(normals_iter) = reader.read_normals() {
                normals.extend(normals_iter.map(|n| Vec3::from_array(n)));
            }
        }

        meshes.push(TriangleMesh::new(
            vertices,
            face_indices,
            normals,
            metal(Point3::new(0.7, 0.1, 0.1), 0.02),
        ));
    }

    Ok(meshes)
}
