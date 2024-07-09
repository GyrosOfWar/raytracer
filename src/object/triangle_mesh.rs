use std::{error::Error, path::Path, sync::Arc};

use builders::default_normal;
use gltf::mesh::Mode;
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, RgbImage, Rgba, RgbaImage};
use tracing::info;

use crate::{
    aabb::Aabb,
    material::{
        helpers::{lambertian, lambertian_texture},
        Material,
    },
    range::Range,
    ray::Ray,
    texture::{Image, Texture, TextureCoordinates},
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

fn load_image(image: gltf::image::Data, name: &str) -> Result<DynamicImage, Box<dyn Error>> {
    use gltf::image::Format;

    let image = match image.format {
        Format::R8 => DynamicImage::from(
            ImageBuffer::<Luma<u8>, _>::from_raw(image.width, image.height, image.pixels)
                .expect("failed to construct image"),
        ),
        Format::R8G8 => DynamicImage::from(
            ImageBuffer::<LumaA<u8>, _>::from_raw(image.width, image.height, image.pixels)
                .expect("failed to construct image"),
        ),
        Format::R8G8B8 => DynamicImage::from(
            ImageBuffer::<Rgb<u8>, _>::from_raw(image.width, image.height, image.pixels)
                .expect("failed to construct image"),
        ),
        Format::R8G8B8A8 => DynamicImage::from(
            ImageBuffer::<Rgba<u8>, _>::from_raw(image.width, image.height, image.pixels)
                .expect("failed to construct image"),
        ),
        _ => panic!(
            "unsupported image format {:?} for image {}",
            image.format, name
        ),
    };

    Ok(image)
}

pub fn load_from_gltf(path: impl AsRef<Path>) -> Result<Vec<Object>, Box<dyn Error>> {
    let (gltf, buffers, mut images) = gltf::import(path)?;
    let mut meshes = Vec::new();

    for source_mesh in gltf.meshes() {
        info!("loading mesh {:?}", source_mesh.name());
        let mut vertices = Vec::new();
        let mut face_indices = Vec::new();
        let mut normals = Vec::new();
        let mut uv = Vec::new();
        let primitive = source_mesh
            .primitives()
            .filter(|p| p.mode() == Mode::Triangles)
            .nth(0)
            .expect("mesh must have at least one triangles primitive");

        let reader = primitive.reader(|b| Some(&buffers[b.index()]));
        let material = primitive.material();
        let material = if let Some(texture) = material.pbr_metallic_roughness().base_color_texture()
        {
            let idx = texture.texture().source().index();
            let image = images[idx].clone();
            let image = load_image(image, texture.texture().name().unwrap_or("<no name>"))?;
            lambertian_texture(Arc::new(Texture::Image(Image::new(image))))
        } else {
            let color = material.pbr_metallic_roughness().base_color_factor();
            lambertian(Point3::from_slice(&color))
        };

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
        info!(
            "loaded mesh {} with {} vertices, {} faces, {} normals and {} texture coordinates",
            source_mesh.name().unwrap_or("<no name>"),
            vertices.len(),
            face_indices.len(),
            normals.len(),
            uv.len(),
        );
        info!("assigned material {material:?}");

        meshes.push(TriangleMesh::new(
            vertices,
            face_indices,
            normals,
            uv,
            material,
        ));
    }

    Ok(meshes
        .into_iter()
        .flat_map(|m| m.faces().collect::<Vec<_>>())
        .map(|f| Object::TriangleRef(f))
        .collect())
}

pub mod builders {
    use std::sync::Arc;

    use crate::{
        material::Material,
        object::Object,
        vec3::{Point3, Vec3},
    };

    use super::TriangleMesh;

    pub fn default_normal(v0: Point3<f32>, v1: Point3<f32>, v2: Point3<f32>) -> Vec3<f32> {
        let e1 = v1 - v0;
        let e2 = v2 - v0;

        e1.cross(e2).unit()
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
        );

        mesh.faces().map(Object::TriangleRef).collect()
    }
}
