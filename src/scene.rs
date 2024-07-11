use std::path::Path;
use std::sync::Arc;

use glam::{Affine3A, Mat4};
use gltf::camera::Projection;
use gltf::mesh::Mode;
use image::{DynamicImage, ImageBuffer, Luma, LumaA, Rgb, Rgba};
use tracing::{debug, info};

use crate::bvh::BvhNode;
use crate::material::{DiffuseLight, Material};
use crate::object::triangle_mesh::TriangleMesh;
use crate::object::{Object, World};
use crate::texture::{Image, SolidColor, Texture, TextureCoordinates};
use crate::vec3::{Color, Point3, Vec3};
use crate::Result;

#[derive(Debug)]
pub struct SceneDescription {
    pub root_object: Object,
    pub cameras: Vec<CameraSettings>,
    pub background_color: Color,
    pub render: RenderSettings,
}

impl SceneDescription {
    pub fn camera(&self, index: usize) -> CameraSettings {
        self.cameras.get(index).cloned().unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct RenderSettings {
    pub image_width: u32,
    pub image_height: u32,
    pub selected_camera: usize,
    pub max_depth: u32,
    pub samples_per_pixel: u32,
}

#[derive(Debug, Clone)]
pub struct CameraSettings {
    pub name: Option<String>,
    pub y_fov: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub transform: Affine3A,
    pub focus_dist: f32,
    pub defocus_angle: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            name: Default::default(),
            y_fov: 80.0f32.to_radians(),
            z_near: 0.0001,
            z_far: f32::INFINITY,
            transform: Affine3A::IDENTITY,
            focus_dist: 10.0,
            defocus_angle: 0.0,
        }
    }
}

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

fn read_mesh(
    source_mesh: &gltf::Mesh,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    transform: Affine3A,
) -> Result<TriangleMesh> {
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
    let emissive_factor = Vec3::from(material.emissive_factor());

    let color_texture =
        if let Some(texture) = material.pbr_metallic_roughness().base_color_texture() {
            let idx = texture.texture().source().index();
            let image = images[idx].clone();
            let image = load_image(image, texture.texture().name().unwrap_or("<no name>"))?;
            Arc::new(Texture::Image(Image::new(image)))
        } else {
            let color = material.pbr_metallic_roughness().base_color_factor();
            Arc::new(Texture::SolidColor(SolidColor {
                albedo: Vec3::new(color[0], color[1], color[2]),
            }))
        };

    // TODO actual PBR shader
    let material = if emissive_factor != Vec3::ZERO {
        Arc::new(Material::DiffuseLight(DiffuseLight {
            texture: Texture::solid_color(Vec3::ONE),
            strength: material.emissive_strength().unwrap_or(1.0),
        }))
    } else {
        Material::lambertian_texture(color_texture)
    };

    if let Some(positions) = reader.read_positions() {
        let positions: Vec<_> = positions.collect();
        debug!("untrasformed vertices: {:#?}", &positions);
        vertices.extend(
            positions
                .into_iter()
                .map(|p| transform.transform_point3a(Point3::from(p))),
        );
    }

    debug!("transformed vertices {:#?}", &vertices[0..100]);

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
    info!("assigned material {material:#?}");

    Ok(TriangleMesh::new(
        vertices,
        face_indices,
        normals,
        uv,
        material,
    ))
}

pub fn load_from_gltf(
    path: impl AsRef<Path>,
    bvh_disabled: bool,
    render_settings: RenderSettings,
) -> Result<SceneDescription> {
    let (gltf, buffers, images) = gltf::import(path)?;
    let mut meshes = Vec::new();
    let mut cameras = vec![];

    // TODO this would have to walk the entire scene graph
    for node in gltf.nodes() {
        let matrix = Mat4::from_cols_array_2d(&node.transform().matrix());
        let transform = Affine3A::from_mat4(matrix);

        if let Some(mesh) = node.mesh() {
            let mesh = read_mesh(&mesh, &buffers, &images, transform)?;
            meshes.push(mesh);
        }

        if let Some(camera) = node.camera() {
            if let Projection::Perspective(projection) = camera.projection() {
                cameras.push(CameraSettings {
                    name: camera.name().map(From::from),
                    y_fov: projection.yfov(),
                    z_near: projection.znear(),
                    z_far: projection.zfar().unwrap_or(f32::INFINITY),
                    transform,
                    focus_dist: 10.0,
                    defocus_angle: 0.0,
                });
            }
        }
    }

    let objects = meshes
        .into_iter()
        .flat_map(|m| m.faces().collect::<Vec<_>>())
        .map(|f| Object::TriangleRef(f))
        .collect();

    let root_object = if bvh_disabled {
        Object::World(World::new(objects))
    } else {
        Object::BvhNode(BvhNode::from(objects))
    };

    debug!("cameras: {cameras:#?}");

    Ok(SceneDescription {
        root_object,
        cameras,
        background_color: Vec3::ZERO,
        render: render_settings,
    })
}
