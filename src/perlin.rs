use crate::vec3::Point3;

const PERLIN_POINT_COUNT : u32 = 255;

pub struct Perlin {
    random_flot: Vec<f32>,
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
            random_flot: (0..PERLIN_POINT_COUNT).map(|x| 
                                uniform.sample(&mut rng)
                             ).collect::<Vec<_>>(),
            perm_x: Self::perlin_generate_perm(),
            perm_y: Self::perlin_generate_perm(),
            perm_z: Self::perlin_generate_perm(),
        }
    }

    pub fn noise(&self, point: &Point3) -> f32 {
        let i = (4.0 * point.x) as u32 & 255;
        let j = (4.0 * point.y) as u32 & 255;
        let k = (4.0 * point.z) as u32 & 255;
        self.random_flot[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
    }

    fn perlin_generate_perm() -> Vec<u32> {
        let mut p = (0..PERLIN_POINT_COUNT).map(|x| x).collect::<Vec<u32>>(); //vec![0; PERLIN_POINT_COUNT as usize];

        use rand::distributions::Distribution;
        let mut rng = rand::thread_rng();
        for i in (0..PERLIN_POINT_COUNT).rev() {
            let uniform = rand::distributions::Uniform::new(0, i);
            let target = uniform.sample(&mut rng);
            std::mem::swap(p[i], p[target]);
        }
        p
    }
}
