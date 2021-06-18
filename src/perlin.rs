use crate::vec3::{Point3, Vec3};

const PERLIN_POINT_COUNT: u32 = 256;

pub struct Perlin {
    random_vec: Vec<Vec3>,
    perm_x: Vec<u32>,
    perm_y: Vec<u32>,
    perm_z: Vec<u32>,
}

impl Perlin {
    pub fn new() -> Self {
        Self {
            random_vec: (0..PERLIN_POINT_COUNT)
                .map(|_| Vec3::random(-1.0, 1.0).unit())
                .collect::<Vec<_>>(),
            perm_x: Self::perlin_generate_perm(),
            perm_y: Self::perlin_generate_perm(),
            perm_z: Self::perlin_generate_perm(),
        }
    }

    pub fn noise(&self, point: &Point3) -> f32 {
        let u = point.x - point.x.floor();
        let v = point.y - point.y.floor();
        let w = point.z - point.z.floor();

        let i = point.x.floor() as i32;
        let j = point.y.floor() as i32;
        let k = point.z.floor() as i32;

        let mut c = vec![vec![vec![Vec3::default(); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.random_vec[(self.perm_x[((i + di as i32) & 255) as usize]
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
        let mut tmp_p = point.clone();
        let mut weight = 1.0;
        for _ in 0..depth {
            accum += weight * Self::noise(self, &tmp_p);
            weight *= 0.5;
            tmp_p *= 2.0;
        }
        accum.abs()
    }

    fn perlin_generate_perm() -> Vec<u32> {
        let mut p = (0..PERLIN_POINT_COUNT).map(|x| x).collect::<Vec<u32>>();

        use rand::distributions::Distribution;
        let mut rng = rand::thread_rng();
        for i in (1..PERLIN_POINT_COUNT).rev() {
            let uniform = rand::distributions::Uniform::new(0, i);
            let target = uniform.sample(&mut rng);
            p.swap(i as usize, target as usize);
        }
        p
    }

    fn trilinear_interp(c: &Vec<Vec<Vec<Vec3>>>, u: f32, v: f32, w: f32) -> f32 {
        let uu = u.powi(2) * (3.0 - 2.0 * u);
        let vv = v.powi(2) * (3.0 - 2.0 * v);
        let ww = w.powi(2) * (3.0 - 2.0 * w);

        let mut accum: f32 = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight = Vec3::new(u - i as f32, v - j as f32, w - k as f32);
                    accum += (i as f32 * uu + (1.0 - i as f32) * (1.0 - uu))
                        * (j as f32 * vv + (1.0 - j as f32) * (1.0 - vv))
                        * (k as f32 * ww + (1.0 - k as f32) * (1.0 - ww))
                        * c[i][j][k].dot(&weight);
                }
            }
        }
        accum
    }
}
