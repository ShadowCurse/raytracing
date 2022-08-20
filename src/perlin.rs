use crate::vec3::{Point3, Vec3};

const PERLIN_POINT_COUNT: u32 = 256;

pub struct Perlin {
    random_vec: Vec<Vec3>,
    perm_x: Vec<u32>,
    perm_y: Vec<u32>,
    perm_z: Vec<u32>,
}

impl Default for Perlin {
    fn default() -> Self {
        Self {
            random_vec: (0..PERLIN_POINT_COUNT)
                .map(|_| Vec3::random(-1.0, 1.0).unit())
                .collect::<Vec<_>>(),
            perm_x: Self::perlin_generate_perm(),
            perm_y: Self::perlin_generate_perm(),
            perm_z: Self::perlin_generate_perm(),
        }
    }
}

impl Perlin {
    pub fn noise(&self, point: &Point3) -> f32 {
        let u = point.x - point.x.floor();
        let v = point.y - point.y.floor();
        let w = point.z - point.z.floor();

        let i = point.x.floor() as i32;
        let j = point.y.floor() as i32;
        let k = point.z.floor() as i32;

        let mut c = vec![vec![vec![Vec3::default(); 2]; 2]; 2];

        for (di, vec_i) in c.iter_mut().enumerate() {
            for (dj, vec_j) in vec_i.iter_mut().enumerate() {
                for (dk, val) in vec_j.iter_mut().enumerate() {
                    *val = self.random_vec[(self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize])
                        as usize]
                }
            }
        }
        Self::trilinear_interp(&c, u, v, w)
    }

    pub fn turb(&self, point: &Point3, depth: u32) -> f32 {
        let mut accum = 0.0;
        let mut tmp_p = *point;
        let mut weight = 1.0;
        for _ in 0..depth {
            accum += weight * Self::noise(self, &tmp_p);
            weight *= 0.5;
            tmp_p *= 2.0;
        }
        accum.abs()
    }

    fn perlin_generate_perm() -> Vec<u32> {
        let mut p = (0..PERLIN_POINT_COUNT).collect::<Vec<u32>>();

        use rand::distributions::Distribution;
        let mut rng = rand::thread_rng();
        for i in (1..PERLIN_POINT_COUNT).rev() {
            let uniform = rand::distributions::Uniform::new(0, i);
            let target = uniform.sample(&mut rng);
            p.swap(i as usize, target as usize);
        }
        p
    }

    fn trilinear_interp(c: &[Vec<Vec<Vec3>>], u: f32, v: f32, w: f32) -> f32 {
        let uu = u.powi(2) * (3.0 - 2.0 * u);
        let vv = v.powi(2) * (3.0 - 2.0 * v);
        let ww = w.powi(2) * (3.0 - 2.0 * w);

        let mut accum: f32 = 0.0;
        for (i, vec_i) in c.iter().enumerate() {
            for (j, vec_j) in vec_i.iter().enumerate() {
                for (k, val) in vec_j.iter().enumerate() {
                    let weight = Vec3::new(u - i as f32, v - j as f32, w - k as f32);
                    accum += (i as f32 * uu + (1.0 - i as f32) * (1.0 - uu))
                        * (j as f32 * vv + (1.0 - j as f32) * (1.0 - vv))
                        * (k as f32 * ww + (1.0 - k as f32) * (1.0 - ww))
                        * val.dot(&weight);
                }
            }
        }
        accum
    }
}
