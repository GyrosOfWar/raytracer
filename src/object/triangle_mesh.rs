use std::{error::Error, path::Path, sync::Arc};

use tracing::info;

use crate::{
    aabb::Aabb,
    material::{helpers, Material},
    range::Range,
    ray::Ray,
    texture::TextureCoordinates,
    vec3::{Point3, Vec3},
};

use super::{HitRecord, Hittable, Object};

#[derive(Debug)]
struct TriangleMeshData {
    vertices: Box<[Point3<f32>]>,
    face_indices: Box<[(u32, u32, u32)]>,
    normals: Box<[Vec3<f32>]>,
    uv: Box<[TextureCoordinates]>,
    material: Arc<Material>,
}

impl TriangleMeshData {
    pub fn new(
        vertices: Vec<Point3<f32>>,
        face_indices: Vec<(u32, u32, u32)>,
        normals: Vec<Vec3<f32>>,
        uv: Vec<TextureCoordinates>,
        material: Arc<Material>,
    ) -> Self {
        TriangleMeshData {
            vertices: vertices.into_boxed_slice(),
            face_indices: face_indices.into_boxed_slice(),
            normals: normals.into_boxed_slice(),
            uv: uv.into_boxed_slice(),
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
        uv: Vec<TextureCoordinates>,
        material: Arc<Material>,
    ) -> Self {
        let data = TriangleMeshData::new(vertices, face_indices, normals, uv, material);
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

    pub fn normals(&self) -> (Vec3<f32>, Vec3<f32>, Vec3<f32>) {
        let (v0, v1, v2) = self.mesh.face_indices[self.index as usize];
        (
            self.mesh.normals[v0 as usize],
            self.mesh.normals[v1 as usize],
            self.mesh.normals[v2 as usize],
        )
    }
}

impl Hittable for TriangleRef {
    fn hit(&self, ray: &Ray<f32>, hit_range: Range) -> Option<HitRecord> {
        let (v0, v1, v2) = self.vertices();

        // Möller-Trumbore algorithm
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

        let (n0, n1, n2) = self.normals();
        // interpolate normals based on barycentric coordinates
        let normal = n0 * (1.0 - u - v) + n1 * u + n2 * v;

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

pub fn load_from_gltf(path: impl AsRef<Path>) -> Result<Vec<Object>, Box<dyn Error>> {
    let (gltf, buffers, images) = gltf::import(path)?;
    let mut meshes = Vec::new();

    for source_mesh in gltf.meshes() {
        let mut vertices = Vec::new();
        let mut face_indices = Vec::new();
        let mut normals = Vec::new();
        let mut uv = Vec::new();

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

            if let Some(tex_coords) = reader.read_tex_coords(0) {
                let tex_coords: Vec<_> = tex_coords.into_f32().collect();
                uv.extend(
                    tex_coords
                        .into_iter()
                        .map(|uv| TextureCoordinates::from_array(uv)),
                )
            }
        }

        info!(
            "loaded mesh with {} vertices, {} faces, {} normals and {} texture coordinates",
            vertices.len(),
            face_indices.len(),
            normals.len(),
            uv.len(),
        );

        meshes.push(TriangleMesh::new(
            vertices,
            face_indices,
            normals,
            uv,
            // diffuse_light(Point3::new(1.0, 1.0, 1.0)),
            // metal(Point3::new(0.1, 0.1, 0.1), 0.02),
            helpers::lambertian(Point3::new(0.2, 0.2, 0.9)),
        ));
    }

    Ok(meshes
        .into_iter()
        .flat_map(|m| m.faces().collect::<Vec<_>>())
        .map(|f| Object::TriangleRef(f))
        .collect())
}
