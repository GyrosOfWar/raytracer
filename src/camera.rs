use crate::random::random;
use crate::ray::Ray;
use crate::scene::CameraSettings;
use crate::vec3::{self, Point3, Vec3};

pub struct Camera {
    center: Point3,
    pixel_00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
    defocus_angle: f32,
}

impl Camera {
    pub fn new(settings: CameraSettings, width: u32, height: u32) -> Self {
        let v_up = Vec3::Y;

        // gltf FoV is already in radians
        let theta = settings.y_fov;
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * settings.focus_dist;
        let viewport_width = viewport_height * (width as f32 / height as f32);

        let look_from = settings.transform.transform_point3a(Point3::ZERO);
        let look_at = settings.transform.transform_point3a(Point3::NEG_Z);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (look_from - look_at).normalize();
        let u = v_up.cross(w).normalize();
        let v = w.cross(u);

        let camera_center = look_from;
        let viewport_u = u * viewport_width;
        let viewport_v = -v * viewport_height;

        let pixel_delta_u = viewport_u / width as f32;
        let pixel_delta_v = viewport_v / height as f32;

        let viewport_upper_left =
            camera_center - (w * settings.focus_dist) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel_00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

        let defocus_radius =
            settings.focus_dist * (settings.defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Camera {
            center: camera_center,
            pixel_00_loc,
            pixel_delta_u,
            pixel_delta_v,
            defocus_disk_u,
            defocus_disk_v,
            defocus_angle: settings.defocus_angle,
        }
    }

    /// Construct a camera ray originating from the origin and directed at randomly sampled
    /// point around the pixel location `(i, j)`.
    pub fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = self.sample_square();
        let pixel_sample = self.pixel_00_loc
            + (self.pixel_delta_u * (i as f32 + offset.x))
            + (self.pixel_delta_v * (j as f32 + offset.y));

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    /// Returns a random point in the camera defocus disk.
    fn defocus_disk_sample(&self) -> Point3 {
        let p = vec3::random::gen_unit_disk();
        self.center + (self.defocus_disk_u * p.x) + (self.defocus_disk_v * p.y)
    }

    // Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square.
    fn sample_square(&self) -> Vec3 {
        Vec3::new(random() - 0.5, random() - 0.5, 0.0)
    }
}
