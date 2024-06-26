use camera::Camera;
use std::{
    fs::File,
    io::{self, BufWriter},
    time::Instant,
};
use trace::{Object, Sphere, World};
use vec3::Point3;

mod camera;
mod helpers;
mod ppm;
mod ray;
mod trace;
mod vec3;

fn main() -> io::Result<()> {
    let world = World::new(vec![
        Object::Sphere(Sphere {
            center: Point3::new(0.0, 0.0, -1.0),
            radius: 0.5,
        }),
        Object::Sphere(Sphere {
            center: Point3::new(0.0, -100.5, -1.0),
            radius: 100.0,
        }),
    ]);

    let camera = Camera::new(720, 480);
    let image = camera.render(&world);

    let start = Instant::now();
    let mut file = BufWriter::new(File::create("image.ppm")?);
    image.write_to_ppm(&mut file)?;
    let elapsed = start.elapsed();
    println!("writing image took {elapsed:?}");

    Ok(())
}
