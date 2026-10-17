"""Package an already-tested release DLL; does not publish or bundle LAMMPS."""
import hashlib
from pathlib import Path
import re
import struct
import zipfile

root = Path(__file__).resolve().parents[1]
dll = root / 'target/release/mol_engine_core.dll'
binary = dll.read_bytes()
if binary[:2] != b'MZ':
    raise RuntimeError('Expected a Windows PE library')
pe = struct.unpack_from('<I', binary, 0x3C)[0]
if binary[pe:pe+4] != b'PE\0\0' or struct.unpack_from('<H', binary, pe+4)[0] != 0x8664:
    raise RuntimeError('Release package requires an x64 DLL')
version = re.search(r'^version\s*=\s*"([^"]+)"', (root/'Cargo.toml').read_text(), re.M)[1]
out = root/'artifacts/release'
out.mkdir(parents=True, exist_ok=True)
archive = out/f'TerraMol-v{version}-windows-x64.zip'
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as z:
    for source, target in [
        ('target/release/mol_engine_core.dll', 'mol_engine_core.dll'),
        ('include/mol_engine_core.h', 'include/mol_engine_core.h'),
        ('python/terramol.py', 'python/terramol.py'),
        ('python/README.md', 'python/README.md'),
        ('RELEASE_NOTES.md', 'RELEASE_NOTES.md'),
        ('LICENSE', 'LICENSE'),
    ]:
        z.write(root/source, target)
    z.writestr('README.txt', 'TerraMol core prototype / Windows x64\n'
               'Default Native backend; LAMMPS is not bundled.\n'
               'Full source and Unity setup: https://github.com/masterTHou/TerraMol\n'
               'See RELEASE_NOTES.md for verified behavior and known limitations.\n')
with zipfile.ZipFile(archive) as z:
    assert z.testzip() is None
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
(out/'SHA256SUMS.txt').write_text(f'{digest}  {archive.name}\n', encoding='ascii')
print(archive)
print('SHA256:', digest)
