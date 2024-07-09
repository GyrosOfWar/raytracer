use std::path::Path;
use std::sync::Arc;

use gltf::mesh::Mode;
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use nalgebra::Projective3;
use tracing::info;

use crate::bvh::BvhNode;
use crate::material::helpers::{lambertian, lambertian_texture};
use crate::object::triangle_mesh::TriangleMesh;
use crate::object::Object;
use crate::texture::{Image, Texture, TextureCoordinates};
use crate::vec3::{Color, Point3, Vec3};
use crate::Result;

#[derive(Debug)]
pub struct SceneDescription {
    pub image_width: u32,
    pub image_height: u32,
    pub root_object: Object,
    pub cameras: Vec<CameraSettings>,
    pub selected_camera: usize,
    pub background_color: Color,
    pub max_depth: u32,
    pub samples_per_pixel: u32,
}

#[derive(Debug)]
pub struct CameraSettings {}

fn load_image(image: gltf::image::Data, name: &str) -> Result<DynamicImage> {
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

pub fn load_from_gltf(path: impl AsRef<Path>) -> Result<SceneDescription> {
    let (gltf, buffers, images) = gltf::import(path)?;
    let mut meshes = Vec::new();

    // TODO need to iterate over the nodes to also get the transformations

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
            lambertian(Vec3::from([color[0], color[1], color[2]]))
        };

        if let Some(positions) = reader.read_positions() {
            vertices.extend(positions.map(|p| Point3::from(p)));
        }

        if let Some(indices) = reader.read_indices() {
            let indices: Vec<_> = indices.into_u32().collect();
            for chunk in indices.chunks(3) {
                face_indices.push((chunk[0], chunk[1], chunk[2]));
            }
        }

        if let Some(normals_iter) = reader.read_normals() {
            normals.extend(normals_iter.map(|n| Vec3::from(n)));
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

        let transform = Projective3::identity();

        meshes.push(TriangleMesh::new(
            vertices,
            face_indices,
            normals,
            uv,
            material,
            transform,
        ));
    }

    let objects = meshes
        .into_iter()
        .flat_map(|m| m.faces().collect::<Vec<_>>())
        .map(|f| Object::TriangleRef(f))
        .collect();

    Ok(SceneDescription {
        root_object: Object::BvhNode(BvhNode::from(objects)),
        image_width: 1280,
        image_height: 720,
        max_depth: 50,
        cameras: vec![],
        samples_per_pixel: 100,
        selected_camera: 0,
        background_color: Vec3::new(0.5, 0.5, 0.5),
    })
}
