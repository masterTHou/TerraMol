use crate::{
    error::{Error, Result},
    geometry::{norm, sub},
    molecule::{AtomDefinition, Bond, MoleculeDefinition},
};
#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub symbol: &'static str,
    pub number: u32,
    pub mass: f64,
    pub covalent: f64,
    pub vdw: f64,
    pub cpk: [f32; 3],
}
pub const ELEMENTS: [Element; 8] = [
    Element {
        symbol: "H",
        number: 1,
        mass: 1.008,
        covalent: 0.31,
        vdw: 1.20,
        cpk: [1., 1., 1.],
    },
    Element {
        symbol: "C",
        number: 6,
        mass: 12.011,
        covalent: 0.76,
        vdw: 1.70,
        cpk: [0.2, 0.2, 0.2],
    },
    Element {
        symbol: "N",
        number: 7,
        mass: 14.007,
        covalent: 0.71,
        vdw: 1.55,
        cpk: [0.19, 0.31, 0.97],
    },
    Element {
        symbol: "O",
        number: 8,
        mass: 15.999,
        covalent: 0.66,
        vdw: 1.52,
        cpk: [1., 0.05, 0.05],
    },
    Element {
        symbol: "F",
        number: 9,
        mass: 18.998,
        covalent: 0.57,
        vdw: 1.47,
        cpk: [0.56, 0.88, 0.31],
    },
    Element {
        symbol: "S",
        number: 16,
        mass: 32.06,
        covalent: 1.05,
        vdw: 1.80,
        cpk: [1., 1., 0.19],
    },
    Element {
        symbol: "P",
        number: 15,
        mass: 30.974,
        covalent: 1.07,
        vdw: 1.80,
        cpk: [1., 0.5, 0.],
    },
    Element {
        symbol: "Cl",
        number: 17,
        mass: 35.45,
        covalent: 1.02,
        vdw: 1.75,
        cpk: [0.12, 0.94, 0.12],
    },
];
pub fn element(symbol: &str) -> Result<Element> {
    ELEMENTS
        .iter()
        .find(|e| e.symbol == symbol)
        .copied()
        .ok_or_else(|| Error::invalid(format!("unsupported element: {symbol}")))
}
pub fn by_number(number: u32) -> Result<Element> {
    ELEMENTS
        .iter()
        .find(|e| e.number == number)
        .copied()
        .ok_or_else(|| Error::invalid("unsupported atomic number"))
}
/// XYZ only provides geometry. Inferred edges have unknown bond order and aromaticity.
pub fn parse_xyz(text: &str) -> Result<MoleculeDefinition> {
    let mut lines = text.lines();
    let count: usize = lines
        .next()
        .ok_or_else(|| Error::invalid("missing count"))?
        .trim()
        .parse()
        .map_err(|_| Error::invalid("invalid count"))?;
    if count == 0 || count > 100_000 {
        return Err(Error::invalid("XYZ count must be 1..100000"));
    }
    let name = lines
        .next()
        .ok_or_else(|| Error::invalid("missing XYZ comment"))?;
    let mut atoms = Vec::with_capacity(count);
    for _ in 0..count {
        let fields: Vec<_> = lines
            .next()
            .ok_or_else(|| Error::invalid("truncated XYZ"))?
            .split_whitespace()
            .collect();
        if fields.len() != 4 {
            return Err(Error::invalid("expected element x y z"));
        }
        let mut local = [0.; 3];
        for k in 0..3 {
            local[k] = fields[k + 1]
                .parse()
                .map_err(|_| Error::invalid("invalid coordinate"))?;
        }
        atoms.push(AtomDefinition {
            element: element(fields[0])?,
            local,
        });
    }
    if lines.any(|l| !l.trim().is_empty()) {
        return Err(Error::invalid("unexpected trailing XYZ records"));
    }
    if atoms.iter().any(|a| !crate::geometry::finite(a.local)) {
        return Err(Error::invalid("nonfinite XYZ"));
    }
    let mut bonds = Vec::new();
    for i in 0..count {
        for j in i + 1..count {
            let d = norm(sub(atoms[i].local, atoms[j].local));
            if d > 0.1 && d <= 1.2 * (atoms[i].element.covalent + atoms[j].element.covalent) {
                bonds.push(Bond {
                    a: i,
                    b: j,
                    order: None,
                });
            }
        }
    }
    MoleculeDefinition::new(name, atoms, bonds)
}
pub fn template(kind: u32) -> Result<MoleculeDefinition> {
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    let mut push = |s: &str, p| {
        atoms.push(AtomDefinition {
            element: element(s).expect("built-in element"),
            local: p,
        });
    };
    let name = match kind {
        0 => {
            push("O", [0., 0., 0.]);
            push("H", [0.9572, 0., 0.]);
            push("H", [-0.2399872, 0.9266272, 0.]);
            bonds.extend([
                Bond {
                    a: 0,
                    b: 1,
                    order: Some(1),
                },
                Bond {
                    a: 0,
                    b: 2,
                    order: Some(1),
                },
            ]);
            "Water"
        }
        1 => {
            push("C", [0., 0., 0.]);
            let a = 1.09 / 3f64.sqrt();
            for p in [[a, a, a], [a, -a, -a], [-a, a, -a], [-a, -a, a]] {
                push("H", p);
            }
            for b in 1..5 {
                bonds.push(Bond {
                    a: 0,
                    b,
                    order: Some(1),
                });
            }
            "Methane"
        }
        2 => {
            for (s, r) in [("C", 1.397), ("H", 2.487)] {
                for i in 0..6 {
                    let t = i as f64 * std::f64::consts::TAU / 6.;
                    push(s, [r * t.cos(), r * t.sin(), 0.]);
                }
            }
            for i in 0..6 {
                bonds.push(Bond {
                    a: i,
                    b: (i + 1) % 6,
                    order: Some(if i % 2 == 0 { 2 } else { 1 }),
                });
                bonds.push(Bond {
                    a: i,
                    b: i + 6,
                    order: Some(1),
                });
            }
            "Benzene (Kekule depiction)"
        }
        _ => {
            return Err(Error::invalid(
                "template must be 0 water, 1 methane, 2 benzene",
            ))
        }
    };
    MoleculeDefinition::new(name, atoms, bonds)
}
