use num_traits::Zero;

use crate::{
    camera::{Camera, CameraParams},
    material::{dielectric, lambertian, lambertian_texture, metal},
    object::{Object, Quad, Sphere},
    random::{random, random_range},
    texture::{checkerboard, image, solid},
    vec3::{self, Point3, Vec3},
};

pub fn lots_of_spheres() -> (Camera, Vec<Object>) {
    let mut objects = vec![];
    let ground_material = lambertian_texture(checkerboard(
        3.2,
        solid(Point3::new(0.2, 0.3, 0.1)),
        solid(Point3::new(0.9, 0.9, 0.9)),
    ));
    objects.push(Object::Sphere(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let float = random();
            let center = Point3::new(
                (a as f32) + 0.9 * random(),
                0.2,
                (b as f32) + 0.9 * random(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let object = match float {
                    0.0..=0.8 => {
                        let albedo = vec3::random::gen() * vec3::random::gen();
                        let material = lambertian(albedo);
                        Sphere::new(center, 0.2, material)
                    }
                    0.8..=0.95 => {
                        let albedo = vec3::random::gen_range(0.5, 1.0);
                        let fuzz = random_range(0.0, 0.5);
                        let material = metal(albedo, fuzz);
                        Sphere::new(center, 0.2, material)
                    }
                    _ => Sphere::new(center, 0.2, dielectric(1.5)),
                };

                objects.push(Object::Sphere(object));
            }
        }
    }

    let material1 = dielectric(1.5);
    objects.push(Object::Sphere(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1,
    )));

    let material2 = lambertian(Point3::new(0.4, 0.2, 0.1));
    objects.push(Object::Sphere(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2,
    )));

    let material3 = metal(Point3::new(0.7, 0.6, 0.5), 0.0);
    objects.push(Object::Sphere(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3,
    )));

    let params = CameraParams {
        look_from: Point3::new(13.0, 2.0, 3.0),
        defocus_angle: 1.2,
        focus_dist: 10.0,
        vertical_fov: 20.0,
        samples_per_pixel: 500,
        image_size: (2560, 1440),
        ..Default::default()
    };

    let camera = Camera::new(params);

    (camera, objects)
}

pub fn earth() -> (Camera, Vec<Object>) {
    let texture = image("textures/earthmap.jpeg");
    let sphere = Object::Sphere(Sphere::new(
        Point3::zero(),
        1.0,
        lambertian_texture(texture),
    ));

    let params = CameraParams {
        vertical_fov: 20.0,
        samples_per_pixel: 100,
        look_from: Point3::new(0.0, 0.0, 12.0),
        ..Default::default()
    };
    let camera = Camera::new(params);

    (camera, vec![sphere])
}

pub fn quads() -> (Camera, Vec<Object>) {
    let material = lambertian(Point3::new(1.0, 0.2, 0.2));
    let objects = vec![
        // Object::Quad(Quad::new(
        //     Point3::new(-3.0, -2.0, 5.0),
        //     Vec3::new(0.0, 0.0, -4.0),
        //     Vec3::new(0.0, 4.0, 0.0),
        //     material.clone(),
        // )),
        Object::Quad(Quad::new(
            Point3::new(-2.0, -2.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
            material.clone(),
        )),
        // Object::Quad(Quad::new(
        //     Point3::new(3.0, -2.0, 1.0),
        //     Vec3::new(0.0, 0.0, 4.0),
        //     Vec3::new(0.0, 4.0, 0.0),
        //     material.clone(),
        // )),
        // Object::Quad(Quad::new(
        //     Point3::new(-2.0, 3.0, 1.0),
        //     Vec3::new(4.0, 0.0, 0.0),
        //     Vec3::new(0.0, 0.0, 4.0),
        //     material.clone(),
        // )),
        // Object::Quad(Quad::new(
        //     Point3::new(-2.0, -3.0, 5.0),
        //     Vec3::new(4.0, 0.0, 0.0),
        //     Vec3::new(0.0, 0.0, -4.0),
        //     material.clone(),
        // )),
    ];

    let params = CameraParams {
        vertical_fov: 80.0,
        look_from: Point3::new(0.0, 0.0, 9.0),
        image_size: (400, 400),
        samples_per_pixel: 100,
        ..Default::default()
    };

    let camera = Camera::new(params);

    (camera, objects)
}
