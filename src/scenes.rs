use num_traits::Zero;

use crate::{
    camera::Camera,
    helpers::{random, random_range},
    material::{dielectric, lambertian, lambertian_texture, metal},
    object::{Object, Sphere},
    texture::{checkerboard, image, solid},
    vec3::{self, Point3},
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

    let camera = Camera::new(
        1280,
        720,
        100,
        Point3::new(13.0, 2.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        0.6,
        10.0,
    );

    (camera, objects)
}

pub fn earth() -> (Camera, Vec<Object>) {
    let texture = image("textures/earthmap.jpeg");
    let sphere = Object::Sphere(Sphere::new(
        Point3::zero(),
        1.0,
        lambertian_texture(texture),
    ));

    let camera = Camera::new(
        1280,
        720,
        50,
        Point3::new(0.0, 0.0, 12.0),
        Point3::new(0.0, 0.0, 0.0),
        0.0,
        10.0,
    );

    (camera, vec![sphere])
}
