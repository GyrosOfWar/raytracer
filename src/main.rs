use camera::Camera;
use material::{dielectric, lambertian, metal};
use std::{
    fs::File,
    io::{self, BufWriter},
    time::Instant,
};
use trace::{Object, Sphere, World};
use vec3::{Point3, Vec3};

mod bvh;
mod camera;
mod helpers;
mod material;
mod ppm;
mod ray;
mod trace;
mod vec3;

fn main() -> io::Result<()> {
    let material_ground = lambertian(Vec3::new(0.8, 0.8, 0.0));
    let material_center = lambertian(Vec3::new(0.1, 0.2, 0.5));
    let material_left = dielectric(1.50);
    let material_bubble = dielectric(1.00 / 1.50);
    let material_right = metal(Vec3::new(0.8, 0.6, 0.2), 0.0);

    let world = World::new(vec![
        Object::Sphere(Sphere::new(
            Vec3::new(0.0, -100.5, -1.0),
            100.0,
            material_ground,
        )),
        Object::Sphere(Sphere::new(Vec3::new(0.0, 0.0, -1.2), 0.5, material_center)),
        Object::Sphere(Sphere::new(Vec3::new(-1.0, 0.0, -1.0), 0.5, material_left)),
        Object::Sphere(Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.4,
            material_bubble,
        )),
        Object::Sphere(Sphere::new(Vec3::new(1.0, 0.0, -1.0), 0.5, material_right)),
    ]);

    let camera = Camera::new(
        1280,
        720,
        200,
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, -1.0),
        7.0,
        2.0,
    );
    let image = camera.render(&world);

    let start = Instant::now();
    let mut file = BufWriter::new(File::create("image.ppm")?);
    image.write_to_ppm(&mut file)?;
    let elapsed = start.elapsed();
    println!("writing image took {elapsed:?}");

    Ok(())
}
