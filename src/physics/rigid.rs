//! Inertia from atomic point masses; torque-free world angular momentum is conserved.
use crate::{
    geometry::*,
    molecule::{AtomDefinition, MoleculeInstance},
};
#[derive(Debug)]
pub struct Inertia {
    pub moments: Vec3,
    /// Principal axes in body coordinates (columns).
    pub axes: [Vec3; 3],
}
impl Inertia {
    pub fn from_atoms(atoms: &[AtomDefinition]) -> Self {
        let mut a = [[0.; 3]; 3];
        for atom in atoms {
            for (i, row) in a.iter_mut().enumerate() {
                for (j, v) in row.iter_mut().enumerate() {
                    *v += atom.element.mass
                        * (if i == j {
                            dot(atom.local, atom.local)
                        } else {
                            0.
                        } - atom.local[i] * atom.local[j]);
                }
            }
        }
        let mut v = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        for _ in 0..64 {
            let (p, q) = [(0, 1), (0, 2), (1, 2)]
                .into_iter()
                .max_by(|&(i, j), &(k, l)| a[i][j].abs().total_cmp(&a[k][l].abs()))
                .unwrap();
            if a[p][q].abs()
                < 1e-14
                    * a.iter()
                        .enumerate()
                        .map(|(i, r)| r[i].abs())
                        .sum::<f64>()
                        .max(1.)
            {
                break;
            }
            let theta = 0.5 * (2. * a[p][q]).atan2(a[q][q] - a[p][p]);
            let (s, c) = theta.sin_cos();
            let app = a[p][p];
            let aqq = a[q][q];
            let apq = a[p][q];
            a[p][p] = c * c * app - 2. * s * c * apq + s * s * aqq;
            a[q][q] = s * s * app + 2. * s * c * apq + c * c * aqq;
            a[p][q] = 0.;
            a[q][p] = 0.;
            #[allow(clippy::needless_range_loop)]
            // Symmetric row and column updates cannot borrow one row alone.
            for k in 0..3 {
                if k != p && k != q {
                    let x = a[k][p];
                    let y = a[k][q];
                    a[k][p] = c * x - s * y;
                    a[p][k] = a[k][p];
                    a[k][q] = s * x + c * y;
                    a[q][k] = a[k][q];
                }
            }
            for row in &mut v {
                let x = row[p];
                let y = row[q];
                row[p] = c * x - s * y;
                row[q] = s * x + c * y;
            }
        }
        let max = (0..3).map(|i| a[i][i]).fold(0., f64::max);
        Self {
            moments: std::array::from_fn(|i| if a[i][i] > max * 1e-12 { a[i][i] } else { 0. }),
            axes: std::array::from_fn(|i| std::array::from_fn(|j| v[j][i])),
        }
    }
    pub fn inverse(&self, l: Vec3) -> Vec3 {
        (0..3)
            .filter(|&i| self.moments[i] > 0.)
            .fold([0.; 3], |s, i| {
                add(
                    s,
                    scale(self.axes[i], dot(self.axes[i], l) / self.moments[i]),
                )
            })
    }
    pub fn dof(&self) -> u32 {
        self.moments.iter().filter(|&&x| x > 0.).count() as u32
    }
    pub fn sample_momentum(&self, rng: &mut Rng, kt: f64) -> Vec3 {
        (0..3).fold([0.; 3], |s, i| {
            add(
                s,
                scale(self.axes[i], rng.normal() * (kt * self.moments[i]).sqrt()),
            )
        })
    }
}
/// Lie midpoint orientation with bounded angular increments. L is fixed in world frame.
pub fn advance(m: &mut MoleculeInstance, dt: f64) {
    let q = m.rotation;
    let mut half = q;
    for _ in 0..12 {
        let omega = rotate(
            half,
            m.definition
                .inertia
                .inverse(rotate(conjugate(half), m.angular_momentum)),
        );
        half = advance_rotation(q, omega, dt * 0.5);
    }
    let omega = rotate(
        half,
        m.definition
            .inertia
            .inverse(rotate(conjugate(half), m.angular_momentum)),
    );
    m.rotation = advance_rotation(q, omega, dt);
}
