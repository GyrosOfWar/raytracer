use bvh::BvhNode;
use camera::Camera;
use helpers::{random, random_range};
use material::{dielectric, lambertian, metal};
use object::{Object, Sphere, World};
use std::{
    fs::File,
    io::{self, BufWriter},
};
use vec3::Point3;

mod aabb;
mod bvh;
mod camera;
mod helpers;
mod material;
mod object;
mod ppm;
mod range;
mod ray;
mod vec3;

fn main() -> io::Result<()> {
    let mut objects = vec![];
    let ground_material = lambertian(Point3::new(0.5, 0.5, 0.5));
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

    let mut world = World::new(objects);
    let world = BvhNode::from_world(&mut world);

    let camera = Camera::new(
        1280,
        720,
        500,
        Point3::new(13.0, 2.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        0.6,
        10.0,
    );
    let image = camera.render(&world);
    let mut file = BufWriter::new(File::create("image.ppm")?);
    image.write_to_ppm(&mut file)?;

    Ok(())
}
