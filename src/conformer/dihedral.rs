use crate::{
    error::{Code, Error, Result},
    geometry::*,
    molecule::MoleculeDefinition,
};
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn validate(coordinates: &[Vec3], ids: [usize; 4]) -> Result<()> {
    if ids.iter().any(|&i| i >= coordinates.len())
        || (0..4).any(|i| (i + 1..4).any(|j| ids[i] == ids[j]))
        || coordinates.iter().any(|v| !finite(*v))
    {
        return Err(Error::invalid("invalid dihedral coordinates/indices"));
    }
    Ok(())
}
/// Signed radians in [-pi,pi], using the directed b→c axis.
pub fn measure(coordinates: &[Vec3], ids: [usize; 4]) -> Result<f64> {
    validate(coordinates, ids)?;
    let [a, b, c, d] = ids;
    let axis = sub(coordinates[c], coordinates[b]);
    let length = norm(axis);
    if length < 1e-12 {
        return Err(Error::invalid("degenerate dihedral axis"));
    }
    let u = scale(axis, 1. / length);
    let left = sub(coordinates[a], coordinates[b]);
    let right = sub(coordinates[d], coordinates[c]);
    let v = sub(left, scale(u, dot(left, u)));
    let w = sub(right, scale(u, dot(right, u)));
    if norm(v) < 1e-12 || norm(w) < 1e-12 {
        return Err(Error::invalid("collinear dihedral is undefined"));
    }
    Ok(dot(cross(u, v), w).atan2(dot(v, w)))
}
/// Returns edited coordinates; rotates only c-side graph component. Rings rejected.
/// This geometry operation performs neither steric checks nor energy optimization.
pub fn set(
    definition: &MoleculeDefinition,
    coordinates: &[Vec3],
    ids: [usize; 4],
    target_radians: f64,
) -> Result<Vec<Vec3>> {
    validate(coordinates, ids)?;
    if coordinates.len() != definition.atoms.len() || !target_radians.is_finite() {
        return Err(Error::invalid("invalid dihedral target/atom count"));
    }
    let [a, b, c, d] = ids;
    let edge = |i, j| {
        definition
            .bonds
            .iter()
            .any(|e| (e.a == i && e.b == j) || (e.a == j && e.b == i))
    };
    if !edge(a, b) || !edge(b, c) || !edge(c, d) {
        return Err(Error::invalid(
            "dihedral requires three explicit consecutive bonds",
        ));
    }
    let mut seen = vec![false; coordinates.len()];
    let mut stack = vec![c];
    seen[c] = true;
    while let Some(i) = stack.pop() {
        for e in &definition.bonds {
            if (e.a == b && e.b == c) || (e.a == c && e.b == b) {
                continue;
            }
            let j = if e.a == i {
                e.b
            } else if e.b == i {
                e.a
            } else {
                continue;
            };
            if !seen[j] {
                seen[j] = true;
                stack.push(j);
            }
        }
    }
    if seen[b] {
        return Err(Error::new(
            Code::Unsupported,
            "ring torsions require a ring-specific conformer algorithm",
        ));
    }
    let angle = (target_radians - measure(coordinates, ids)?).rem_euclid(std::f64::consts::TAU);
    let origin = coordinates[b];
    let axis = sub(coordinates[c], origin);
    let u = scale(axis, 1. / norm(axis));
    let (s, c) = angle.sin_cos();
    let mut edited = coordinates.to_vec();
    for (i, p) in edited.iter_mut().enumerate() {
        if seen[i] {
            let v = sub(*p, origin);
            *p = add(
                origin,
                add(
                    add(scale(v, c), scale(cross(u, v), s)),
                    scale(u, dot(u, v) * (1. - c)),
                ),
            );
        }
    }
    Ok(edited)
}
