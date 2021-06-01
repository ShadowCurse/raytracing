use crate::vec3::Point3;

const PERLIN_POINT_COUNT: u32 = 256;

pub struct Perlin {
    random_float: Vec<f32>,
    perm_x: Vec<u32>,
    perm_y: Vec<u32>,
    perm_z: Vec<u32>,
}

impl Perlin {
    pub fn new() -> Self {
        use rand::distributions::Distribution;
        let mut rng = rand::thread_rng();
        let uniform = rand::distributions::Uniform::new(0.0, 1.0);

        Self {
            random_float: (0..PERLIN_POINT_COUNT)
                .map(|_| uniform.sample(&mut rng))
                .collect::<Vec<_>>(),
            perm_x: Self::perlin_generate_perm(),
            perm_y: Self::perlin_generate_perm(),
            perm_z: Self::perlin_generate_perm(),
        }
    }

    pub fn noise(&self, point: &Point3) -> f32 {
        let i = ((4.0 * point.x) as i32 & 255) as usize;
        let j = ((4.0 * point.y) as i32 & 255) as usize;
        let k = ((4.0 * point.z) as i32 & 255) as usize;
        self.random_float[(self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]) as usize]
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
}
