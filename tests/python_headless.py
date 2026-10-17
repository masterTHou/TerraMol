import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'python'))
from terramol import HeadlessSimulation

with HeadlessSimulation(sys.argv[1]) as sim:
    a = sim.reset()
    assert len(a.atoms) == 150 and len(a.bonds) == 100
    sim.step(100)
    b = sim.reset()
    assert a.topology_version < b.topology_version
    assert [(v.position, v.velocity) for v in a.atoms] == [(v.position, v.velocity) for v in b.atoms]
    assert {v.molecule_id for v in a.atoms}.isdisjoint(v.molecule_id for v in b.atoms)
    sim.velocity(b.atoms[0].molecule_id, (0.01, 0, 0))
    assert abs(sim.step(1).atoms[0].velocity[0] - 0.01) < 1e-8
    try:
        sim.step(-1)
        raise AssertionError('negative step accepted')
    except ValueError:
        pass
sim.close()
try:
    sim.step(1)
    raise AssertionError('closed world accepted')
except RuntimeError:
    pass
print('Python headless reset / observation / action / disposal OK')
