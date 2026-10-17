use crate::error::{Error, Result};
pub type Vec3 = [f64; 3];
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|k| a[k] + b[k])
}
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|k| a[k] - b[k])
}
pub fn scale(a: Vec3, s: f64) -> Vec3 {
    a.map(|v| v * s)
}
pub fn dot(a: Vec3, b: Vec3) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
pub fn norm(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}
pub fn finite(a: Vec3) -> bool {
    a.iter().all(|v| v.is_finite() && v.abs() <= 1e12)
}
pub fn rotate(q: [f64; 4], p: Vec3) -> Vec3 {
    let [x, y, z, w] = q;
    let t = [
        2.0 * (y * p[2] - z * p[1]),
        2.0 * (z * p[0] - x * p[2]),
        2.0 * (x * p[1] - y * p[0]),
    ];
    add(
        add(p, scale(t, w)),
        [
            y * t[2] - z * t[1],
            z * t[0] - x * t[2],
            x * t[1] - y * t[0],
        ],
    )
}
#[derive(Clone, Copy, Debug)]
pub struct SimBox {
    pub size: f64,
}
impl SimBox {
    pub fn new(size: f64) -> Result<Self> {
        if !size.is_finite() || !(1.0..=1e6).contains(&size) {
            return Err(Error::invalid("box side must be 1..1000000 Angstrom"));
        }
        Ok(Self { size })
    }
    pub fn contains(&self, p: Vec3, radius: f64) -> bool {
        finite(p) && p.iter().all(|&x| x >= radius && x <= self.size - radius)
    }
}
/// Platform-independent SplitMix64, Box-Muller normal sampling, Haar rotations.
pub struct Rng {
    state: u64,
}
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    pub fn uniform(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        (((z ^ (z >> 31)) >> 11) as f64 + 0.5) / 9007199254740992.0
    }
    pub fn normal(&mut self) -> f64 {
        (-2.0 * self.uniform().ln()).sqrt() * (std::f64::consts::TAU * self.uniform()).cos()
    }
    pub fn rotation(&mut self) -> [f64; 4] {
        let u = self.uniform();
        let a = std::f64::consts::TAU * self.uniform();
        let b = std::f64::consts::TAU * self.uniform();
        [
            (1.0 - u).sqrt() * a.sin(),
            (1.0 - u).sqrt() * a.cos(),
            u.sqrt() * b.sin(),
            u.sqrt() * b.cos(),
        ]
    }
}
