use crate::ray::Ray;
use crate::vec3::*;

pub struct Camera {
    pub origin: Point3,
    pub lower_left_corner: Point3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub lens_radius: f32,
    pub time0: f32,
    pub time1: f32,
}

impl Camera {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        look_from: &Point3,
        look_at: &Point3,
        v_up: &Vec3,
        vfov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32,
        time0: f32,
        time1: f32,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = (look_from - look_at).unit();
        let u = v_up.cross(&w).unit();
        let v = w.cross(&u);

        let origin = *look_from;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - 0.5 * horizontal - 0.5 * vertical - focus_dist * w;
        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            w,
            lens_radius: aperture / 2.0,
            time0,
            time1,
        }
    }

    pub fn get_ray(&self, x: f32, y: f32) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;

        let dir =
            self.lower_left_corner + x * self.horizontal + y * self.vertical - self.origin - offset;

        use rand::distributions::Distribution;
        let mut rng = rand::thread_rng();
        let uniform = rand::distributions::Uniform::new(self.time0, self.time1);
        Ray::new(self.origin + offset, dir, uniform.sample(&mut rng))
    }
}
