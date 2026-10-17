use crate::{
    error::{Code, Error, Result},
    world::World,
};
pub const ABI_VERSION: u32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SnapshotHeader {
    pub abi_version: u32,
    pub header_bytes: u32,
    pub atom_count: u32,
    pub bond_count: u32,
    pub state_version: u64,
    pub topology_version: u64,
    pub atom_stride: u32,
    pub bond_stride: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderAtom {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub element: u32,
    pub reserved: u32,
    pub molecule_id: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderBond {
    pub a: u32,
    pub b: u32,
    pub order: u32,
    pub reserved: u32,
}
pub struct Snapshot {
    pub header: SnapshotHeader,
    pub atoms: Vec<RenderAtom>,
    pub bonds: Vec<RenderBond>,
}
impl Snapshot {
    pub fn capture(w: &World) -> Result<Self> {
        let atom_count = u32::try_from(w.atoms().len())
            .map_err(|_| Error::new(Code::Capacity, "snapshot atom count overflow"))?;
        let bond_count = u32::try_from(w.bonds().len())
            .map_err(|_| Error::new(Code::Capacity, "snapshot bond count overflow"))?;
        Ok(Self {
            header: SnapshotHeader {
                abi_version: ABI_VERSION,
                header_bytes: 40,
                atom_count,
                bond_count,
                state_version: w.state_version,
                topology_version: w.topology_version,
                atom_stride: 40,
                bond_stride: 16,
            },
            atoms: w
                .atoms()
                .iter()
                .map(|a| RenderAtom {
                    position: a.position.map(|v| v as f32),
                    velocity: a.velocity.map(|v| v as f32),
                    element: a.number,
                    reserved: 0,
                    molecule_id: a.molecule_id,
                })
                .collect(),
            bonds: w
                .bonds()
                .iter()
                .map(|b| RenderBond {
                    a: b.a as u32,
                    b: b.b as u32,
                    order: b.order.unwrap_or(0) as u32,
                    reserved: 0,
                })
                .collect(),
        })
    }
    pub fn byte_len(&self) -> usize {
        40 + self.atoms.len() * 40 + self.bonds.len() * 16
    }
    /// Explicit little-endian wire encoding: no uninitialized struct padding is copied.
    pub fn bytes(&self) -> Vec<u8> {
        let h = &self.header;
        let mut b = Vec::with_capacity(self.byte_len());
        for v in [h.abi_version, h.header_bytes, h.atom_count, h.bond_count] {
            b.extend(v.to_le_bytes());
        }
        for v in [h.state_version, h.topology_version] {
            b.extend(v.to_le_bytes());
        }
        for v in [h.atom_stride, h.bond_stride] {
            b.extend(v.to_le_bytes());
        }
        for a in &self.atoms {
            for v in a.position.into_iter().chain(a.velocity) {
                b.extend(v.to_le_bytes());
            }
            b.extend(a.element.to_le_bytes());
            b.extend(0u32.to_le_bytes());
            b.extend(a.molecule_id.to_le_bytes());
        }
        for bond in &self.bonds {
            for v in [bond.a, bond.b, bond.order, 0] {
                b.extend(v.to_le_bytes());
            }
        }
        b
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};
    #[test]
    fn layout() {
        assert_eq!(size_of::<SnapshotHeader>(), 40);
        assert_eq!(size_of::<RenderAtom>(), 40);
        assert_eq!(size_of::<RenderBond>(), 16);
        assert_eq!(offset_of!(SnapshotHeader, state_version), 16);
        assert_eq!(offset_of!(RenderAtom, molecule_id), 32);
        assert_eq!(offset_of!(RenderAtom, element), 24);
    }
}
